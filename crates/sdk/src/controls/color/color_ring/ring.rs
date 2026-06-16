use super::common::{
    effective_position as resolve_effective_position, end_drag, pointer_hits_active_target, pointer_to_ring_position,
    position_to_theta, ring_geometry, start_drag, value_from_position, value_percent,
};
use super::model::ColorRingModel;
pub use super::surface::ColorRing;
use crate::controls::color::mouse_behavior::{
    MouseCursorDecision, SharedMousePreset, SharedMousePresetContext, apply_hover_cursor, apply_window_cursor,
    reset_window_cursor_if_claimed, resolve_shared_mouse_preset,
};
use crate::controls::color::style::{Sizable, Size};
use gpui::{prelude::*, *};
use std::sync::Arc;

pub mod sizing {
    pub const RING_THICKNESS_XSMALL: f32 = 7.0;
    pub const RING_THICKNESS_SMALL: f32 = 14.0;
    pub const RING_THICKNESS_MEDIUM: f32 = 20.0;
    pub const RING_THICKNESS_LARGE: f32 = 28.0;

    pub const THUMB_SIZE_SMALL: f32 = 12.0;
    pub const THUMB_SIZE_MEDIUM: f32 = 16.0;
    pub const THUMB_SIZE_LARGE: f32 = 20.0;
}

pub trait ColorRingDelegate: 'static {
    fn style_background(&self, circle: &ColorRingState, container: Div, window: &mut Window, cx: &App) -> Div;

    fn get_color_at_position(&self, circle: &ColorRingState, position: f32) -> Hsla;

    fn position_to_value(&self, _circle: &ColorRingState, position: f32) -> f32 {
        position.rem_euclid(1.0)
    }

    fn value_to_position(&self, _circle: &ColorRingState, value: f32) -> f32 {
        value.clamp(0.0, 1.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum ColorRingMousePreset {
    #[default]
    Default,
    Crosshair,
    Passthrough,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct ColorRingMouseContext {
    pub pointer: Point<Pixels>,
    pub hovered: bool,
    pub contains_pointer: bool,
    pub dragging: bool,
    pub disabled: bool,
    pub external_drag_active: bool,
}

#[derive(Clone)]
pub enum ColorRingMouseBehavior {
    Preset(ColorRingMousePreset),
    Custom(Arc<dyn Fn(ColorRingMouseContext) -> MouseCursorDecision + Send + Sync>),
}

impl Default for ColorRingMouseBehavior {
    fn default() -> Self {
        Self::Preset(ColorRingMousePreset::Default)
    }
}

impl ColorRingMousePreset {
    fn resolve(self, ctx: ColorRingMouseContext) -> MouseCursorDecision {
        let (shared_preset, active_cursor_style) = match self {
            ColorRingMousePreset::Default => (SharedMousePreset::Default, CursorStyle::PointingHand),
            ColorRingMousePreset::Crosshair => (SharedMousePreset::Crosshair, CursorStyle::Crosshair),
            ColorRingMousePreset::Passthrough => (SharedMousePreset::Passthrough, CursorStyle::Arrow),
        };

        resolve_shared_mouse_preset(
            shared_preset,
            SharedMousePresetContext {
                hovered: ctx.hovered,
                contains_pointer: ctx.contains_pointer,
                dragging: ctx.dragging,
                external_drag_active: ctx.external_drag_active,
            },
            active_cursor_style,
        )
    }
}

impl ColorRingMouseBehavior {
    fn resolve(&self, ctx: ColorRingMouseContext) -> MouseCursorDecision {
        match self {
            ColorRingMouseBehavior::Preset(preset) => preset.resolve(ctx),
            ColorRingMouseBehavior::Custom(resolver) => resolver(ctx),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColorRingEvent {
    Change(f32),
    Release(f32),
}

pub struct ColorRingState {
    pub id: SharedString,
    pub value: f32,
    pub range: std::ops::Range<f32>,
    pub step: Option<f32>,
    pub reversed: bool,
    pub size: Size,
    pub ring_thickness: Option<f32>,
    pub ring_thickness_size: Option<Size>,
    pub thumb_size: Option<f32>,
    pub bounds: Bounds<Pixels>,
    pub delegate: Box<dyn ColorRingDelegate>,
    pub style: StyleRefinement,
    pub disabled: bool,
    pub ring_inner_border: bool,
    pub ring_outer_border: bool,
    pub ring_border_color: Option<Hsla>,
    pub rotation_degrees: f32,
    pub allow_inner_target: bool,
    pub mouse_behavior: ColorRingMouseBehavior,
    interaction_position: Option<f32>,
    drag_interaction_enabled: bool,
    window_cursor_claimed: bool,
    pub focus_handle: FocusHandle,
}

impl ColorRingState {
    pub fn from_model<V: 'static>(model: ColorRingModel, cx: &mut Context<V>) -> Self {
        if let Err(err) = model.validate() {
            panic!("invalid ColorRingModel: {err}");
        }

        let mut this = Self {
            id: model.id,
            value: model.value,
            range: model.range,
            step: model.step,
            reversed: model.reversed,
            size: model.size,
            ring_thickness: model.ring_thickness,
            ring_thickness_size: model.ring_thickness_size,
            thumb_size: model.thumb_size,
            bounds: Bounds::default(),
            delegate: model.delegate,
            style: StyleRefinement::default(),
            disabled: model.disabled,
            ring_inner_border: model.ring_inner_border,
            ring_outer_border: model.ring_outer_border,
            ring_border_color: model.ring_border_color,
            rotation_degrees: model.rotation_degrees,
            allow_inner_target: model.allow_inner_target,
            mouse_behavior: model.mouse_behavior,
            interaction_position: None,
            drag_interaction_enabled: false,
            window_cursor_claimed: false,
            focus_handle: cx.focus_handle(),
        };

        this.value = this.clamp_to_range(this.value);
        this
    }

    pub fn new<V: 'static>(
        id: impl Into<SharedString>,
        value: f32,
        delegate: Box<dyn ColorRingDelegate>,
        cx: &mut Context<V>,
    ) -> Self {
        Self::from_model(ColorRingModel::new(id, value, delegate), cx)
    }

    #[allow(dead_code)] // Raster-first default; use `*_with_renderer(..., Vector, ...)` to opt in.
    pub fn saturation<V: 'static>(
        id: impl Into<SharedString>,
        value: f32,
        delegate: super::delegates::SaturationRingDelegate,
        cx: &mut Context<V>,
    ) -> Self {
        Self::saturation_raster(id, value, delegate.hue, delegate.hsv_value, cx)
    }

    #[allow(dead_code)] // Raster-first default; use `*_with_renderer(..., Vector, ...)` to opt in.
    pub fn hue<V: 'static>(
        id: impl Into<SharedString>,
        value: f32,
        delegate: super::delegates::HueRingDelegate,
        cx: &mut Context<V>,
    ) -> Self {
        Self::hue_raster(id, value, delegate.saturation, delegate.lightness, cx)
    }

    #[allow(dead_code)] // Raster-first default; use `*_with_renderer(..., Vector, ...)` to opt in.
    pub fn lightness<V: 'static>(
        id: impl Into<SharedString>,
        value: f32,
        delegate: super::delegates::LightnessRingDelegate,
        cx: &mut Context<V>,
    ) -> Self {
        Self::lightness_raster(id, value, delegate.hue, delegate.saturation, cx)
    }

    #[allow(dead_code)] // Kept for API parity with ColorSlider-style builders.
    pub fn min(mut self, min: f32) -> Self {
        self.range.start = min;
        self
    }

    #[allow(dead_code)] // Kept for API parity with ColorSlider-style builders.
    pub fn max(mut self, max: f32) -> Self {
        self.range.end = max;
        self
    }

    #[allow(dead_code)] // Kept for API parity with ColorSlider-style builders.
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

    #[allow(dead_code)] // Kept for API parity with ColorSlider-style builders.
    pub fn ring_thickness(mut self, thickness: f32) -> Self {
        self.ring_thickness = Some(thickness.max(1.0));
        self
    }

    pub fn ring_thickness_size(mut self, size: impl Into<Size>) -> Self {
        self.ring_thickness_size = Some(size.into());
        self
    }

    #[allow(dead_code)] // Kept for API parity with ColorSlider-style builders.
    pub fn thumb_size(mut self, size: f32) -> Self {
        self.thumb_size = Some(size.max(4.0));
        self
    }

    pub fn ring_inner_border(mut self, enabled: bool) -> Self {
        self.ring_inner_border = enabled;
        self
    }

    pub fn ring_outer_border(mut self, enabled: bool) -> Self {
        self.ring_outer_border = enabled;
        self
    }

    pub fn ring_border_color(mut self, color: Hsla) -> Self {
        self.ring_border_color = Some(color);
        self
    }

    #[allow(dead_code)] // Public builder for rotating ring orientation.
    pub fn rotation_degrees(mut self, degrees: f32) -> Self {
        self.rotation_degrees = degrees;
        self
    }

    #[allow(dead_code)] // Optional: allow activation from center hole area.
    pub fn allow_inner_target(mut self, allow: bool) -> Self {
        self.allow_inner_target = allow;
        self
    }

    #[allow(dead_code)]
    pub fn mouse_preset(mut self, preset: ColorRingMousePreset) -> Self {
        self.mouse_behavior = ColorRingMouseBehavior::Preset(preset);
        self
    }

    #[allow(dead_code)]
    pub fn mouse_behavior(mut self, behavior: ColorRingMouseBehavior) -> Self {
        self.mouse_behavior = behavior;
        self
    }

    #[allow(dead_code)]
    pub fn mouse_behavior_custom<F>(mut self, resolver: F) -> Self
    where
        F: Fn(ColorRingMouseContext) -> MouseCursorDecision + Send + Sync + 'static,
    {
        self.mouse_behavior = ColorRingMouseBehavior::Custom(Arc::new(resolver));
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

    pub fn set_value_with_angle_turns(&mut self, value: f32, angle_turns: f32, cx: &mut Context<Self>) {
        let clamped_value = self.clamp_to_range(value);
        let mut position = (angle_turns.rem_euclid(1.0) + 0.25 - self.rotation_turns()).rem_euclid(1.0);
        if self.reversed {
            position = 1.0 - position;
        }

        let value_changed = self.value != clamped_value;
        let position_changed =
            self.interaction_position.map(|current| (current - position).abs() > f32::EPSILON).unwrap_or(true);

        if value_changed || position_changed {
            self.value = clamped_value;
            self.interaction_position = Some(position);
            cx.notify();
        }
    }

    pub fn effective_angle_turns(&self) -> f32 {
        (self.effective_position() - 0.25 + self.rotation_turns()).rem_euclid(1.0)
    }

    pub fn set_delegate(&mut self, delegate: Box<dyn ColorRingDelegate>, cx: &mut Context<Self>) {
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

    #[allow(dead_code)] // Public setter kept for symmetry with builder API.
    pub fn set_ring_inner_border(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.ring_inner_border != enabled {
            self.ring_inner_border = enabled;
            cx.notify();
        }
    }

    #[allow(dead_code)] // Public setter kept for symmetry with builder API.
    pub fn set_ring_outer_border(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.ring_outer_border != enabled {
            self.ring_outer_border = enabled;
            cx.notify();
        }
    }

    #[allow(dead_code)] // Runtime setter for rotating ring orientation.
    pub fn set_rotation_degrees(&mut self, degrees: f32, cx: &mut Context<Self>) {
        if (self.rotation_degrees - degrees).abs() > f32::EPSILON {
            self.rotation_degrees = degrees;
            cx.notify();
        }
    }

    #[allow(dead_code)]
    pub fn set_mouse_behavior(&mut self, behavior: ColorRingMouseBehavior, cx: &mut Context<Self>) {
        self.mouse_behavior = behavior;
        cx.notify();
    }

    pub fn rotation_turns(&self) -> f32 {
        (self.rotation_degrees / 360.0).rem_euclid(1.0)
    }

    pub fn ring_thickness_px(&self) -> f32 {
        if let Some(thickness) = self.ring_thickness {
            return thickness;
        }

        let thickness_size = self.ring_thickness_size.unwrap_or(self.size);
        match thickness_size {
            Size::XSmall => sizing::RING_THICKNESS_XSMALL,
            Size::Small => sizing::RING_THICKNESS_SMALL,
            Size::Medium => sizing::RING_THICKNESS_MEDIUM,
            Size::Large => sizing::RING_THICKNESS_LARGE,
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

impl ColorRingState {
    fn clamp_to_range(&self, value: f32) -> f32 {
        value.clamp(self.range.start.min(self.range.end), self.range.end.max(self.range.start))
    }

    fn value_to_percent(&self) -> f32 {
        value_percent(self.value, self.range.clone())
    }

    pub(super) fn update_from_mouse(&mut self, position: Point<Pixels>, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(position) = pointer_to_ring_position(self.bounds, position, self.rotation_turns(), self.reversed)
        else {
            return;
        };

        self.value = value_from_position(position, self.range.clone(), self.step, |position| {
            self.delegate.position_to_value(self, position)
        });
        self.interaction_position = Some(position);

        cx.emit(ColorRingEvent::Change(self.value));
        cx.notify();
    }

    fn emit_release(&self, cx: &mut Context<Self>) {
        cx.emit(ColorRingEvent::Release(self.value));
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
                position
            },
            |position| self.delegate.position_to_value(self, position),
        )
    }

    pub(super) fn accepts_pointer_at(&self, pointer: Point<Pixels>) -> bool {
        let Some(geometry) = ring_geometry(self.bounds, self.ring_thickness_px()) else {
            return false;
        };

        let theta = position_to_theta(self.effective_position(), self.rotation_turns());
        pointer_hits_active_target(pointer, geometry, theta, self.thumb_size_px(), self.allow_inner_target)
    }

    #[allow(dead_code)]
    pub fn contains_pointer(&self, pointer: Point<Pixels>) -> bool {
        self.accepts_pointer_at(pointer)
    }

    fn resolve_mouse_cursor(
        &self,
        pointer: Point<Pixels>,
        hovered: bool,
        external_drag_active: bool,
    ) -> MouseCursorDecision {
        let contains_pointer = self.accepts_pointer_at(pointer);
        self.mouse_behavior.resolve(ColorRingMouseContext {
            pointer,
            hovered,
            contains_pointer,
            dragging: self.drag_interaction_enabled,
            disabled: self.disabled,
            external_drag_active,
        })
    }

    fn apply_idle_cursor_handoff(
        &mut self,
        window: &mut Window,
        hitbox: &Hitbox,
        decision: MouseCursorDecision,
        hovered: bool,
    ) {
        if hovered {
            // Avoid a one-frame Arrow flash on release:
            // if we still hover the ring, drop the window-cursor
            // claim and let hitbox hover cursor take over directly.
            self.window_cursor_claimed = false;
            apply_hover_cursor(window, hitbox, decision);
        } else {
            reset_window_cursor_if_claimed(window, &mut self.window_cursor_claimed);
        }
    }

    pub(super) fn update_cursor_state(
        &mut self,
        pointer: Point<Pixels>,
        hovered: bool,
        external_drag_active: bool,
        window: &mut Window,
        hitbox: &Hitbox,
    ) {
        let decision = self.resolve_mouse_cursor(pointer, hovered, external_drag_active);
        if self.drag_interaction_enabled {
            apply_window_cursor(window, decision, &mut self.window_cursor_claimed);
        } else {
            self.apply_idle_cursor_handoff(window, hitbox, decision, hovered);
        }
    }

    pub(super) fn handle_drag_move(&mut self, pointer: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        if !self.drag_interaction_enabled {
            return;
        }
        self.update_from_mouse(pointer, window, cx);
    }

    pub(super) fn handle_drag_release(
        &mut self,
        _pointer: Point<Pixels>,
        _hovered_after_release: bool,
        window: &mut Window,
        _hitbox: &Hitbox,
        cx: &mut Context<Self>,
    ) {
        if !end_drag(&mut self.drag_interaction_enabled) {
            return;
        }

        self.emit_release(cx);
        cx.notify();
        window.refresh();
    }

    pub(super) fn is_drag_interaction_enabled(&self) -> bool {
        self.drag_interaction_enabled
    }

    pub(super) fn begin_drag_from_pointer(
        &mut self,
        pointer: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let accepts_pointer = self.accepts_pointer_at(pointer);
        if !start_drag(&mut self.drag_interaction_enabled, accepts_pointer) {
            return false;
        }

        self.focus_handle.focus(window, cx);
        self.update_from_mouse(pointer, window, cx);
        true
    }

    pub(super) fn apply_keyboard_position(&mut self, value: f32, position: f32, cx: &mut Context<Self>) {
        self.value = value;
        self.interaction_position = Some(position);
        cx.notify();
    }
}

impl Styled for ColorRingState {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Sizable for ColorRingState {
    fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl EventEmitter<ColorRingEvent> for ColorRingState {}

impl Render for ColorRingState {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        super::surface::ColorRing::new(&cx.entity())
    }
}
