use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::checkbox::Checkbox;
use gpui_luma::controls::command::button::ButtonEvent;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::progress::Progress;
use gpui_luma::controls::slider::{Slider, SliderEvent};
use gpui_luma::controls::switch::Switch;
use gpui_luma::theme::radix::prelude::*;
use gpui_luma::theme::RadixTheme;

use crate::gallery::forms::declare_form;

use super::common::{card_container, card_title};
use super::pane::{AppEvent, EventBus};

declare_form! {
    pub(super) struct SystemPanel {
        controls: {
            terms_checkbox: Checkbox = radix_theme
                .primary_checkbox("intro-terms")
                .with_data(false)
                .content(|_, _| div().child("I agree to the terms and conditions").into_any_element())
                => ButtonEvent |this, _event, cx| {
                    this.handle_terms_event(cx);
                },
            social_checkbox: Checkbox = radix_theme
                .primary_checkbox("intro-social-source")
                .with_data(true)
                .content(|_, _| div().child("Social").into_any_element())
                => ButtonEvent |this, _event, cx| {
                    this.handle_social_event(cx);
                },
            referral_checkbox: Checkbox = radix_theme
                .primary_checkbox("intro-referral-source")
                .with_data(false)
                .content(|_, _| div().child("Referral").into_any_element())
                => ButtonEvent |this, _event, cx| {
                    this.handle_referral_event(cx);
                },
            two_factor_switch: Switch = radix_theme
                .primary_switch("intro-two-factor")
                .content(|_, _| div().child("Two-factor authentication").into_any_element())
                => ButtonEvent |this, event, cx| {
                    this.handle_two_factor_event(event, cx);
                },
            budget_slider: Slider = radix_theme.slider("intro-budget").range(0..100).step(5).value(40)
                => SliderEvent |this, event, cx| {
                    this.handle_budget_event(event, cx);
                },
            completion_progress: Progress = radix_theme.progress("intro-completion").range(0..100).value(initial_completion as i32),
        },
        args: {
            radix_theme: Arc<RadixTheme>,
            event_bus: Entity<EventBus>,
            initial_completion: f32,
        },
        fields: {
            accepted_terms: bool = false,
            social_source: bool = true,
            referral_source: bool = false,
            two_factor_enabled: bool = false,
            budget: f32 = 40.0,
            completion: f32 = initial_completion,
        }
    }
}

impl SystemPanel {
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
