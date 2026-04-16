use gpui::{
    App, ClickEvent, Context, EventEmitter, FocusHandle, Focusable, IntoElement, MouseButton,
    MouseDownEvent, MouseUpEvent, Render, SharedString, Window, div, prelude::*,
};

use super::{ToggleButtonBuilder, ToggleButtonRenderModel, ToggleButtonState};
use crate::controls::toggle_button::model::ToggleButtonModel;

#[derive(Clone, Debug)]
pub enum ToggleButtonEvent {
    Change { selected: bool },
}

pub struct ToggleButton {
    model: ToggleButtonModel,
    state: ToggleButtonState,
    focus_handle: FocusHandle,
}

impl EventEmitter<ToggleButtonEvent> for ToggleButton {}

impl ToggleButton {
    pub fn new(id: impl Into<SharedString>) -> ToggleButtonBuilder {
        ToggleButtonBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: ToggleButtonBuilder, cx: &mut Context<Self>) -> Self {
        Self {
            model: builder.model,
            state: ToggleButtonState::default(),
            focus_handle: cx.focus_handle().tab_stop(true),
        }
    }

    pub fn selected(&self) -> bool {
        self.model.selected
    }

    pub fn set_selected(&mut self, selected: bool, cx: &mut Context<Self>) {
        self.model.selected = selected;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.state.disabled = !enabled;
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> ToggleButtonRenderModel<'a> {
        ToggleButtonRenderModel {
            id: &self.model.id,
            label: &self.model.label,
            kind: self.model.kind,
            size: self.model.size,
            enabled: self.model.enabled,
            selected: self.model.selected,
            state: ToggleButtonState {
                focused: self.focus_handle.is_focused(window),
                disabled: !self.model.enabled,
                ..self.state
            },
        }
    }

    fn handle_click(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled {
            self.model.selected = !self.model.selected;
            cx.emit(ToggleButtonEvent::Change {
                selected: self.model.selected,
            });
            cx.notify();
        }
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        self.state.hovered = *hovered;
        if !hovered {
            self.state.pressed = false;
        }
        cx.notify();
    }

    fn handle_mouse_down(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.model.enabled {
            self.state.pressed = true;
            self.focus_handle.focus(window, cx);
            cx.notify();
        }
    }

    fn handle_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.state.pressed {
            self.state.pressed = false;
            cx.notify();
        }
    }
}

impl Focusable for ToggleButton {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ToggleButton {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);

        div()
            .child(
                self.model
                    .template
                    .render(&model, window, cx)
                    .track_focus(&self.focus_handle)
                    .on_hover(cx.listener(Self::handle_hover))
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_mouse_down))
                    .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_mouse_up))
                    .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_mouse_up))
                    .on_click(cx.listener(Self::handle_click)),
            )
            .into_any_element()
    }
}
