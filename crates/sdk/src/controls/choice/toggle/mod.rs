use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, SharedString, Subscription, Window, div, hsla, prelude::*,
};

use crate::motion::{DEFAULT_TRANSITION_DURATION, VisualTransition};
use crate::controls::button_family::{
    ButtonFamilyPalette, ButtonFamilyRole, button_family_effective_border, default_button_family_theme,
};
use crate::controls::command::button::{
    Button, ButtonBuilder, ButtonEvent, ButtonRenderModel, ButtonTemplate, DefaultButtonTemplate,
};
use crate::infra::presenter::{ControlPresenter, HasPresenter};
use crate::theme::ControlSize;

pub type Toggle = Entity<ToggleControl>;

/// Typed payload for [`Button<ToggleData>`] / [`ButtonTemplate<ToggleData>`].
///
/// `selected` is the settled semantic value; `progress` is the continuous visual factor
/// (`0.0`..`1.0`) driven by [`VisualTransition`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleData {
    pub selected: bool,
    pub progress: f32,
}

impl ToggleData {
    pub fn new(selected: bool) -> Self {
        Self { selected, progress: if selected { 1.0 } else { 0.0 } }
    }
}

impl Default for ToggleData {
    fn default() -> Self {
        Self::new(false)
    }
}

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
    animated: bool,
    transition: VisualTransition,
    button: Entity<Button<ToggleData>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ToggleEvent> for ToggleControl {}

impl ToggleControl {
    fn from_builder(builder: ToggleBuilder, cx: &mut Context<Self>) -> Self {
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
        button_builder.model.data = ToggleData { selected, progress };
        button_builder.model.role = ButtonFamilyRole::Toggle { selected };
        let button = button_builder.spawn(cx);
        let subscription = cx.subscribe(&button, Self::handle_button_event);

        Self { selected, animated, transition, button, _subscriptions: vec![subscription] }
    }

    fn push_button_data(&mut self, cx: &mut Context<Self>) {
        let data = ToggleData { selected: self.selected, progress: self.transition.progress() };
        let role = ButtonFamilyRole::Toggle { selected: self.selected };
        self.button.update(cx, |button, cx| {
            let role_changed = button.model.role != role;
            let data_changed = button.data() != &data;
            if !role_changed && !data_changed {
                return;
            }
            button.model.role = role;
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

    pub fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<ToggleData>>, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_presenter(content, cx));
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn ButtonTemplate<ToggleData>>, cx: &mut Context<Self>) {
        self.button.update(cx, |button, cx| button.set_template(template, cx));
        cx.notify();
    }

    fn handle_button_event(&mut self, _: Entity<Button<ToggleData>>, event: &ButtonEvent, cx: &mut Context<Self>) {
        match event {
            ButtonEvent::Click => {
                let selected = !self.selected;
                self.selected = selected;
                self.transition.set_target(if selected { 1.0 } else { 0.0 });
                self.push_button_data(cx);
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

impl IntoElement for ToggleControl {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

pub struct ToggleBuilder {
    button: ButtonBuilder<ToggleData>,
    animated: bool,
}

impl ToggleBuilder {
    pub fn icon(self, icon: impl Into<crate::controls::command::button::ControlIcon>) -> Self {
        Self { button: self.button.icon(icon), ..self }
    }

    pub fn with_data(self, selected: bool) -> Self {
        Self { button: self.button.with_data(ToggleData::new(selected)), ..self }
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

    pub fn role(self, role: ButtonFamilyRole) -> Self {
        Self { button: self.button.role(role), ..self }
    }

    pub fn round(self, round: bool) -> Self {
        Self { button: self.button.round(round), ..self }
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

    pub fn template(self, template: Arc<dyn ButtonTemplate<ToggleData>>) -> Self {
        Self { button: self.button.template(template), ..self }
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Toggle {
        cx.new(|cx| ToggleControl::from_builder(self, cx))
    }
}

impl HasPresenter<ButtonRenderModel<ToggleData>> for ToggleBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<ToggleData>>) {
        self.button.set_presenter(content);
    }
}

fn button_builder(id: impl Into<SharedString>) -> ButtonBuilder<ToggleData> {
    ButtonBuilder::new(id)
        .typed(ToggleData::default())
        .role(ButtonFamilyRole::Toggle { selected: false })
        .template(default_toggle_template())
}

pub fn new(id: impl Into<SharedString>) -> ToggleBuilder {
    ToggleBuilder { button: button_builder(id), animated: true }
}

fn lerp_f32(start: f32, end: f32, t: f32) -> f32 {
    start + ((end - start) * t)
}

fn lerp_hsla(start: gpui::Hsla, end: gpui::Hsla, t: f32) -> gpui::Hsla {
    hsla(
        lerp_f32(start.h, end.h, t),
        lerp_f32(start.s, end.s, t),
        lerp_f32(start.l, end.l, t),
        lerp_f32(start.a, end.a, t),
    )
}

fn lerp_optional_hsla(start: Option<gpui::Hsla>, end: Option<gpui::Hsla>, t: f32) -> Option<gpui::Hsla> {
    match (start, end) {
        (Some(a), Some(b)) => Some(lerp_hsla(a, b, t)),
        (Some(a), None) if t < 0.5 => Some(a),
        (None, Some(b)) if t >= 0.5 => Some(b),
        (Some(_), None) | (None, Some(_)) => None,
        (None, None) => None,
    }
}

fn lerp_toggle_palette(off: &ButtonFamilyPalette, on: &ButtonFamilyPalette, progress: f32) -> ButtonFamilyPalette {
    let t = progress.clamp(0.0, 1.0);
    let settled = if t >= 0.5 { on } else { off };
    ButtonFamilyPalette {
        background: lerp_hsla(off.background, on.background, t),
        foreground: lerp_hsla(off.foreground, on.foreground, t),
        border: lerp_optional_hsla(off.border, on.border, t),
        typography: settled.typography,
        font_family: settled.font_family.clone(),
    }
}

/// Applies interpolated toggle chrome from off/on palettes for `model.data.progress`.
pub fn apply_toggle_progress_chrome(
    element: gpui::Stateful<gpui::Div>,
    off: &ButtonFamilyPalette,
    on: &ButtonFamilyPalette,
    progress: f32,
) -> gpui::Stateful<gpui::Div> {
    let palette = lerp_toggle_palette(off, on, progress);
    element
        .bg(palette.background)
        .text_color(palette.foreground)
        .border_color(button_family_effective_border(palette.border))
}

pub fn default_toggle_template() -> Arc<dyn ButtonTemplate<ToggleData>> {
    let button_family_theme = default_button_family_theme();
    Arc::new(DefaultButtonTemplate::<ToggleData>::new(button_family_theme.clone()).with_modifier(
        move |element, model| {
            if model.look.is_some() {
                return element;
            }

            let off =
                button_family_theme.resolve(ButtonFamilyRole::Toggle { selected: false }, model.size, model.state);
            let on = button_family_theme.resolve(ButtonFamilyRole::Toggle { selected: true }, model.size, model.state);
            apply_toggle_progress_chrome(element, &off, &on, model.data.progress)
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_builder_animated_option() {
        assert!(new("animated-default").animated);
        assert!(!new("animated-off").animated(false).animated);
    }

    #[test]
    fn toggle_data_seeds_progress_from_selected() {
        assert_eq!(ToggleData::new(true).progress, 1.0);
        assert_eq!(ToggleData::new(false).progress, 0.0);
    }
}
