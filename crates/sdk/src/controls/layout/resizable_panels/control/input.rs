use gpui::{Context, DragMoveEvent, FocusOutEvent, KeyDownEvent, MouseButton, MouseDownEvent, MouseUpEvent, Window};

use super::super::model::ResizablePanelsOrientation;
use super::{ResizablePanels, ResizablePanelsEvent, ResizablePanelsHandleDrag};

impl ResizablePanels {
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
        if event.click_count >= 2 && self.apply_double_click_collapse(index, cx) {
            self.dragging_handle = None;
            self.drag_content_axis_px = None;
            window.prevent_default();
            cx.stop_propagation();
            return;
        }

        self.dragging_handle = Some(index);
        self.drag_start_axis_px = self.axis_position(event.position);
        self.drag_start_states = self.layout_states.clone();
        self.drag_content_axis_px = Some(self.content_axis_size_px());
        cx.emit(ResizablePanelsEvent::ResizeStart);
        cx.notify();
    }

    pub(crate) fn handle_handle_hover(&mut self, index: usize, hovered: &bool, cx: &mut Context<Self>) {
        if !self.model.enabled || index >= self.handle_focuses.len() {
            return;
        }

        let next = if *hovered {
            Some(index)
        } else if self.hovered_handle == Some(index) {
            None
        } else {
            return;
        };
        if self.hovered_handle == next {
            return;
        }
        self.hovered_handle = next;
        cx.emit(ResizablePanelsEvent::HandleHoverChanged { handle_index: index, hovered: *hovered });
        cx.notify();
    }

    pub(super) fn emit_handle_focus_changed(&mut self, index: usize, focused: bool, cx: &mut Context<Self>) -> bool {
        let Some(current) = self.emitted_focused_handles.get_mut(index) else {
            return false;
        };
        if *current == focused {
            return false;
        }
        *current = focused;
        cx.emit(ResizablePanelsEvent::HandleFocusChanged { handle_index: index, focused });
        true
    }

    pub(super) fn handle_handle_focus_in(&mut self, index: usize, _window: &mut Window, cx: &mut Context<Self>) {
        if self.emit_handle_focus_changed(index, true, cx) {
            cx.notify();
        }
    }

    pub(super) fn handle_handle_focus_out(
        &mut self,
        index: usize,
        _: FocusOutEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.emit_handle_focus_changed(index, false, cx) {
            cx.notify();
        }
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
        let changed = self.apply_pair_delta(index, delta_px, cx);
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
        self.drag_content_axis_px = None;
        cx.emit(ResizablePanelsEvent::ResizeEnd { sizes_px: self.panel_sizes_px.clone() });
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

        let delta_px = match (self.model.orientation, event.keystroke.key.as_str()) {
            (ResizablePanelsOrientation::Horizontal, "left") => -step_px,
            (ResizablePanelsOrientation::Horizontal, "right") => step_px,
            (ResizablePanelsOrientation::Vertical, "up") => -step_px,
            (ResizablePanelsOrientation::Vertical, "down") => step_px,
            _ => return,
        };

        self.handle_focuses[index].focus(window, cx);
        if self.apply_pair_delta(index, delta_px, cx) {
            self.refresh_panel_sizes_px();
            let sizes_px = self.panel_sizes_px.clone();
            cx.emit(ResizablePanelsEvent::ResizeStart);
            cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: sizes_px.clone() });
            cx.emit(ResizablePanelsEvent::ResizeEnd { sizes_px });
            window.prevent_default();
            cx.stop_propagation();
            cx.notify();
        }
    }
}
