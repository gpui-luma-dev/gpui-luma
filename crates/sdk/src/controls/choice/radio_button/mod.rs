mod template;
mod theme;

pub use template::{ThemedRadioButtonTemplate, default_template as default_radio_button_template};
pub use theme::{
    DefaultRadioButtonTheme, RadioButtonLook, RadioButtonPalette, RadioButtonTheme, RadioScale,
    default_radio_button_theme,
};

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, SharedString, Subscription, Window, div,
};

use crate::motion::{DEFAULT_TRANSITION_DURATION, VisualTransition};
use crate::controls::button_family::{ButtonFamilyRole, ButtonSize};
use crate::controls::button::{Button, ButtonBuilder, ButtonContentContext, ButtonEvent, ButtonTemplate};
use crate::infra::presenter::{ControlPresenter, HasPresenter};

pub type RadioButton = Entity<RadioButtonControl>;

/// Typed payload for [`Button<RadioButtonData>`] / [`ButtonTemplate<RadioButtonData>`].
///
/// `selected` is the settled semantic value; `progress` is the continuous visual factor
/// (`0.0`..`1.0`) driven by [`VisualTransition`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadioButtonData {
    pub selected: bool,
    pub progress: f32,
}

impl RadioButtonData {
    pub fn new(selected: bool) -> Self {
        Self { selected, progress: if selected { 1.0 } else { 0.0 } }
    }
}

impl Default for RadioButtonData {
    fn default() -> Self {
        Self::new(false)
    }
}

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
    animated: bool,
    transition: VisualTransition,
    button: Entity<Button<RadioButtonData>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<RadioButtonEvent> for RadioButtonControl {}

impl RadioButtonControl {
    fn from_builder(builder: RadioButtonBuilder, cx: &mut Context<Self>) -> Self {
        let selected = builder.button.model.data.selected;
        let animated = builder.animated;
        let duration = if animated {
            DEFAULT_TRANSITION_DURATION
        } else {
            Duration::ZERO
        };
        let progress = if selected { 1.0 } else { 0.0 };
        let transition = VisualTransition::new(progress, duration);

        let mut button_builder = builder.button;
        button_builder.model.data = RadioButtonData { selected, progress };
        let button = button_builder.spawn(cx);
        let subscription = cx.subscribe(&button, Self::handle_button_event);

        Self { selected, animated, transition, button, _subscriptions: vec![subscription] }
    }

    fn push_button_data(&mut self, cx: &mut Context<Self>) {
        let data = RadioButtonData { selected: self.selected, progress: self.transition.progress() };
        self.button.update(cx, |button, cx| {
            if button.data() == &data {
                return;
            }
            button.set_data(data, cx);
        });
    }

    pub fn set_data(&mut self, selected: bool, cx: &mut Context<Self>) {
        if self.selected == selected {
            return;
        }
        self.selected = selected;
        self.transition.set_target(if selected { 1.0 } else { 0.0 });
        self.push_button_data(cx);
        cx.notify();
    }

    pub fn data(&self) -> &bool {
        &self.selected
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
        self.transition.set_target(if self.selected { 1.0 } else { 0.0 });
        self.push_button_data(cx);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_enabled(enabled, cx));
        cx.notify();
    }

    pub fn set_presenter(
        &mut self,
        content: ControlPresenter<ButtonContentContext<RadioButtonData>>,
        cx: &mut Context<Self>,
    ) {
        self.button.update(cx, |button, cx| button.set_presenter(content, cx));
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn ButtonTemplate<RadioButtonData>>, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_template(template, cx));
        cx.notify();
    }

    fn handle_button_event(&mut self, _: Entity<Button<RadioButtonData>>, event: &ButtonEvent, cx: &mut Context<Self>) {
        match event {
            ButtonEvent::Click => {
                let selected = !self.selected;
                self.selected = selected;
                self.transition.set_target(if selected { 1.0 } else { 0.0 });
                self.push_button_data(cx);
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

impl IntoElement for RadioButtonControl {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

pub struct RadioButtonBuilder {
    button: ButtonBuilder<RadioButtonData>,
    animated: bool,
}

impl RadioButtonBuilder {
    pub fn with_data(self, selected: bool) -> Self {
        Self { button: self.button.with_data(RadioButtonData::new(selected)), ..self }
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self { button: self.button.enabled(enabled), ..self }
    }

    pub fn tab_stop(self, tab_stop: bool) -> Self {
        Self { button: self.button.tab_stop(tab_stop), ..self }
    }

    pub fn size(self, size: ButtonSize) -> Self {
        Self { button: self.button.size(size), ..self }
    }

    pub fn role(self, role: ButtonFamilyRole) -> Self {
        Self { button: self.button.role(role), ..self }
    }

    pub fn without_elevation(self) -> Self {
        Self { button: self.button.without_elevation(), ..self }
    }

    pub fn compact(self) -> Self {
        Self { button: self.button.compact(), ..self }
    }

    /// Enables or disables the selected/unselected visual transition (default `true`).
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn template(self, template: Arc<dyn ButtonTemplate<RadioButtonData>>) -> Self {
        Self { button: self.button.template(template), ..self }
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> RadioButton {
        cx.new(|cx| RadioButtonControl::from_builder(self, cx))
    }
}

impl HasPresenter<ButtonContentContext<RadioButtonData>> for RadioButtonBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<RadioButtonData>>) {
        self.button.set_presenter(content);
    }
}

pub fn new(id: impl Into<SharedString>) -> RadioButtonBuilder {
    RadioButtonBuilder {
        button: ButtonBuilder::new(id).typed(RadioButtonData::default()).template(default_radio_button_template()),
        animated: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_button_builder_animated_option() {
        assert!(new("animated-default").animated);
        assert!(!new("animated-off").animated(false).animated);
    }

    #[test]
    fn radio_button_data_seeds_progress_from_selected() {
        assert_eq!(RadioButtonData::new(true).progress, 1.0);
        assert_eq!(RadioButtonData::new(false).progress, 0.0);
    }
}
