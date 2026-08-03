mod template;
mod theme;

pub use template::{ThemedRadioButtonTemplate, default_template as default_radio_button_template};
pub use theme::{
    DefaultRadioButtonTheme, RadioButtonLook, RadioButtonPalette, RadioButtonTheme, RadioScale,
    default_radio_button_theme,
};

pub use crate::theme::InteractionState as RadioButtonState;

use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, SharedString, Subscription, Window, div,
};

use crate::controls::button_family::{ButtonFamilyRole, ButtonSize};
use crate::controls::command::button::{Button, ButtonBuilder, ButtonEvent, ButtonRenderModel, ButtonTemplate};
use crate::controls::presenter::{ControlPresenter, HasPresenter};

pub type RadioButton = Entity<RadioButtonControl>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RadioButtonEvent {
    Change { selected: bool },
    FocusChanged { focused: bool },
    EnabledChanged { enabled: bool },
    HoverChanged { hovered: bool },
}

pub struct RadioButtonControl {
    selected: bool,
    button: Entity<Button<bool>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<RadioButtonEvent> for RadioButtonControl {}

impl RadioButtonControl {
    fn from_builder(builder: RadioButtonBuilder, cx: &mut Context<Self>) -> Self {
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
                cx.emit(RadioButtonEvent::Change { selected });
                cx.notify();
            }
            ButtonEvent::FocusChanged { focused } => cx.emit(RadioButtonEvent::FocusChanged { focused: *focused }),
            ButtonEvent::EnabledChanged { enabled } => cx.emit(RadioButtonEvent::EnabledChanged { enabled: *enabled }),
            ButtonEvent::HoverChanged { hovered } => cx.emit(RadioButtonEvent::HoverChanged { hovered: *hovered }),
        }
    }
}

impl Focusable for RadioButtonControl {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.button.read(cx).focus_handle(cx)
    }
}

impl Render for RadioButtonControl {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(self.button.clone()).into_any_element()
    }
}

impl IntoElement for RadioButtonControl {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

pub struct RadioButtonBuilder(ButtonBuilder<bool>);

impl RadioButtonBuilder {
    pub fn with_data(self, data: bool) -> Self {
        Self(self.0.with_data(data))
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self(self.0.enabled(enabled))
    }

    pub fn tab_stop(self, tab_stop: bool) -> Self {
        Self(self.0.tab_stop(tab_stop))
    }

    pub fn size(self, size: ButtonSize) -> Self {
        Self(self.0.size(size))
    }

    pub fn role(self, role: ButtonFamilyRole) -> Self {
        Self(self.0.role(role))
    }

    pub fn without_elevation(self) -> Self {
        Self(self.0.without_elevation())
    }

    pub fn compact(self) -> Self {
        Self(self.0.compact())
    }

    pub fn template(self, template: Arc<dyn ButtonTemplate<bool>>) -> Self {
        Self(self.0.template(template))
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> RadioButton {
        cx.new(|cx| RadioButtonControl::from_builder(self, cx))
    }
}

impl HasPresenter<ButtonRenderModel<bool>> for RadioButtonBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<bool>>) {
        self.0.set_presenter(content);
    }
}

pub fn new(id: impl Into<SharedString>) -> RadioButtonBuilder {
    RadioButtonBuilder(ButtonBuilder::new(id).typed(false).template(default_radio_button_template()))
}
