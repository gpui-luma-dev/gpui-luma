use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::checkbox::{self, Checkbox};
use gpui_luma::controls::command::button::ButtonEvent;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::progress::{self, Progress};
use gpui_luma::controls::slider::{self, Slider, SliderEvent};
use gpui_luma::controls::switch::{self, Switch};
use gpui_luma::theme::{RadixButtonStyle, RadixTheme};

use super::common::{card_container, card_title};
use super::pane::{AppEvent, EventBus};

pub(super) struct SystemPanel {
    radix_theme: Arc<RadixTheme>,
    event_bus: Entity<EventBus>,
    terms_checkbox: Checkbox,
    social_checkbox: Checkbox,
    referral_checkbox: Checkbox,
    two_factor_switch: Switch,
    budget_slider: Slider,
    completion_progress: Progress,
    accepted_terms: bool,
    social_source: bool,
    referral_source: bool,
    two_factor_enabled: bool,
    budget: f32,
    completion: f32,
    _subscriptions: Vec<Subscription>,
}

impl SystemPanel {
    pub(super) fn new(
        cx: &mut Context<Self>,
        radix_theme: Arc<RadixTheme>,
        event_bus: Entity<EventBus>,
        initial_completion: f32,
    ) -> Self {
        let terms_checkbox = checkbox::new("intro-terms")
            .template(radix_theme.checkbox_template(RadixButtonStyle::Primary))
            .with_data(false)
            .content(|_, _| div().child("I agree to the terms and conditions").into_any_element())
            .spawn(cx);
        let social_checkbox = checkbox::new("intro-social-source")
            .template(radix_theme.checkbox_template(RadixButtonStyle::Primary))
            .with_data(true)
            .content(|_, _| div().child("Social").into_any_element())
            .spawn(cx);
        let referral_checkbox = checkbox::new("intro-referral-source")
            .template(radix_theme.checkbox_template(RadixButtonStyle::Primary))
            .with_data(false)
            .content(|_, _| div().child("Referral").into_any_element())
            .spawn(cx);
        let two_factor_switch = switch::new("intro-two-factor")
            .template(radix_theme.switch_template(RadixButtonStyle::Primary))
            .content(|_, _| div().child("Two-factor authentication").into_any_element())
            .spawn(cx);
        let budget_slider = slider::new("intro-budget")
            .template(radix_theme.slider_template())
            .range(0..100)
            .step(5)
            .value(40)
            .spawn(cx);
        let completion_progress = progress::new("intro-completion")
            .template(radix_theme.progress_template())
            .range(0..100)
            .value(initial_completion as i32)
            .spawn(cx);

        let subscriptions = vec![
            cx.subscribe(&terms_checkbox, |this, _, _: &ButtonEvent, cx| {
                this.handle_terms_event(cx);
            }),
            cx.subscribe(&social_checkbox, |this, _, _: &ButtonEvent, cx| {
                this.handle_social_event(cx);
            }),
            cx.subscribe(&referral_checkbox, |this, _, _: &ButtonEvent, cx| {
                this.handle_referral_event(cx);
            }),
            cx.subscribe(&two_factor_switch, |this, _, event: &ButtonEvent, cx| {
                this.handle_two_factor_event(event, cx);
            }),
            cx.subscribe(&budget_slider, |this, _, event: &SliderEvent, cx| {
                this.handle_budget_event(event, cx);
            }),
        ];

        Self {
            radix_theme,
            event_bus,
            terms_checkbox,
            social_checkbox,
            referral_checkbox,
            two_factor_switch,
            budget_slider,
            completion_progress,
            accepted_terms: false,
            social_source: true,
            referral_source: false,
            two_factor_enabled: false,
            budget: 40.0,
            completion: initial_completion,
            _subscriptions: subscriptions,
        }
    }

    pub(super) fn set_completion(&mut self, completion: f32, cx: &mut Context<Self>) {
        self.completion = completion;
        self.completion_progress.update(cx, |progress, cx| progress.set_value(completion as f64, cx));
        cx.notify();
    }

    fn handle_terms_event(&mut self, cx: &mut Context<Self>) {
        self.accepted_terms = !self.accepted_terms;
        self.terms_checkbox.update(cx, |button, cx| button.set_data(self.accepted_terms, cx));
        self.emit_change("Checkbox::Terms", cx);
        cx.notify();
    }

    fn handle_social_event(&mut self, cx: &mut Context<Self>) {
        self.social_source = !self.social_source;
        self.social_checkbox.update(cx, |button, cx| button.set_data(self.social_source, cx));
        self.emit_change("Checkbox::Social", cx);
        cx.notify();
    }

    fn handle_referral_event(&mut self, cx: &mut Context<Self>) {
        self.referral_source = !self.referral_source;
        self.referral_checkbox.update(cx, |button, cx| button.set_data(self.referral_source, cx));
        self.emit_change("Checkbox::Referral", cx);
        cx.notify();
    }

    fn handle_two_factor_event(&mut self, event: &ButtonEvent, cx: &mut Context<Self>) {
        match event {
            ButtonEvent::Click => {
                self.two_factor_switch.update(cx, |button, cx| {
                    let next_on = !*button.data();
                    button.set_data(next_on, cx);
                });
                self.two_factor_enabled = !self.two_factor_enabled;
                self.emit_change("Switch::TwoFactor", cx);
                cx.notify();
            }
        }
    }

    fn handle_budget_event(&mut self, event: &SliderEvent, cx: &mut Context<Self>) {
        let SliderEvent::Change { value } = event;
        self.budget = *value;
        self.emit_change("Slider::Budget", cx);
        cx.notify();
    }

    fn emit_change(&self, event_name: &'static str, cx: &mut Context<Self>) {
        let budget = self.budget;
        let accepted_terms = self.accepted_terms;
        let social_source = self.social_source;
        let referral_source = self.referral_source;
        let two_factor_enabled = self.two_factor_enabled;

        self.event_bus.update(cx, |_bus, cx| {
            cx.emit(AppEvent::SystemChanged {
                budget,
                accepted_terms,
                social_source,
                referral_source,
                two_factor_enabled,
                event_name: SharedString::from(event_name),
            });
        });
    }
}

impl Render for SystemPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();

        card_container(chrome.border, chrome.panel_background)
            .child(card_title(
                "System & Preferences",
                "Choice controls plus progress feedback.",
                chrome.title_text,
                chrome.muted_text,
            ))
            .child(self.two_factor_switch.clone())
            .child(self.terms_checkbox.clone())
            .child(self.social_checkbox.clone())
            .child(self.referral_checkbox.clone())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .line_height(px(16.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child(format!("Budget: {:.0}%", self.budget)),
                    )
                    .child(self.budget_slider.clone()),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .line_height(px(16.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child(format!("Profile completion: {:.0}%", self.completion)),
                    )
                    .child(self.completion_progress.clone()),
            )
    }
}
