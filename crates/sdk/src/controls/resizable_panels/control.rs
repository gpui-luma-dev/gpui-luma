use gpui::{
    Context, DragMoveEvent, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent,
    MouseUpEvent, ParentElement, Pixels, Render, SharedString, Window, div, prelude::*,
};

use super::{
    math::{
        all_panels_use_weight, apply_pair_delta, apply_pair_delta_px, content_axis_size,
        layout_states_to_legacy_percents, normalize_weights, solve_layout_px, PanelSizeBounds,
    },
    model::{
        PanelLayoutState, ResizablePanelsBuilder, ResizablePanelsModel, ResizablePanelsOrientation,
        ResizablePanelsRenderModel,
    },
};
use crate::theme::InteractionState;

#[derive(Clone, Debug)]
pub struct ResizablePanelsHandleDrag {
    pub(crate) id: SharedString,
    pub(crate) handle_index: usize,
}

impl Render for ResizablePanelsHandleDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

#[derive(Clone, Debug)]
pub enum ResizablePanelsEvent {
    ResizeStart,
    /// Legacy percent-of-content values (100 total for weight-only strips).
    SizesChanged {
        sizes: Vec<f32>,
    },
    ResizeEnd {
        sizes: Vec<f32>,
    },
}

pub struct ResizablePanels {
    model: ResizablePanelsModel,
    layout_states: Vec<PanelLayoutState>,
    panel_sizes_px: Vec<f32>,
    handle_focuses: Vec<FocusHandle>,
    dragging_handle: Option<usize>,
    drag_start_axis_px: f32,
    drag_start_states: Vec<PanelLayoutState>,
    measured_size: Option<gpui::Size<Pixels>>,
}

impl EventEmitter<ResizablePanelsEvent> for ResizablePanels {}

impl ResizablePanels {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ResizablePanelsBuilder {
        ResizablePanelsBuilder::new(id)
    }

    pub fn horizontal(id: impl Into<SharedString>) -> ResizablePanelsBuilder {
        ResizablePanelsBuilder::horizontal(id)
    }

    pub fn vertical(id: impl Into<SharedString>) -> ResizablePanelsBuilder {
        ResizablePanelsBuilder::vertical(id)
    }

    pub(crate) fn from_builder(builder: ResizablePanelsBuilder, cx: &mut Context<Self>) -> Self {
        let mut layout_states: Vec<PanelLayoutState> =
            builder.model.panels.iter().map(PanelLayoutState::from_spec).collect();

        if all_panels_use_weight(&builder.model.panels) {
            let mut weights: Vec<f32> = layout_states
                .iter()
                .map(|state| match state {
                    PanelLayoutState::Weight(weight) => *weight,
                    PanelLayoutState::Absolute(px) => *px,
                })
                .collect();
            normalize_weights(&mut weights);
            for (state, weight) in layout_states.iter_mut().zip(weights) {
                *state = PanelLayoutState::Weight(weight);
            }
        }

        if layout_states.len() != builder.model.panels.len() {
            layout_states = vec![PanelLayoutState::Weight(100.0)];
        }

        let handle_focuses = (0..builder.model.panels.len().saturating_sub(1)).map(|_| cx.focus_handle()).collect();

        let panel_sizes_px = solve_layout_px(&builder.model.panels, &layout_states, 1.0);

        Self {
            model: builder.model,
            layout_states,
            panel_sizes_px,
            handle_focuses,
            dragging_handle: None,
            drag_start_axis_px: 0.0,
            drag_start_states: Vec::new(),
            measured_size: None,
        }
    }

    pub fn set_measured_size(&mut self, size: gpui::Size<Pixels>, cx: &mut Context<Self>) {
        if self.measured_size == Some(size) {
            return;
        }
        self.measured_size = Some(size);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }
        self.model.enabled = enabled;
        if !enabled {
            self.dragging_handle = None;
        }
        cx.notify();
    }

    pub fn set_show_handle(&mut self, show_handle: bool, cx: &mut Context<Self>) {
        if self.model.show_handle == show_handle {
            return;
        }
        self.model.show_handle = show_handle;
        cx.notify();
    }

    pub fn set_resize_handle(&mut self, size: super::model::ResizeHandleSize, cx: &mut Context<Self>) {
        if self.model.resize_handle == size {
            return;
        }
        self.model.resize_handle = size;
        cx.notify();
    }

    /// Legacy API; maps the pixel width to the nearest [`ResizeHandleSize`] preset.
    #[deprecated(note = "use set_resize_handle(ResizeHandleSize::Sm | Md | Lg)")]
    pub fn set_handle_size(&mut self, lane_width: Pixels, cx: &mut Context<Self>) {
        self.set_resize_handle(super::model::ResizeHandleSize::from_lane_px(lane_width.as_f32()), cx);
    }

    pub fn set_frame_size(&mut self, width: Pixels, height: Pixels, cx: &mut Context<Self>) {
        if self.model.frame_width == Some(width) && self.model.frame_height == Some(height) {
            return;
        }
        self.model.frame_width = Some(width);
        self.model.frame_height = Some(height);
        cx.notify();
    }

    /// Updates layout state. For weight-only strips, values are normalized weight percents (legacy).
    pub fn set_sizes(&mut self, mut sizes: Vec<f32>, cx: &mut Context<Self>) {
        if sizes.len() != self.model.panels.len() || sizes.is_empty() {
            return;
        }

        if all_panels_use_weight(&self.model.panels) {
            normalize_weights(&mut sizes);
            let changed = self.layout_states.iter().zip(sizes.iter()).any(|(current, next)| match current {
                PanelLayoutState::Weight(weight) => (weight - next).abs() >= f32::EPSILON,
                PanelLayoutState::Absolute(px) => (px - next).abs() >= f32::EPSILON,
            });
            if !changed {
                return;
            }
            for (state, weight) in self.layout_states.iter_mut().zip(sizes.iter()) {
                *state = PanelLayoutState::Weight(*weight);
            }
        } else {
            return;
        }

        cx.emit(ResizablePanelsEvent::SizesChanged { sizes });
        cx.notify();
    }

    /// Legacy percent-of-content snapshot (100 total for weight-only strips).
    pub fn sizes(&self) -> Vec<f32> {
        self.legacy_sizes()
    }

    pub fn layout_states(&self) -> Vec<PanelLayoutState> {
        self.layout_states.clone()
    }

    pub fn orientation(&self) -> ResizablePanelsOrientation {
        self.model.orientation
    }

    pub fn enabled(&self) -> bool {
        self.model.enabled
    }

    pub fn show_handle(&self) -> bool {
        self.model.show_handle
    }

    fn axis_position(&self, position: gpui::Point<Pixels>) -> f32 {
        match self.model.orientation {
            ResizablePanelsOrientation::Horizontal => position.x.as_f32(),
            ResizablePanelsOrientation::Vertical => position.y.as_f32(),
        }
    }

    fn main_axis_px(&self) -> f32 {
        match (self.model.frame_width, self.model.frame_height, self.model.orientation) {
            (Some(width), _, ResizablePanelsOrientation::Horizontal) => width.as_f32(),
            (_, Some(height), ResizablePanelsOrientation::Vertical) => height.as_f32(),
            (None, _, ResizablePanelsOrientation::Horizontal) => {
                self.measured_size.map(|s| s.width.as_f32()).unwrap_or(1.0)
            }
            (_, None, ResizablePanelsOrientation::Vertical) => {
                self.measured_size.map(|s| s.height.as_f32()).unwrap_or(1.0)
            }
        }
    }

    fn content_axis_size_px(&self) -> f32 {
        let panel_count = self.layout_states.len();
        content_axis_size(self.main_axis_px(), 0.0, panel_count)
    }

    fn legacy_sizes(&self) -> Vec<f32> {
        layout_states_to_legacy_percents(&self.model.panels, &self.layout_states, self.content_axis_size_px())
    }

    fn apply_pair_delta(&mut self, index: usize, delta_px: f32) -> bool {
        if index + 1 >= self.layout_states.len() {
            return false;
        }

        if all_panels_use_weight(&self.model.panels) {
            let mut legacy: Vec<f32> = self
                .layout_states
                .iter()
                .map(|state| match state {
                    PanelLayoutState::Weight(weight) => *weight,
                    PanelLayoutState::Absolute(px) => *px,
                })
                .collect();
            let delta_percent = (delta_px / self.content_axis_size_px()) * 100.0;
            let left = PanelSizeBounds {
                min_size: self.model.panels[index].min_size,
                max_size: self.model.panels[index].max_size,
            };
            let right = PanelSizeBounds {
                min_size: self.model.panels[index + 1].min_size,
                max_size: self.model.panels[index + 1].max_size,
            };
            if !apply_pair_delta(&mut legacy, index, delta_percent, left, right) {
                return false;
            }
            for (state, weight) in self.layout_states.iter_mut().zip(legacy.iter()) {
                *state = PanelLayoutState::Weight(*weight);
            }
            return true;
        }

        let content_main_px = self.content_axis_size_px();
        apply_pair_delta_px(&self.model.panels, &mut self.layout_states, index, delta_px, content_main_px)
    }

    fn keyboard_delta_px(&self, signed_step_px: f32) -> f32 {
        if all_panels_use_weight(&self.model.panels) {
            (signed_step_px / 100.0) * self.content_axis_size_px()
        } else {
            signed_step_px
        }
    }

    fn emit_sizes_changed_if_needed(&mut self, changed: bool, cx: &mut Context<Self>) {
        if !changed {
            return;
        }
        cx.emit(ResizablePanelsEvent::SizesChanged { sizes: self.legacy_sizes() });
        cx.notify();
    }

    pub(crate) fn handle_handle_mouse_down(
        &mut self,
        index: usize,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled || index >= self.handle_focuses.len() {
            return;
        }

        self.handle_focuses[index].focus(window, cx);
        self.dragging_handle = Some(index);
        self.drag_start_axis_px = self.axis_position(event.position);
        self.drag_start_states = self.layout_states.clone();
        cx.emit(ResizablePanelsEvent::ResizeStart);
        cx.notify();
    }

    pub(crate) fn handle_drag_move(
        &mut self,
        event: &DragMoveEvent<ResizablePanelsHandleDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled {
            return;
        }

        let drag = event.drag(cx);
        if drag.id != self.model.id {
            return;
        }
        let Some(index) = self.dragging_handle else {
            return;
        };
        if index != drag.handle_index || index + 1 >= self.layout_states.len() {
            return;
        }

        self.layout_states.clone_from(&self.drag_start_states);
        let delta_px = self.axis_position(event.event.position) - self.drag_start_axis_px;
        let changed = self.apply_pair_delta(index, delta_px);
        if changed {
            self.refresh_panel_sizes_px();
        }
        self.emit_sizes_changed_if_needed(changed, cx);
    }

    pub(crate) fn finish_drag(&mut self, event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || event.button != MouseButton::Left {
            return;
        }
        if self.dragging_handle.take().is_none() {
            return;
        }
        cx.emit(ResizablePanelsEvent::ResizeEnd { sizes: self.legacy_sizes() });
        cx.notify();
    }

    pub(crate) fn handle_handle_key_down(
        &mut self,
        index: usize,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled || index + 1 >= self.layout_states.len() {
            return;
        }

        let step_px = if event.keystroke.modifiers.shift {
            self.model.keyboard_shift_step
        } else {
            self.model.keyboard_step
        };

        let delta = match (self.model.orientation, event.keystroke.key.as_str()) {
            (ResizablePanelsOrientation::Horizontal, "left") => -step_px,
            (ResizablePanelsOrientation::Horizontal, "right") => step_px,
            (ResizablePanelsOrientation::Vertical, "up") => -step_px,
            (ResizablePanelsOrientation::Vertical, "down") => step_px,
            _ => return,
        };

        self.handle_focuses[index].focus(window, cx);
        let delta_px = self.keyboard_delta_px(delta);
        if self.apply_pair_delta(index, delta_px) {
            self.refresh_panel_sizes_px();
            let sizes = self.legacy_sizes();
            cx.emit(ResizablePanelsEvent::SizesChanged { sizes: sizes.clone() });
            cx.emit(ResizablePanelsEvent::ResizeEnd { sizes });
            window.prevent_default();
            cx.stop_propagation();
            cx.notify();
        }
    }

    fn refresh_panel_sizes_px(&mut self) {
        self.panel_sizes_px = solve_layout_px(&self.model.panels, &self.layout_states, self.content_axis_size_px());
    }

    fn render_model(&self) -> ResizablePanelsRenderModel<'_> {
        ResizablePanelsRenderModel {
            id: &self.model.id,
            orientation: self.model.orientation,
            frame_width: self.model.frame_width,
            frame_height: self.model.frame_height,
            show_border: self.model.show_border,
            enabled: self.model.enabled,
            show_handle: self.model.show_handle,
            resize_handle: self.model.resize_handle,
            handle_grip: self.model.handle_grip,
            panel_sizes_px: &self.panel_sizes_px,
            panels: &self.model.panels,
            measured_size: self.measured_size,
        }
    }
}

impl Render for ResizablePanels {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.refresh_panel_sizes_px();
        let appearance =
            self.model.theme.resolve(InteractionState { disabled: !self.model.enabled, ..Default::default() });
        let model = self.render_model();
        let template = self.model.template.clone();

        div().size_full().child(template.render(&model, &appearance, &self.handle_focuses, window, cx))
    }
}
