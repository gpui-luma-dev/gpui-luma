use gpui::{
    Context, DragMoveEvent, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent,
    MouseUpEvent, ParentElement, Pixels, Render, SharedString, Window, div, prelude::*, px,
};

use super::{
    math::{apply_pair_delta, content_axis_size, normalize_sizes, PanelSizeBounds},
    model::{ResizablePanelsBuilder, ResizablePanelsModel, ResizablePanelsOrientation, ResizablePanelsRenderModel},
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
    SizesChanged { sizes: Vec<f32> },
    ResizeEnd { sizes: Vec<f32> },
}

pub struct ResizablePanels {
    model: ResizablePanelsModel,
    sizes: Vec<f32>,
    handle_focuses: Vec<FocusHandle>,
    dragging_handle: Option<usize>,
    drag_start_axis_px: f32,
    drag_start_sizes: Vec<f32>,
    measured_size: Option<gpui::Size<Pixels>>,
}

impl EventEmitter<ResizablePanelsEvent> for ResizablePanels {}

impl ResizablePanels {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ResizablePanelsBuilder {
        ResizablePanelsBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: ResizablePanelsBuilder, cx: &mut Context<Self>) -> Self {
        let mut sizes: Vec<f32> = builder.model.panels.iter().map(|panel| panel.default_size).collect();
        normalize_sizes(&mut sizes);
        if sizes.len() != builder.model.panels.len() {
            sizes = vec![100.0];
        }
        let handle_focuses = (0..builder.model.panels.len().saturating_sub(1)).map(|_| cx.focus_handle()).collect();

        Self {
            model: builder.model,
            sizes,
            handle_focuses,
            dragging_handle: None,
            drag_start_axis_px: 0.0,
            drag_start_sizes: Vec::new(),
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

    pub fn set_handle_size(&mut self, handle_size: Pixels, cx: &mut Context<Self>) {
        let clamped = handle_size.max(px(1.0));
        if self.model.handle_size == clamped {
            return;
        }
        self.model.handle_size = clamped;
        cx.notify();
    }

    pub fn set_frame_size(&mut self, width: Pixels, height: Pixels, cx: &mut Context<Self>) {
        if self.model.frame_width == Some(width) && self.model.frame_height == Some(height) {
            return;
        }
        self.model.frame_width = Some(width);
        self.model.frame_height = Some(height);
        cx.notify();
    }

    pub fn set_sizes(&mut self, mut sizes: Vec<f32>, cx: &mut Context<Self>) {
        if sizes.len() != self.model.panels.len() || sizes.is_empty() {
            return;
        }

        normalize_sizes(&mut sizes);
        if self.sizes.iter().zip(sizes.iter()).all(|(current, next)| (current - next).abs() < f32::EPSILON) {
            return;
        }

        self.sizes = sizes.clone();
        cx.emit(ResizablePanelsEvent::SizesChanged { sizes });
        cx.notify();
    }

    pub fn sizes(&self) -> Vec<f32> {
        self.sizes.clone()
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

    fn content_axis_size_px(&self) -> f32 {
        let panel_count = self.sizes.len();
        let main_px = match (self.model.frame_width, self.model.frame_height, self.model.orientation) {
            (Some(width), _, ResizablePanelsOrientation::Horizontal) => width.as_f32(),
            (_, Some(height), ResizablePanelsOrientation::Vertical) => height.as_f32(),
            (None, _, ResizablePanelsOrientation::Horizontal) => {
                self.measured_size.map(|s| s.width.as_f32()).unwrap_or(1.0)
            }
            (_, None, ResizablePanelsOrientation::Vertical) => {
                self.measured_size.map(|s| s.height.as_f32()).unwrap_or(1.0)
            }
        };
        let handle_size = self.model.handle_size.as_f32().max(1.0);
        content_axis_size(main_px, handle_size, panel_count)
    }

    fn apply_pair_delta(&mut self, index: usize, delta_percent: f32) -> bool {
        if index + 1 >= self.sizes.len() {
            return false;
        }

        let left = PanelSizeBounds {
            min_size: self.model.panels[index].min_size,
            max_size: self.model.panels[index].max_size,
        };
        let right = PanelSizeBounds {
            min_size: self.model.panels[index + 1].min_size,
            max_size: self.model.panels[index + 1].max_size,
        };
        apply_pair_delta(&mut self.sizes, index, delta_percent, left, right)
    }

    fn emit_sizes_changed_if_needed(&mut self, changed: bool, cx: &mut Context<Self>) {
        if !changed {
            return;
        }
        cx.emit(ResizablePanelsEvent::SizesChanged { sizes: self.sizes.clone() });
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
        self.drag_start_sizes = self.sizes.clone();
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
        if index != drag.handle_index || index + 1 >= self.sizes.len() {
            return;
        }

        self.sizes.clone_from(&self.drag_start_sizes);
        let delta_px = self.axis_position(event.event.position) - self.drag_start_axis_px;
        let delta_percent = (delta_px / self.content_axis_size_px()) * 100.0;
        let changed = self.apply_pair_delta(index, delta_percent);
        self.emit_sizes_changed_if_needed(changed, cx);
    }

    pub(crate) fn finish_drag(&mut self, event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || event.button != MouseButton::Left {
            return;
        }
        if self.dragging_handle.take().is_none() {
            return;
        }
        cx.emit(ResizablePanelsEvent::ResizeEnd { sizes: self.sizes.clone() });
        cx.notify();
    }

    pub(crate) fn handle_handle_key_down(
        &mut self,
        index: usize,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled || index + 1 >= self.sizes.len() {
            return;
        }

        let step = if event.keystroke.modifiers.shift {
            self.model.keyboard_shift_step
        } else {
            self.model.keyboard_step
        };

        let delta = match (self.model.orientation, event.keystroke.key.as_str()) {
            (ResizablePanelsOrientation::Horizontal, "left") => -step,
            (ResizablePanelsOrientation::Horizontal, "right") => step,
            (ResizablePanelsOrientation::Vertical, "up") => -step,
            (ResizablePanelsOrientation::Vertical, "down") => step,
            _ => return,
        };

        self.handle_focuses[index].focus(window, cx);
        if self.apply_pair_delta(index, delta) {
            cx.emit(ResizablePanelsEvent::SizesChanged { sizes: self.sizes.clone() });
            cx.emit(ResizablePanelsEvent::ResizeEnd { sizes: self.sizes.clone() });
            window.prevent_default();
            cx.stop_propagation();
            cx.notify();
        }
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
            handle_size: self.model.handle_size,
            handle_grip: self.model.handle_grip,
            sizes: &self.sizes,
            panels: &self.model.panels,
            measured_size: self.measured_size,
        }
    }
}

impl Render for ResizablePanels {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let appearance =
            self.model.theme.resolve(InteractionState { disabled: !self.model.enabled, ..Default::default() });
        let model = self.render_model();
        let template = self.model.template.clone();

        div().size_full().child(template.render(&model, &appearance, &self.handle_focuses, window, cx))
    }
}
