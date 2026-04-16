use gpui::{
    App, ClickEvent, Context, EventEmitter, FocusHandle, Focusable, IntoElement, MouseButton,
    MouseDownEvent, MouseUpEvent, Render, SharedString, Window, div, prelude::*,
};

use super::{IconButtonBuilder, IconButtonIcon, IconButtonRenderModel, IconButtonState};
use crate::controls::icon_button::model::IconButtonModel;

#[derive(Clone, Debug)]
pub enum IconButtonEvent {
    Click,
}

pub struct IconButton {
    model: IconButtonModel,
    state: IconButtonState,
    focus_handle: FocusHandle,
}

impl EventEmitter<IconButtonEvent> for IconButton {}

impl IconButton {
    pub fn new(id: impl Into<SharedString>) -> IconButtonBuilder {
        IconButtonBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: IconButtonBuilder, cx: &mut Context<Self>) -> Self {
        Self {
            model: builder.model,
            state: IconButtonState::default(),
            focus_handle: cx.focus_handle().tab_stop(true),
        }
    }

    pub fn set_icon(&mut self, icon: impl Into<IconButtonIcon>, cx: &mut Context<Self>) {
        self.model.icon = icon.into();
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.state.disabled = !enabled;
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> IconButtonRenderModel<'a> {
        IconButtonRenderModel {
            id: &self.model.id,
            icon: &self.model.icon,
            kind: self.model.kind,
            size: self.model.size,
            state: IconButtonState {
                focused: self.focus_handle.is_focused(window),
                disabled: !self.model.enabled,
                ..self.state
            },
        }
    }

    fn handle_click(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled {
            cx.emit(IconButtonEvent::Click);
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

impl Focusable for IconButton {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for IconButton {
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
