use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseButton, MouseDownEvent,
    MouseUpEvent, Render, SharedString, Window, div, prelude::*, px,
};

pub use crate::controls::presenter::{ControlPresenter, HasPresenter};

use super::{ButtonBuilder, ButtonRenderModel};
pub use crate::controls::command::{CommandCore, CommandEvent as ButtonEvent};
use crate::keyhandling::{ActivateControl, ControlKeyProfile};
use crate::controls::command::button::model::ControlIcon;
use crate::controls::button_family::ButtonFamilyRole;

pub struct Button<D = ()> {
    model: super::model::ButtonModel<D>,
    command: CommandCore,
}

impl<D: 'static> EventEmitter<ButtonEvent> for Button<D> {}

impl Button<()> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<()> {
        ButtonBuilder::new(id)
    }

    pub fn icon(id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()> {
        let icon = icon.into();
        ButtonBuilder::new(id).role(ButtonFamilyRole::Icon).round(true).content(move |_, _| match &icon {
            ControlIcon::Lucide(lucide) => div()
                .font_family("lucide")
                .text_size(px(16.0))
                .child(char::from(*lucide).to_string())
                .into_any_element(),
            ControlIcon::SvgPath(path) => gpui::svg().size(px(16.0)).path(path.clone()).into_any_element(),
        })
    }
}

impl<D: Clone + 'static> Button<D> {
    pub(crate) fn from_builder(builder: ButtonBuilder<D>, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;
        let tab_stop = builder.model.tab_stop;
        Self { model: builder.model, command: CommandCore::new_with_tab_stop(enabled, tab_stop, cx) }
    }

    pub fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<D>>, cx: &mut Context<Self>) {
        self.model.content = content;
        cx.notify();
    }

    pub fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>)
    where
        D: Default + 'static,
    {
        let label = label.into();
        self.model.content = Arc::new(move |_, _| div().child(label.clone()).into_any_element());
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn super::template::ButtonTemplate<D>>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_data(&mut self, data: D, cx: &mut Context<Self>) {
        self.model.data = data;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.command.set_enabled(enabled);
        cx.notify();
    }

    pub fn data(&self) -> &D {
        &self.model.data
    }

    pub fn render_model(&self, window: &Window) -> ButtonRenderModel<D> {
        ButtonRenderModel {
            id: self.model.id.clone(),
            data: self.model.data.clone(),
            content: self.model.content.clone(),
            kind: self.model.kind,
            role: self.model.role,
            size: self.model.size,
            state: self.command.render_state(self.model.enabled, window),
            round: self.model.round,
            radius_override: std::cell::Cell::new(None),
            appearance: self.model.appearance.clone(),
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

impl<D: 'static> Focusable for Button<D> {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.command.focus_handle().clone()
    }
}

impl<D: Clone + 'static> Render for Button<D> {
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

impl<D: Clone + 'static> IntoElement for Button<D> {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

impl<D: Clone + 'static> From<Button<D>> for AnyElement {
    fn from(button: Button<D>) -> Self {
        button.into_element()
    }
}
