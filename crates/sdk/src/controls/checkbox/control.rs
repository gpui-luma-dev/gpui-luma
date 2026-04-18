use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseButton, MouseDownEvent,
    MouseUpEvent, Render, SharedString, Window, div, prelude::*,
};

use super::{CheckboxBuilder, CheckboxRenderModel};
use crate::controls::checkbox::model::CheckboxModel;
use crate::controls::interaction::ControlInteraction;

#[derive(Clone, Debug)]
pub enum CheckboxEvent {
    Change { checked: bool },
}

pub struct Checkbox {
    model: CheckboxModel,
    interaction: ControlInteraction,
}

impl EventEmitter<CheckboxEvent> for Checkbox {}

impl Checkbox {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> CheckboxBuilder {
        CheckboxBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: CheckboxBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        Self {
            model: builder.model,
            interaction: ControlInteraction::new(enabled, cx),
        }
    }

    pub fn checked(&self) -> bool {
        self.model.checked
    }

    pub fn set_checked(&mut self, checked: bool, cx: &mut Context<Self>) {
        self.model.checked = checked;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> CheckboxRenderModel<'a> {
        CheckboxRenderModel {
            id: &self.model.id,
            label: &self.model.label,
            checked: self.model.checked,
            enabled: self.model.enabled,
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn handle_click(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled {
            self.model.checked = !self.model.checked;
            cx.emit(CheckboxEvent::Change {
                checked: self.model.checked,
            });
            cx.notify();
        }
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_hover(*hovered) {
            cx.notify();
        }
    }

    fn handle_mouse_down(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self
            .interaction
            .handle_mouse_down(self.model.enabled, window, cx)
        {
            cx.notify();
        }
    }

    fn handle_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.interaction.handle_mouse_up() {
            cx.notify();
        }
    }
}

impl Focusable for Checkbox {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for Checkbox {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);

        div()
            .child(
                self.model
                    .template
                    .render(&model, window, cx)
                    .track_focus(self.interaction.focus_handle())
                    .on_hover(cx.listener(Self::handle_hover))
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_mouse_down))
                    .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_mouse_up))
                    .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_mouse_up))
                    .on_click(cx.listener(Self::handle_click)),
            )
            .into_any_element()
    }
}
