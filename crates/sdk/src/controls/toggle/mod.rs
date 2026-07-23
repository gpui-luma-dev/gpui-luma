use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, SharedString, Subscription, Window, div,
};
use gpui::prelude::*;

use crate::controls::button_family::{ButtonFamilyRole, button_family_effective_border, default_button_family_theme};
use crate::controls::command::button::{
    Button, ButtonBuilder, ButtonEvent, ButtonRenderModel, ButtonTemplate, DefaultButtonTemplate,
};
use crate::controls::presenter::{ControlPresenter, HasPresenter};
use crate::theme::ControlSize;

pub type Toggle = Entity<ToggleControl>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ToggleEvent {
    Change { selected: bool },
    FocusChanged { focused: bool },
    EnabledChanged { enabled: bool },
    HoverChanged { hovered: bool },
}

pub struct ToggleControl {
    selected: bool,
    button: Entity<Button<bool>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ToggleEvent> for ToggleControl {}

impl ToggleControl {
    fn from_builder(builder: ToggleBuilder, cx: &mut Context<Self>) -> Self {
        let selected = builder.0.model.data;
        let button = builder.0.spawn(cx);
        let subscription = cx.subscribe(&button, Self::handle_button_event);

        Self { selected, button, _subscriptions: vec![subscription] }
    }

    pub fn set_data(&mut self, selected: bool, cx: &mut Context<Self>) {
        self.selected = selected;
        self.button.update(cx, |button, cx| button.set_data(selected, cx));
        cx.notify();
    }

    pub fn data(&self) -> &bool {
        &self.selected
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_enabled(enabled, cx));
        cx.notify();
    }

    pub fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<bool>>, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_presenter(content, cx));
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn ButtonTemplate<bool>>, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_template(template, cx));
        cx.notify();
    }

    fn handle_button_event(&mut self, _: Entity<Button<bool>>, event: &ButtonEvent, cx: &mut Context<Self>) {
        match event {
            ButtonEvent::Click => {
                let selected = !self.selected;
                self.selected = selected;
                self.button.update(cx, |button, cx| button.set_data(selected, cx));
                cx.emit(ToggleEvent::Change { selected });
                cx.notify();
            }
            ButtonEvent::FocusChanged { focused } => cx.emit(ToggleEvent::FocusChanged { focused: *focused }),
            ButtonEvent::EnabledChanged { enabled } => cx.emit(ToggleEvent::EnabledChanged { enabled: *enabled }),
            ButtonEvent::HoverChanged { hovered } => cx.emit(ToggleEvent::HoverChanged { hovered: *hovered }),
        }
    }
}

impl Focusable for ToggleControl {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.button.read(cx).focus_handle(cx)
    }
}

impl Render for ToggleControl {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(self.button.clone()).into_any_element()
    }
}

impl IntoElement for ToggleControl {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

pub struct ToggleBuilder(ButtonBuilder<bool>);

impl ToggleBuilder {
    pub fn with_data(self, data: bool) -> Self {
        Self(self.0.with_data(data))
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self(self.0.enabled(enabled))
    }

    pub fn tab_stop(self, tab_stop: bool) -> Self {
        Self(self.0.tab_stop(tab_stop))
    }

    pub fn size(self, size: ControlSize) -> Self {
        Self(self.0.size(size))
    }

    pub fn role(self, role: ButtonFamilyRole) -> Self {
        Self(self.0.role(role))
    }

    pub fn round(self, round: bool) -> Self {
        Self(self.0.round(round))
    }

    pub fn without_elevation(self) -> Self {
        Self(self.0.without_elevation())
    }

    pub fn without_adorners(self) -> Self {
        Self(self.0.without_adorners())
    }

    pub fn compact(self) -> Self {
        Self(self.0.compact())
    }

    pub fn template(self, template: Arc<dyn ButtonTemplate<bool>>) -> Self {
        Self(self.0.template(template))
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Toggle {
        cx.new(|cx| ToggleControl::from_builder(self, cx))
    }
}

impl HasPresenter<ButtonRenderModel<bool>> for ToggleBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<bool>>) {
        self.0.set_presenter(content);
    }
}

fn button_builder(id: impl Into<SharedString>) -> ButtonBuilder<bool> {
    ButtonBuilder::new(id).typed(false).template(default_toggle_template())
}

pub fn new(id: impl Into<SharedString>) -> ToggleBuilder {
    ToggleBuilder(button_builder(id))
}

pub fn default_toggle_template() -> Arc<dyn ButtonTemplate<bool>> {
    let button_family_theme = default_button_family_theme();
    Arc::new(DefaultButtonTemplate::new(button_family_theme.clone()).with_modifier(move |element, model| {
        if model.look.is_some() {
            return element;
        }

        let palette =
            button_family_theme.resolve(ButtonFamilyRole::Toggle { selected: model.data }, model.size, model.state);
        element
            .bg(palette.background)
            .text_color(palette.foreground)
            .border_color(button_family_effective_border(palette.border))
    }))
}
