use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, EventEmitter, FontWeight, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::{flow, hstack, vstack};
use gpui_luma::theme::RadixTheme;

use crate::gallery::control::GalleryApp;

use super::super::shared::{gallery_pane_with_description, notify_entity};
use super::payment_panel::PaymentPanel;
use super::system_panel::SystemPanel;
use super::workspace_panel::WorkspacePanel;

const INTRO_DESCRIPTION: &str = concat!(
    "A control-dense landing page built with real interactive gpui-luma controls. ",
    "Use it as a compositional reference for form-heavy product screens."
);

const INTRO_HEADING: &str = "Foundation Controls for GPUI";
const INTRO_SUBHEADING: &str = "A real, interactive introduction screen that combines command, input, choice, and feedback controls in one composition.";

#[derive(Clone)]
pub(in crate::gallery) struct IntroductionPane {
    event_bus: Entity<EventBus>,
    payment_panel: Entity<PaymentPanel>,
    workspace_panel: Entity<WorkspacePanel>,
    system_panel: Entity<SystemPanel>,

    name_value: SharedString,
    email_value: SharedString,
    payment_selection_set: bool,

    same_as_shipping: bool,
    default_payment_method: bool,
    accepted_terms: bool,
    social_source: bool,
    referral_source: bool,
    two_factor_enabled: bool,

    pub(super) workspace_layout: SharedString,
    pub(super) workspace_density: SharedString,
    pub(super) workspace_icon_demo: SharedString,
    pub(super) workspace_action: SharedString,

    pub(super) budget: f32,
    pub(super) completion: f32,
    clicks_submit: usize,
    clicks_cancel: usize,
    last_event: SharedString,
}

#[derive(Clone, Debug)]
pub(super) enum AppEvent {
    PaymentSubmit,
    PaymentCancel,
    PaymentChanged {
        name: SharedString,
        email: SharedString,
        payment_selection_set: bool,
        same_as_shipping: bool,
        default_payment_method: bool,
        event_name: SharedString,
    },
    WorkspaceChanged {
        layout: SharedString,
        density: SharedString,
        icon_demo: SharedString,
        action: SharedString,
        event_name: SharedString,
    },
    SystemChanged {
        budget: f32,
        accepted_terms: bool,
        social_source: bool,
        referral_source: bool,
        two_factor_enabled: bool,
        event_name: SharedString,
    },
}

pub(super) struct EventBus;

impl EventEmitter<AppEvent> for EventBus {}

struct CompletionInputs<'a> {
    name_value: &'a SharedString,
    email_value: &'a SharedString,
    payment_selection_set: bool,
    accepted_terms: bool,
    social_source: bool,
    referral_source: bool,
    two_factor_enabled: bool,
    workspace_density: &'a SharedString,
    budget: f32,
}

impl IntroductionPane {
    // ===== Construction =====

    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let event_bus = cx.new(|_| EventBus);
        let payment_panel = cx.new(|cx| PaymentPanel::new(cx, radix_theme.clone(), event_bus.clone()));
        let workspace_panel = cx.new(|cx| WorkspacePanel::new(cx, radix_theme.clone(), event_bus.clone()));

        let name_value = SharedString::default();
        let email_value = SharedString::default();
        let payment_selection_set = false;
        let same_as_shipping = true;
        let default_payment_method = true;
        let accepted_terms = false;
        let social_source = true;
        let referral_source = false;
        let two_factor_enabled = false;
        let workspace_layout = SharedString::from("Grid");
        let workspace_density = SharedString::from("Balanced");
        let workspace_icon_demo = SharedString::from("Left");
        let workspace_action = SharedString::from("None");
        let budget = 40.0;
        let completion = Self::compute_completion(&CompletionInputs {
            name_value: &name_value,
            email_value: &email_value,
            payment_selection_set,
            accepted_terms,
            social_source,
            referral_source,
            two_factor_enabled,
            workspace_density: &workspace_density,
            budget,
        });
        let system_panel = cx.new(|cx| SystemPanel::new(cx, radix_theme.clone(), event_bus.clone(), completion));

        Self {
            event_bus,
            payment_panel,
            workspace_panel,
            system_panel,
            name_value,
            email_value,
            payment_selection_set,
            same_as_shipping,
            default_payment_method,
            accepted_terms,
            social_source,
            referral_source,
            two_factor_enabled,
            workspace_layout,
            workspace_density,
            workspace_icon_demo,
            workspace_action,
            budget,
            completion,
            clicks_submit: 0,
            clicks_cancel: 0,
            last_event: SharedString::from("Ready"),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.event_bus, |app, _, event: &AppEvent, cx| {
            app.panes.introduction.handle_app_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.payment_panel, cx);
        notify_entity(&self.workspace_panel, cx);
        notify_entity(&self.system_panel, cx);
    }

    // ===== Render =====

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        let chrome = radix_theme.chrome();

        gallery_pane_with_description(
            "Introduction",
            Some(INTRO_DESCRIPTION),
            hstack! {
                vstack! {
                    gap=16.0;
                    self.render_intro_header(chrome.title_text, chrome.body_text),
                    vstack! {
                        gap=16.0;
                        self.render_panel_row(),
                        self.render_status_line(chrome.muted_text),
                    }
                    .w_full(),
                }
                .w_full()
                .max_w(px(1180.0)),
            }
            .w_full()
            .pt(px(32.0))
            .justify_center()
            .into_any_element(),
            radix_theme,
        )
    }

    fn render_intro_header(&self, title_color: gpui::Hsla, body_color: gpui::Hsla) -> AnyElement {
        vstack! {
            gap=32.0 align=start;
            div()
                .text_size(px(44.0))
                .line_height(px(52.0))
                .font_weight(FontWeight::BOLD)
                .text_color(title_color)
                .child(INTRO_HEADING),
            div().text_size(px(17.0)).line_height(px(26.0)).text_color(body_color).child(INTRO_SUBHEADING),
        }
        .w_full()
        .max_w(px(860.0))
        .into_any_element()
    }

    fn render_panel_row(&self) -> AnyElement {
        flow! {
            gap=16.0;
            self.payment_panel.clone(),
            self.workspace_panel.clone(),
            self.system_panel.clone(),
        }
        .w_full()
        .mt(px(32.0))
        .justify_center()
        .items_stretch()
        .into_any_element()
    }

    fn render_status_line(&self, muted_text: gpui::Hsla) -> AnyElement {
        div()
            .pt(px(2.0))
            .text_size(px(11.0))
            .line_height(px(16.0))
            .text_color(muted_text)
            .child(format!(
                "Clicks: submit={}, cancel={} | Last event: {}",
                self.clicks_submit, self.clicks_cancel, self.last_event
            ))
            .into_any_element()
    }

    fn handle_app_event(&mut self, event: &AppEvent, cx: &mut Context<GalleryApp>) {
        match event {
            AppEvent::PaymentSubmit => {
                self.clicks_submit += 1;
                self.last_event = SharedString::from("Button::Submit");
            }
            AppEvent::PaymentCancel => {
                self.clicks_cancel += 1;
                self.last_event = SharedString::from("Button::Cancel");
            }
            AppEvent::PaymentChanged {
                name,
                email,
                payment_selection_set,
                same_as_shipping,
                default_payment_method,
                event_name,
            } => {
                self.name_value = name.clone();
                self.email_value = email.clone();
                self.payment_selection_set = *payment_selection_set;
                self.same_as_shipping = *same_as_shipping;
                self.default_payment_method = *default_payment_method;
                self.last_event = event_name.clone();
                self.recompute_completion(cx);
            }
            AppEvent::WorkspaceChanged { layout, density, icon_demo, action, event_name } => {
                self.workspace_layout = layout.clone();
                self.workspace_density = density.clone();
                self.workspace_icon_demo = icon_demo.clone();
                self.workspace_action = action.clone();
                self.last_event = event_name.clone();
                self.recompute_completion(cx);
            }
            AppEvent::SystemChanged {
                budget,
                accepted_terms,
                social_source,
                referral_source,
                two_factor_enabled,
                event_name,
            } => {
                self.budget = *budget;
                self.accepted_terms = *accepted_terms;
                self.social_source = *social_source;
                self.referral_source = *referral_source;
                self.two_factor_enabled = *two_factor_enabled;
                self.last_event = event_name.clone();
                self.recompute_completion(cx);
            }
        }

        cx.notify();
    }

    fn recompute_completion(&mut self, cx: &mut Context<GalleryApp>) {
        self.completion = Self::compute_completion(&self.completion_inputs());

        let completion = self.completion;
        self.system_panel.update(cx, |panel, cx| panel.set_completion(completion, cx));
    }

    fn completion_inputs(&self) -> CompletionInputs<'_> {
        CompletionInputs {
            name_value: &self.name_value,
            email_value: &self.email_value,
            payment_selection_set: self.payment_selection_set,
            accepted_terms: self.accepted_terms,
            social_source: self.social_source,
            referral_source: self.referral_source,
            two_factor_enabled: self.two_factor_enabled,
            workspace_density: &self.workspace_density,
            budget: self.budget,
        }
    }

    fn compute_completion(inputs: &CompletionInputs<'_>) -> f32 {
        let mut score = 0.0f32;
        let mut max_score = 0.0f32;

        max_score += 1.0;
        if !inputs.name_value.is_empty() {
            score += 1.0;
        }

        max_score += 1.0;
        if !inputs.email_value.is_empty() {
            score += 1.0;
        }

        max_score += 1.0;
        if inputs.payment_selection_set {
            score += 1.0;
        }

        max_score += 1.0;
        if inputs.accepted_terms {
            score += 1.0;
        }

        max_score += 1.0;
        if inputs.two_factor_enabled {
            score += 1.0;
        }

        max_score += 1.0;
        if inputs.social_source || inputs.referral_source {
            score += 1.0;
        }

        max_score += 1.0;
        if !inputs.workspace_density.is_empty() && inputs.workspace_density.as_ref() != "None" {
            score += 1.0;
        }

        max_score += 1.0;
        score += inputs.budget / 100.0;

        if max_score > 0.0 {
            (score / max_score) * 100.0
        } else {
            0.0
        }
    }
}
