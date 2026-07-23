mod template;
mod theme;

pub use template::{ThemedSwitchTemplate, default_template as default_switch_template};
pub use theme::{DefaultSwitchTheme, SwitchLook, SwitchPalette, SwitchScale, SwitchTheme, default_switch_theme};

pub use crate::theme::InteractionState as SwitchState;

use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, SharedString, Subscription, Window, div,
};

use crate::controls::button_family::ButtonSize;
use crate::controls::command::button::{Button, ButtonBuilder, ButtonEvent, ButtonRenderModel, ButtonTemplate};
use crate::controls::presenter::{ControlPresenter, HasPresenter};

pub type Switch = Entity<SwitchControl>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SwitchEvent {
    Change { on: bool },
    FocusChanged { focused: bool },
    EnabledChanged { enabled: bool },
    HoverChanged { hovered: bool },
}

pub struct SwitchControl {
    on: bool,
    button: Entity<Button<bool>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<SwitchEvent> for SwitchControl {}

impl SwitchControl {
    fn from_builder(builder: SwitchBuilder, cx: &mut Context<Self>) -> Self {
        let on = builder.0.model.data;
        let button = builder.0.spawn(cx);
        let subscription = cx.subscribe(&button, Self::handle_button_event);

        Self { on, button, _subscriptions: vec![subscription] }
    }

    pub fn set_data(&mut self, on: bool, cx: &mut Context<Self>) {
        self.on = on;
        self.button.update(cx, |button, cx| button.set_data(on, cx));
        cx.notify();
    }

    pub fn data(&self) -> &bool {
        &self.on
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

    pub fn set_switch_orientation(&mut self, orientation: SwitchOrientation, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_switch_orientation(orientation, cx));
        cx.notify();
    }

    pub fn set_switch_track_length_extra(&mut self, extra_length: f32, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_switch_track_length_extra(extra_length, cx));
        cx.notify();
    }

    pub fn set_switch_track_width_extra(&mut self, extra_width: f32, cx: &mut Context<Self>) {
        self.set_switch_track_length_extra(extra_width, cx);
    }

    pub fn set_switch_track_content<F, E>(&mut self, builder: F, cx: &mut Context<Self>)
    where
        F: Fn(&ButtonRenderModel<bool>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.button.update(cx, |button, cx| button.set_switch_track_content(builder, cx));
        cx.notify();
    }

    pub fn set_switch_thumb_content<F, E>(&mut self, builder: F, cx: &mut Context<Self>)
    where
        F: Fn(&ButtonRenderModel<bool>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.button.update(cx, |button, cx| button.set_switch_thumb_content(builder, cx));
        cx.notify();
    }

    fn handle_button_event(&mut self, _: Entity<Button<bool>>, event: &ButtonEvent, cx: &mut Context<Self>) {
        match event {
            ButtonEvent::Click => {
                let on = !self.on;
                self.on = on;
                self.button.update(cx, |button, cx| button.set_data(on, cx));
                cx.emit(SwitchEvent::Change { on });
                cx.notify();
            }
            ButtonEvent::FocusChanged { focused } => cx.emit(SwitchEvent::FocusChanged { focused: *focused }),
            ButtonEvent::EnabledChanged { enabled } => cx.emit(SwitchEvent::EnabledChanged { enabled: *enabled }),
            ButtonEvent::HoverChanged { hovered } => cx.emit(SwitchEvent::HoverChanged { hovered: *hovered }),
        }
    }
}

impl Focusable for SwitchControl {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.button.read(cx).focus_handle(cx)
    }
}

impl Render for SwitchControl {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(self.button.clone()).into_any_element()
    }
}

impl IntoElement for SwitchControl {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SwitchOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// Builder for [`Switch`] controls. Distinct from [`ButtonBuilder<bool>`] so Shadcn style
/// helpers apply the switch template rather than the checkbox template.
pub struct SwitchBuilder(ButtonBuilder<bool>);

impl SwitchBuilder {
    pub fn with_data(self, data: bool) -> Self {
        Self(self.0.with_data(data))
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self(self.0.enabled(enabled))
    }

    pub fn tab_stop(self, tab_stop: bool) -> Self {
        Self(self.0.tab_stop(tab_stop))
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

    pub fn orientation(mut self, orientation: SwitchOrientation) -> Self {
        self.0.model.switch_orientation = orientation;
        self
    }

    pub fn horizontal(self) -> Self {
        self.orientation(SwitchOrientation::Horizontal)
    }

    pub fn vertical(self) -> Self {
        self.orientation(SwitchOrientation::Vertical)
    }

    /// Adds extra length along the switch track's movement axis.
    pub fn track_length_extra(mut self, extra_length: f32) -> Self {
        self.0.model.switch_track_width_extra = extra_length.max(0.0);
        self
    }

    /// Adds extra width to a horizontal switch track for interior content such as ON/OFF labels.
    pub fn track_width_extra(self, extra_width: f32) -> Self {
        self.track_length_extra(extra_width)
    }

    /// Renders content inside the switch track behind the thumb.
    pub fn track_content<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&ButtonRenderModel<bool>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.0.model.switch_track_content = Some(Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        self
    }

    /// Renders content inside the moving switch thumb.
    pub fn thumb_content<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&ButtonRenderModel<bool>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.0.model.switch_thumb_content = Some(Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        self
    }

    pub fn size(self, size: ButtonSize) -> Self {
        Self(self.0.size(size))
    }

    pub fn template(self, template: Arc<dyn ButtonTemplate<bool>>) -> Self {
        Self(self.0.template(template))
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Switch {
        cx.new(|cx| SwitchControl::from_builder(self, cx))
    }
}

impl HasPresenter<ButtonRenderModel<bool>> for SwitchBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<bool>>) {
        self.0.set_presenter(content);
    }
}

impl Button<bool> {
    pub fn set_switch_orientation(&mut self, orientation: SwitchOrientation, cx: &mut Context<Self>) {
        self.model.switch_orientation = orientation;
        cx.notify();
    }

    pub fn set_switch_track_length_extra(&mut self, extra_length: f32, cx: &mut Context<Self>) {
        self.model.switch_track_width_extra = extra_length.max(0.0);
        cx.notify();
    }

    pub fn set_switch_track_width_extra(&mut self, extra_width: f32, cx: &mut Context<Self>) {
        self.set_switch_track_length_extra(extra_width, cx);
    }

    pub fn set_switch_track_content<F, E>(&mut self, builder: F, cx: &mut Context<Self>)
    where
        F: Fn(&ButtonRenderModel<bool>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.switch_track_content = Some(Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        cx.notify();
    }

    pub fn set_switch_thumb_content<F, E>(&mut self, builder: F, cx: &mut Context<Self>)
    where
        F: Fn(&ButtonRenderModel<bool>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.switch_thumb_content = Some(Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        cx.notify();
    }
}

pub fn new(id: impl Into<SharedString>) -> SwitchBuilder {
    SwitchBuilder(ButtonBuilder::new(id).typed(false).template(default_switch_template()))
}

#[cfg(test)]
mod tests {
    use gpui::div;

    use super::*;

    #[test]
    fn switch_builder_sets_slot_content_track_length_and_orientation() {
        let builder = new("slot-switch")
            .vertical()
            .track_length_extra(12.0)
            .track_content(|_, _| div())
            .thumb_content(|_, _| div());

        assert_eq!(builder.0.model.switch_track_width_extra, 12.0);
        assert_eq!(builder.0.model.switch_orientation, SwitchOrientation::Vertical);
        assert!(builder.0.model.switch_track_content.is_some());
        assert!(builder.0.model.switch_thumb_content.is_some());
    }
}
