mod template;
mod theme;

pub use template::{ThemedCheckboxTemplate, default_template as default_checkbox_template};

pub use theme::{CheckboxLook, CheckboxPalette, CheckboxScale, CheckboxTheme, DefaultCheckboxTheme, default_checkbox_theme};

pub use crate::theme::InteractionState as CheckboxState;

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, SharedString, Subscription, Window, div,
};

use crate::animation::{DEFAULT_TRANSITION_DURATION, VisualTransition};
use crate::controls::button_family::ButtonFamilyRole;
use crate::controls::command::button::{Button, ButtonBuilder, ButtonEvent, ButtonRenderModel, ButtonTemplate};
use crate::controls::presenter::{ControlPresenter, HasPresenter};
use crate::controls::icon::SelectionStatusIcons;
use crate::theme::ControlSize;

pub type Checkbox = Entity<CheckboxControl>;

/// Typed payload for [`Button<CheckboxData>`] / [`ButtonTemplate<CheckboxData>`].
///
/// `checked` is the settled semantic value; `progress` is the continuous visual factor
/// (`0.0`..`1.0`) driven by [`VisualTransition`].
#[derive(Clone, Debug)]
pub struct CheckboxData {
    pub checked: bool,
    pub progress: f32,
    pub icons: SelectionStatusIcons,
}

impl PartialEq for CheckboxData {
    fn eq(&self, other: &Self) -> bool {
        self.checked == other.checked && self.progress == other.progress
    }
}

impl CheckboxData {
    pub fn new(checked: bool) -> Self {
        Self { checked, progress: if checked { 1.0 } else { 0.0 }, icons: SelectionStatusIcons::default() }
    }
}

impl Default for CheckboxData {
    fn default() -> Self {
        Self::new(false)
    }
}

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
    animated: bool,
    transition: VisualTransition,
    button: Entity<Button<CheckboxData>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<CheckboxEvent> for CheckboxControl {}

impl CheckboxControl {
    fn from_builder(builder: CheckboxBuilder, cx: &mut Context<Self>) -> Self {
        let checked = builder.button.model.data.checked;
        let animated = builder.animated;
        let duration = if animated {
            DEFAULT_TRANSITION_DURATION
        } else {
            Duration::ZERO
        };
        let progress = if checked { 1.0 } else { 0.0 };
        let transition = VisualTransition::new(progress, duration);

        let mut button_builder = builder.button;
        button_builder.model.data = CheckboxData { checked, progress, icons: button_builder.model.data.icons.clone() };
        let button = button_builder.spawn(cx);
        let subscription = cx.subscribe(&button, Self::handle_button_event);

        Self { checked, animated, transition, button, _subscriptions: vec![subscription] }
    }

    fn push_button_data(&mut self, cx: &mut Context<Self>) {
        let icons = self.button.read(cx).data().icons.clone();
        let data = CheckboxData { checked: self.checked, progress: self.transition.progress(), icons };
        self.button.update(cx, |button, cx| {
            if button.data() == &data {
                return;
            }
            button.set_data(data, cx);
        });
    }

    pub fn set_data(&mut self, checked: bool, cx: &mut Context<Self>) {
        if self.checked == checked {
            return;
        }
        self.checked = checked;
        self.transition.set_target(if checked { 1.0 } else { 0.0 });
        self.push_button_data(cx);
        cx.notify();
    }

    pub fn data(&self) -> &bool {
        &self.checked
    }

    pub fn animated(&self) -> bool {
        self.animated
    }

    pub fn set_animated(&mut self, animated: bool, cx: &mut Context<Self>) {
        if self.animated == animated {
            return;
        }
        self.animated = animated;
        let duration = if animated {
            DEFAULT_TRANSITION_DURATION
        } else {
            Duration::ZERO
        };
        self.transition = VisualTransition::new(self.transition.progress(), duration);
        self.transition.set_target(if self.checked { 1.0 } else { 0.0 });
        self.push_button_data(cx);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_enabled(enabled, cx));
        cx.notify();
    }

    pub fn set_presenter(
        &mut self,
        content: ControlPresenter<ButtonRenderModel<CheckboxData>>,
        cx: &mut Context<Self>,
    ) {
        self.button.update(cx, |button, cx| button.set_presenter(content, cx));
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn ButtonTemplate<CheckboxData>>, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_template(template, cx));
        cx.notify();
    }

    fn handle_button_event(&mut self, _: Entity<Button<CheckboxData>>, event: &ButtonEvent, cx: &mut Context<Self>) {
        match event {
            ButtonEvent::Click => {
                let checked = !self.checked;
                self.checked = checked;
                self.transition.set_target(if checked { 1.0 } else { 0.0 });
                self.push_button_data(cx);
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let was_animating = self.transition.is_animating();
        let _ = self.transition.sync();
        self.push_button_data(cx);
        self.transition.schedule_frame(window, cx);
        if was_animating || self.transition.is_animating() {
            cx.notify();
        }

        div().child(self.button.clone()).into_any_element()
    }
}

impl IntoElement for CheckboxControl {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

/// Builder for [`Checkbox`] controls. Distinct from [`ButtonBuilder<CheckboxData>`] so Shadcn style
/// helpers apply the checkbox template rather than the switch template.
pub struct CheckboxBuilder {
    button: ButtonBuilder<CheckboxData>,
    animated: bool,
    icons: SelectionStatusIcons,
}

impl CheckboxBuilder {
    pub fn with_data(self, checked: bool) -> Self {
        Self { button: self.button.with_data(CheckboxData::new(checked)), ..self }
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self { button: self.button.enabled(enabled), ..self }
    }

    pub fn tab_stop(self, tab_stop: bool) -> Self {
        Self { button: self.button.tab_stop(tab_stop), ..self }
    }

    pub fn size(self, size: ControlSize) -> Self {
        Self { button: self.button.size(size), ..self }
    }

    /// Renders only the checkbox indicator (no label slot). Use in tables and list rows.
    pub fn indicator_only(self) -> Self {
        Self { button: self.button.role(ButtonFamilyRole::Icon), ..self }
    }

    pub fn without_elevation(self) -> Self {
        Self { button: self.button.without_elevation(), ..self }
    }

    pub fn compact(self) -> Self {
        Self { button: self.button.compact(), ..self }
    }

    /// Enables or disables the checked/unchecked visual transition (default `true`).
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn icons(mut self, icons: SelectionStatusIcons) -> Self {
        self.icons = icons.clone();
        self.button.model.data.icons = icons;
        self
    }

    pub fn template(self, template: Arc<dyn ButtonTemplate<CheckboxData>>) -> Self {
        Self { button: self.button.template(template), ..self }
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Checkbox {
        cx.new(|cx| CheckboxControl::from_builder(self, cx))
    }
}

impl HasPresenter<ButtonRenderModel<CheckboxData>> for CheckboxBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<CheckboxData>>) {
        self.button.set_presenter(content);
    }
}

pub fn new(id: impl Into<SharedString>) -> CheckboxBuilder {
    CheckboxBuilder {
        button: ButtonBuilder::new(id).typed(CheckboxData::default()).template(default_checkbox_template()),
        animated: true,
        icons: SelectionStatusIcons::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkbox_builder_animated_option() {
        assert!(new("animated-default").animated);
        assert!(!new("animated-off").animated(false).animated);
    }

    #[test]
    fn checkbox_data_seeds_progress_from_checked() {
        assert_eq!(CheckboxData::new(true).progress, 1.0);
        assert_eq!(CheckboxData::new(false).progress, 0.0);
    }
}
