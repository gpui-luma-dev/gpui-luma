use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, Render,
    SharedString, Window, div, prelude::*,
};

use super::{ProtoButtonBuilder, ProtoButtonRenderModel, ProtoButtonTemplateParams};
use crate::controls::interaction::ControlInteraction;
use crate::controls::prototypes::proto_button::model::ProtoButtonModel;
use crate::keyhandling::{ActivateControl, ControlKeyProfile};

#[derive(Clone, Debug)]
pub enum ProtoButtonEvent {
    Click,
}

pub struct ProtoButton {
    model: ProtoButtonModel,
    interaction: ControlInteraction,
}

impl EventEmitter<ProtoButtonEvent> for ProtoButton {}

impl ProtoButton {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ProtoButtonBuilder {
        ProtoButtonBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: ProtoButtonBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        Self { model: builder.model, interaction: ControlInteraction::new(enabled, cx) }
    }

    pub fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.label = label.into();
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        cx.notify();
    }

    pub fn template_params(&self) -> Option<ProtoButtonTemplateParams> {
        self.model.template.read_params()
    }

    pub fn set_template_params(&mut self, params: ProtoButtonTemplateParams, cx: &mut Context<Self>) -> bool {
        let updated = self.model.template.write_params(params);
        if updated {
            cx.notify();
        }
        updated
    }

    fn render_model<'a>(&'a self, window: &Window) -> ProtoButtonRenderModel<'a> {
        ProtoButtonRenderModel {
            id: &self.model.id,
            label: &self.model.label,
            size: self.model.size,
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn activate(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.model.enabled {
            return false;
        }

        cx.emit(ProtoButtonEvent::Click);
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

impl Focusable for ProtoButton {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for ProtoButton {
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
