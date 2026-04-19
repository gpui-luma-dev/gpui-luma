use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, Render,
    SharedString, Window, div, prelude::*,
};

use super::{ToggleButtonBuilder, ToggleButtonRenderModel};
use crate::controls::interaction::ControlInteraction;
use crate::controls::toggle_button::model::ToggleButtonModel;
use crate::keyhandling::{ActivateControl, ControlKeyProfile};

#[derive(Clone, Debug)]
pub enum ToggleButtonEvent {
    Change { selected: bool },
}

pub struct ToggleButton {
    model: ToggleButtonModel,
    interaction: ControlInteraction,
}

impl EventEmitter<ToggleButtonEvent> for ToggleButton {}

impl ToggleButton {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ToggleButtonBuilder {
        ToggleButtonBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: ToggleButtonBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        Self { model: builder.model, interaction: ControlInteraction::new(enabled, cx) }
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
        self.interaction.set_enabled(enabled);
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
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn activate(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.model.enabled {
            return false;
        }

        self.model.selected = !self.model.selected;
        cx.emit(ToggleButtonEvent::Change { selected: self.model.selected });
        cx.notify();
        true
    }

    fn handle_click(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
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

impl Focusable for ToggleButton {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
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
                    .track_focus(self.interaction.focus_handle())
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
