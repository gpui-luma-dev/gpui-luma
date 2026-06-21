use std::sync::Arc;

use gpui::{Context, FontWeight, IntoElement, Render, Window, div, prelude::*};
use gpui_luma::controls::checkbox::Checkbox;
use gpui_luma::controls::command::button::ButtonEvent;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::progress::Progress;
use gpui_luma::controls::slider::{Slider, SliderEvent};
use gpui_luma::controls::switch::Switch;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnTextSize};
use gpui_luma::{declare_form, vstack};

use super::common::titled_card;

const SYSTEM_PREFERENCES_CARD_WIDTH: f32 = 380.0;

declare_form! {
    pub struct SystemPreferencesPanel {
        controls: {
            two_factor_switch: Switch = look
                .primary_switch("theme-studio-system-two-factor")
                .with_data(false)
                .content(|_, _| div().child("Two-factor authentication").into_any_element())
                => ButtonEvent |this, event, cx| {
                    this.handle_two_factor_event(event, cx);
                },
            terms_checkbox: Checkbox = look
                .primary_checkbox("theme-studio-system-terms")
                .with_data(false)
                .content(|_, _| div().child("I agree to the terms and conditions").into_any_element())
                => ButtonEvent |this, _event, cx| {
                    this.handle_terms_event(cx);
                },
            social_checkbox: Checkbox = look
                .primary_checkbox("theme-studio-system-social")
                .with_data(true)
                .content(|_, _| div().child("Social").into_any_element())
                => ButtonEvent |this, _event, cx| {
                    this.handle_social_event(cx);
                },
            referral_checkbox: Checkbox = look
                .primary_checkbox("theme-studio-system-referral")
                .with_data(false)
                .content(|_, _| div().child("Referral").into_any_element())
                => ButtonEvent |this, _event, cx| {
                    this.handle_referral_event(cx);
                },
            budget_slider: Slider = look
                .slider("theme-studio-system-budget")
                .range(0..100)
                .step(5)
                .value(40)
                => SliderEvent |this, event, cx| {
                    this.handle_budget_event(event, cx);
                },
            completion_progress: Progress = look
                .progress("theme-studio-system-completion")
                .range(0..100)
                .value(initial_completion as i32),
        },
        args: {
            look: Arc<ShadcnLook>,
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

impl SystemPreferencesPanel {
    fn handle_terms_event(&mut self, cx: &mut Context<Self>) {
        self.accepted_terms = !self.accepted_terms;
        self.terms_checkbox.update(cx, |button, cx| button.set_data(self.accepted_terms, cx));
        cx.notify();
    }

    fn handle_social_event(&mut self, cx: &mut Context<Self>) {
        self.social_source = !self.social_source;
        self.social_checkbox.update(cx, |button, cx| button.set_data(self.social_source, cx));
        cx.notify();
    }

    fn handle_referral_event(&mut self, cx: &mut Context<Self>) {
        self.referral_source = !self.referral_source;
        self.referral_checkbox.update(cx, |button, cx| button.set_data(self.referral_source, cx));
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
                cx.notify();
            }
        }
    }

    fn handle_budget_event(&mut self, event: &SliderEvent, cx: &mut Context<Self>) {
        let value = match event {
            SliderEvent::Change { value } | SliderEvent::Release { value } => *value,
        };
        self.budget = value;
        self.completion = value;
        self.completion_progress.update(cx, |progress, cx| progress.set_value(value as f64, cx));
        cx.notify();
    }
}

impl Render for SystemPreferencesPanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let label_style = self.look.typography_scale(ShadcnTextSize::Sm);
        let two_factor_switch = self.two_factor_switch.clone();
        let terms_checkbox = self.terms_checkbox.clone();
        let social_checkbox = self.social_checkbox.clone();
        let referral_checkbox = self.referral_checkbox.clone();
        let budget_slider = self.budget_slider.clone();
        let completion_progress = self.completion_progress.clone();
        let budget = self.budget;
        let completion = self.completion;

        titled_card(
            "theme-studio-system-preferences-card",
            &self.look,
            SYSTEM_PREFERENCES_CARD_WIDTH,
            "System & Preferences",
            "Choice controls plus progress feedback.",
            move |_, _| {
                vstack! {
                    gap=10.0;
                    two_factor_switch.clone(),
                    terms_checkbox.clone(),
                    social_checkbox.clone(),
                    referral_checkbox.clone(),
                    vstack! {
                        gap=6.0;
                        div()
                            .typography_style(label_style)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child(format!("Budget: {:.0}%", budget)),
                        budget_slider.clone(),
                    },
                    vstack! {
                        gap=6.0;
                        div()
                            .typography_style(label_style)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child(format!("Profile completion: {:.0}%", completion)),
                        completion_progress.clone(),
                    },
                }
                .into_any_element()
            },
            window,
            _cx,
        )
    }
}
