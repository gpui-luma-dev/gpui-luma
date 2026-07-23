mod template;
mod theme;

pub use template::{ThemedCheckboxTemplate, default_template as default_checkbox_template};

pub use theme::{CheckboxLook, CheckboxPalette, CheckboxScale, CheckboxTheme, DefaultCheckboxTheme, default_checkbox_theme};

pub use crate::theme::InteractionState as CheckboxState;

use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, SharedString, Subscription, Window, div,
};

use crate::controls::button_family::ButtonFamilyRole;
use crate::controls::command::button::{Button, ButtonBuilder, ButtonEvent, ButtonRenderModel, ButtonTemplate};
use crate::controls::presenter::{ControlPresenter, HasPresenter};
use crate::theme::ControlSize;

pub type Checkbox = Entity<CheckboxControl>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CheckboxEvent {
    Change { checked: bool },
    FocusChanged { focused: bool },
    EnabledChanged { enabled: bool },
    HoverChanged { hovered: bool },
}

pub struct CheckboxControl {
    checked: bool,
    button: Entity<Button<bool>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<CheckboxEvent> for CheckboxControl {}

impl CheckboxControl {
    fn from_builder(builder: CheckboxBuilder, cx: &mut Context<Self>) -> Self {
        let checked = builder.0.model.data;
        let button = builder.0.spawn(cx);
        let subscription = cx.subscribe(&button, Self::handle_button_event);

        Self { checked, button, _subscriptions: vec![subscription] }
    }

    pub fn set_data(&mut self, checked: bool, cx: &mut Context<Self>) {
        self.checked = checked;
        self.button.update(cx, |button, cx| button.set_data(checked, cx));
        cx.notify();
    }

    pub fn data(&self) -> &bool {
        &self.checked
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
                let checked = !self.checked;
                self.checked = checked;
                self.button.update(cx, |button, cx| button.set_data(checked, cx));
                cx.emit(CheckboxEvent::Change { checked });
                cx.notify();
            }
            ButtonEvent::FocusChanged { focused } => cx.emit(CheckboxEvent::FocusChanged { focused: *focused }),
            ButtonEvent::EnabledChanged { enabled } => cx.emit(CheckboxEvent::EnabledChanged { enabled: *enabled }),
            ButtonEvent::HoverChanged { hovered } => cx.emit(CheckboxEvent::HoverChanged { hovered: *hovered }),
        }
    }
}

impl Focusable for CheckboxControl {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.button.read(cx).focus_handle(cx)
    }
}

impl Render for CheckboxControl {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(self.button.clone()).into_any_element()
    }
}

impl IntoElement for CheckboxControl {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

/// Builder for [`Checkbox`] controls. Distinct from [`ButtonBuilder<bool>`] so Shadcn style
/// helpers apply the checkbox template rather than the switch template.
pub struct CheckboxBuilder(ButtonBuilder<bool>);

impl CheckboxBuilder {
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

    /// Renders only the checkbox indicator (no label slot). Use in tables and list rows.
    pub fn indicator_only(self) -> Self {
        Self(self.0.role(ButtonFamilyRole::Icon))
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

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Checkbox {
        cx.new(|cx| CheckboxControl::from_builder(self, cx))
    }
}

impl HasPresenter<ButtonRenderModel<bool>> for CheckboxBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<bool>>) {
        self.0.set_presenter(content);
    }
}

pub fn new(id: impl Into<SharedString>) -> CheckboxBuilder {
    CheckboxBuilder(ButtonBuilder::new(id).typed(false).template(default_checkbox_template()))
}
