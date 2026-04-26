use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, Render,
    SharedString, Window, div, prelude::*,
};

use super::{ButtonBuilder, ButtonRenderModel};
use crate::controls::command::button::model::ButtonModel;
use crate::controls::command::{CommandCore, CommandEvent};
use crate::keyhandling::{ActivateControl, ControlKeyProfile};

pub type ButtonEvent = CommandEvent;

pub struct Button {
    model: ButtonModel,
    command: CommandCore,
}

impl EventEmitter<ButtonEvent> for Button {}

impl Button {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ButtonBuilder {
        ButtonBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: ButtonBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        Self { model: builder.model, command: CommandCore::new(enabled, cx) }
    }

    pub fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.label = label.into();
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.command.set_enabled(enabled);
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> ButtonRenderModel<'a> {
        ButtonRenderModel {
            id: &self.model.id,
            label: &self.model.label,
            kind: self.model.kind,
            size: self.model.size,
            state: self.command.render_state(self.model.enabled, window),
        }
    }

    fn handle_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.command.handle_click(self.model.enabled, event, cx);
    }

    fn handle_activate_control(&mut self, event: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
        self.command.handle_activate_control(self.model.enabled, event, cx);
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.command.handle_hover(*hovered) {
            cx.notify();
        }
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.command.handle_mouse_down(self.model.enabled, event, window, cx) {
            cx.notify();
        }
    }

    fn handle_mouse_up(&mut self, event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.command.handle_mouse_up(event) {
            cx.notify();
        }
    }
}

impl Focusable for Button {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.command.focus_handle().clone()
    }
}

impl Render for Button {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);

        div()
            .child(
                self.model
                    .template
                    .render(&model, window, cx)
                    .track_focus(self.command.focus_handle())
                    .key_context(ControlKeyProfile::Command.context())
                    .on_action(cx.listener(Self::handle_activate_control))
                    .on_hover(cx.listener(Self::handle_hover))
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_mouse_down))
                    .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_mouse_up))
                    .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_mouse_up))
                    .on_click(cx.listener(Self::handle_click)),
            )
            .into_any_element()
    }
}
