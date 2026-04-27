use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, Render,
    SharedString, Window, div, prelude::*,
};

use super::{SwitchBuilder, SwitchRenderModel};
use crate::controls::interaction::ControlInteraction;
use crate::controls::switch::model::SwitchModel;
use crate::keyhandling::{ActivateControl, ControlKeyProfile};

#[derive(Clone, Debug)]
pub enum SwitchEvent {
    Change { on: bool },
}

pub struct Switch {
    model: SwitchModel,
    interaction: ControlInteraction,
}

impl EventEmitter<SwitchEvent> for Switch {}

impl Switch {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> SwitchBuilder {
        SwitchBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: SwitchBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        Self { model: builder.model, interaction: ControlInteraction::new(enabled, cx) }
    }

    pub fn on(&self) -> bool {
        self.model.on
    }

    pub fn set_on(&mut self, on: bool, cx: &mut Context<Self>) {
        self.model.on = on;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        cx.notify();
    }

    fn render_model(&self, window: &Window) -> SwitchRenderModel {
        SwitchRenderModel {
            id: self.model.id.clone(),
            label: self.model.label.clone(),
            on: self.model.on,
            enabled: self.model.enabled,
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn activate(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.model.enabled {
            return false;
        }

        self.model.on = !self.model.on;
        cx.emit(SwitchEvent::Change { on: self.model.on });
        cx.notify();
        true
    }

    fn handle_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        self.activate(cx);
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
        self.activate(cx);
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_hover(*hovered) {
            cx.notify();
        }
    }

    fn handle_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_mouse_down(self.model.enabled, window, cx) {
            cx.notify();
        }
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_mouse_up() {
            cx.notify();
        }
    }
}

impl Focusable for Switch {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for Switch {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);

        div()
            .child(
                self.model
                    .template
                    .render(&model, window, cx)
                    .track_focus(self.interaction.focus_handle())
                    .key_context(ControlKeyProfile::Choice.context())
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
