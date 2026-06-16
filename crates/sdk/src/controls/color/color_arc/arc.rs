#![allow(dead_code)]

use super::common::{
    arc_geometry, effective_position as resolve_effective_position, pointer_hits_arc_target, pointer_to_arc_position,
    position_to_turn, start_turns, sweep_turns, value_from_position, value_percent,
};
use super::model::ColorArcModel;
pub use super::surface::ColorArc;
use crate::controls::color::style::{ActiveTheme as _, Sizable, Size};
use gpui::{prelude::*, *};

pub mod sizing {
    pub const ARC_THICKNESS_XSMALL: f32 = 7.0;
    pub const ARC_THICKNESS_SMALL: f32 = 14.0;
    pub const ARC_THICKNESS_MEDIUM: f32 = 20.0;
    pub const ARC_THICKNESS_LARGE: f32 = 28.0;

    pub const THUMB_SIZE_SMALL: f32 = 12.0;
    pub const THUMB_SIZE_MEDIUM: f32 = 16.0;
    pub const THUMB_SIZE_LARGE: f32 = 20.0;
}

pub trait ColorArcDelegate: 'static {
    fn style_background(&self, arc: &ColorArcState, container: Div, window: &mut Window, cx: &App) -> Div;

    fn get_color_at_position(&self, arc: &ColorArcState, position: f32) -> Hsla;

    fn position_to_value(&self, _arc: &ColorArcState, position: f32) -> f32 {
        position.clamp(0.0, 1.0)
    }

    fn value_to_position(&self, _arc: &ColorArcState, value: f32) -> f32 {
        value.clamp(0.0, 1.0)
    }

    fn prewarm_raster_cache(&self, _arc: &ColorArcState, _image_size: gpui::Size<Pixels>, _border_color: Hsla) {}

    fn renderer(&self) -> super::raster::ColorArcRenderer {
        super::raster::ColorArcRenderer::Vector
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColorArcEvent {
    Change(f32),
    Release(f32),
}

pub struct ColorArcState {
    pub id: SharedString,
    pub value: f32,
    pub range: std::ops::Range<f32>,
    pub step: Option<f32>,
    pub reversed: bool,
    pub size: Size,
    pub arc_thickness: Option<f32>,
    pub arc_thickness_size: Option<Size>,
    pub thumb_size: Option<f32>,
    pub bounds: Bounds<Pixels>,
    pub delegate: Box<dyn ColorArcDelegate>,
    pub style: StyleRefinement,
    pub disabled: bool,
    pub start_degrees: f32,
    pub sweep_degrees: f32,
    interaction_position: Option<f32>,
    drag_interaction_enabled: bool,
    pub focus_handle: FocusHandle,
}

impl ColorArcState {
    fn clamp_to_range(&self, value: f32) -> f32 {
        value.clamp(self.range.start.min(self.range.end), self.range.end.max(self.range.start))
    }

    pub fn from_model<V: 'static>(model: ColorArcModel, cx: &mut Context<V>) -> Self {
        if let Err(err) = model.validate() {
            panic!("invalid ColorArcModel: {err}");
        }

        let mut this = Self {
            id: model.id,
            value: model.value,
            range: model.range,
            step: model.step,
            reversed: model.reversed,
            size: model.size,
            arc_thickness: model.arc_thickness,
            arc_thickness_size: model.arc_thickness_size,
            thumb_size: model.thumb_size,
            bounds: Bounds::default(),
            delegate: model.delegate,
            style: StyleRefinement::default(),
            disabled: model.disabled,
            start_degrees: model.start_degrees,
            sweep_degrees: model.sweep_degrees,
            interaction_position: None,
            drag_interaction_enabled: false,
            focus_handle: cx.focus_handle(),
        };

        this.value = this.clamp_to_range(this.value);
        this
    }

    pub fn new<V: 'static>(
        id: impl Into<SharedString>,
        value: f32,
        delegate: Box<dyn ColorArcDelegate>,
        cx: &mut Context<V>,
    ) -> Self {
        Self::from_model(ColorArcModel::new(id, value, delegate), cx)
    }

    pub fn hue<V: 'static>(
        id: impl Into<SharedString>,
        value: f32,
        delegate: super::delegates::HueArcDelegate,
        cx: &mut Context<V>,
    ) -> Self {
        // Raster-first default for performance; use `*_with_renderer(..., Vector, ...)` to opt in.
        Self::hue_raster(id, value, delegate.saturation, delegate.lightness, cx)
    }

    pub fn saturation<V: 'static>(
        id: impl Into<SharedString>,
        value: f32,
        delegate: super::delegates::SaturationArcDelegate,
        cx: &mut Context<V>,
    ) -> Self {
        // Raster-first default for performance; use `*_with_renderer(..., Vector, ...)` to opt in.
        Self::saturation_raster(id, value, delegate.hue, delegate.hsv_value, cx)
    }

    pub fn lightness<V: 'static>(
        id: impl Into<SharedString>,
        value: f32,
        delegate: super::delegates::LightnessArcDelegate,
        cx: &mut Context<V>,
    ) -> Self {
        // Raster-first default for performance; use `*_with_renderer(..., Vector, ...)` to opt in.
        Self::lightness_raster(id, value, delegate.hue, delegate.saturation, cx)
    }

    pub fn min(mut self, min: f32) -> Self {
        self.range.start = min;
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.range.end = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step.abs());
        self
    }

    pub fn reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }

    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.disabled = !enabled;
        self
    }

    pub fn arc_thickness(mut self, thickness: f32) -> Self {
        self.arc_thickness = Some(thickness.max(1.0));
        self
    }

    pub fn arc_thickness_size(mut self, size: impl Into<Size>) -> Self {
        self.arc_thickness_size = Some(size.into());
        self
    }

    pub fn thumb_size(mut self, size: f32) -> Self {
        self.thumb_size = Some(size.max(4.0));
        self
    }

    pub fn start_degrees(mut self, degrees: f32) -> Self {
        self.start_degrees = degrees;
        self
    }

    pub fn sweep_degrees(mut self, degrees: f32) -> Self {
        self.sweep_degrees = degrees.max(0.0);
        self
    }

    pub fn set_value(&mut self, value: f32, cx: &mut Context<Self>) {
        let clamped_value = self.clamp_to_range(value);
        if self.value != clamped_value {
            self.value = clamped_value;
            self.interaction_position = None;
            cx.notify();
        }
    }

    pub fn set_delegate(&mut self, delegate: Box<dyn ColorArcDelegate>, cx: &mut Context<Self>) {
        self.delegate = delegate;
        cx.notify();
    }

    #[allow(dead_code)] // Parity helper with other lookless controls.
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        let disabled = !enabled;
        if self.disabled != disabled {
            self.disabled = disabled;
            cx.notify();
        }
    }

    #[allow(dead_code)] // Runtime setter kept for parity with builder API.
    pub fn set_reversed(&mut self, reversed: bool, cx: &mut Context<Self>) {
        if self.reversed != reversed {
            self.reversed = reversed;
            self.interaction_position = None;
            cx.notify();
        }
    }

    #[allow(dead_code)] // Optional prewarm hook to avoid first-interaction raster hitching.
    pub fn prewarm_raster_cache_square_in_place(&mut self, size_px: f32, cx: &mut Context<Self>) {
        let side = px(size_px.max(1.0));
        let image_size = size(side, side);
        self.delegate.prewarm_raster_cache(self, image_size, cx.theme().border);
    }

    pub fn set_start_degrees(&mut self, degrees: f32, cx: &mut Context<Self>) {
        if (self.start_degrees - degrees).abs() > f32::EPSILON {
            self.start_degrees = degrees;
            cx.notify();
        }
    }

    pub fn set_sweep_degrees(&mut self, degrees: f32, cx: &mut Context<Self>) {
        let degrees = degrees.max(0.0);
        if (self.sweep_degrees - degrees).abs() > f32::EPSILON {
            self.sweep_degrees = degrees;
            cx.notify();
        }
    }

    pub fn start_turns(&self) -> f32 {
        start_turns(self.start_degrees)
    }

    pub fn sweep_turns(&self) -> f32 {
        sweep_turns(self.sweep_degrees)
    }

    pub fn arc_thickness_px(&self) -> f32 {
        if let Some(thickness) = self.arc_thickness {
            return thickness;
        }

        let thickness_size = self.arc_thickness_size.unwrap_or(self.size);
        match thickness_size {
            Size::XSmall => sizing::ARC_THICKNESS_XSMALL,
            Size::Small => sizing::ARC_THICKNESS_SMALL,
            Size::Medium => sizing::ARC_THICKNESS_MEDIUM,
            Size::Large => sizing::ARC_THICKNESS_LARGE,
            Size::Size(px) => (px.as_f32() * 0.1).max(8.0),
        }
    }

    pub fn thumb_size_px(&self) -> f32 {
        self.thumb_size.unwrap_or(match self.size {
            Size::XSmall | Size::Small => sizing::THUMB_SIZE_SMALL,
            Size::Medium => sizing::THUMB_SIZE_MEDIUM,
            Size::Large => sizing::THUMB_SIZE_LARGE,
            Size::Size(px) => (px.as_f32() * 0.08).max(10.0),
        })
    }
}

impl ColorArcState {
    fn value_to_percent(&self) -> f32 {
        value_percent(self.value, self.range.clone())
    }

    pub(super) fn update_from_mouse(&mut self, pointer: Point<Pixels>, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(position) =
            pointer_to_arc_position(self.bounds, pointer, self.start_turns(), self.sweep_turns(), self.reversed)
        else {
            return;
        };

        self.value = value_from_position(position, self.range.clone(), self.step, |position| {
            self.delegate.position_to_value(self, position)
        });
        self.interaction_position = Some(position);

        cx.emit(ColorArcEvent::Change(self.value));
        cx.notify();
    }

    pub(super) fn emit_release(&self, cx: &mut Context<Self>) {
        cx.emit(ColorArcEvent::Release(self.value));
    }

    pub(super) fn effective_position(&self) -> f32 {
        let value_percent = self.value_to_percent();
        resolve_effective_position(
            value_percent,
            self.interaction_position,
            |value_percent| {
                let mut position = self.delegate.value_to_position(self, value_percent);
                if self.reversed {
                    position = 1.0 - position;
                }
                position.clamp(0.0, 1.0)
            },
            |position| self.delegate.position_to_value(self, position),
        )
    }

    pub(super) fn position_to_turn(&self, position: f32) -> f32 {
        let logical_position = if self.reversed { 1.0 - position } else { position };
        position_to_turn(logical_position, self.start_turns(), self.sweep_turns())
    }

    pub(super) fn accepts_pointer_at(&self, pointer: Point<Pixels>) -> bool {
        let Some(geometry) = arc_geometry(self.bounds, self.arc_thickness_px()) else {
            return false;
        };

        let thumb_turn = self.position_to_turn(self.effective_position());
        pointer_hits_arc_target(
            pointer,
            geometry,
            self.start_turns(),
            self.sweep_turns(),
            thumb_turn,
            self.thumb_size_px(),
        )
    }

    pub(super) fn keyboard_step(&self, key: &str, modifiers: Modifiers) -> Option<f32> {
        let base_step = self.step.unwrap_or((self.range.end - self.range.start).abs() / 100.0);
        let multiplier = if modifiers.shift {
            10.0
        } else if modifiers.alt {
            0.1
        } else {
            1.0
        };
        let step = base_step * multiplier;

        let mut delta_sign = match key {
            "left" | "up" => -1.0,
            "right" | "down" => 1.0,
            _ => return None,
        };

        if self.reversed {
            delta_sign = -delta_sign;
        }

        Some(self.value + delta_sign * step)
    }

    pub(super) fn set_bounds(&mut self, bounds: Bounds<Pixels>) {
        self.bounds = bounds;
    }

    pub(super) fn begin_drag_from_pointer(
        &mut self,
        pointer: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.accepts_pointer_at(pointer) {
            return false;
        }

        self.drag_interaction_enabled = true;
        self.focus_handle.focus(window, cx);
        self.update_from_mouse(pointer, window, cx);
        true
    }

    pub(super) fn is_drag_interaction_enabled(&self) -> bool {
        self.drag_interaction_enabled
    }

    pub(super) fn set_drag_interaction_enabled(&mut self, enabled: bool) {
        self.drag_interaction_enabled = enabled;
    }

    pub(super) fn apply_keyboard_value(&mut self, clamped_value: f32, cx: &mut Context<Self>) -> f32 {
        self.value = clamped_value;
        let value_pct = value_percent(clamped_value, self.range.clone());
        let mut pos = self.delegate.value_to_position(self, value_pct);
        if self.reversed {
            pos = 1.0 - pos;
        }
        self.interaction_position = Some(pos.clamp(0.0, 1.0));
        cx.notify();
        clamped_value
    }
}

impl Styled for ColorArcState {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Sizable for ColorArcState {
    fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl EventEmitter<ColorArcEvent> for ColorArcState {}

impl Render for ColorArcState {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        super::surface::ColorArc::new(&cx.entity())
    }
}
