use gpui::{AnyElement, Context, Entity, FontWeight, IntoElement, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent, ButtonKind};

use gpui_luma::controls::checkbox::{self, Checkbox};
use gpui_luma::controls::choice_group::{self, ChoiceGroup, ChoiceGroupEvent, ChoiceGroupItem};

use gpui_luma::controls::content_presenter::HasContent;
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::popup_menu::{PopupMenu, PopupMenuEvent, PopupMenuPlacement};
use gpui_luma::controls::progress::{self, Progress};
use gpui_luma::controls::radio_button;
use gpui_luma::controls::slider::{self, Slider, SliderEvent};
use gpui_luma::controls::switch::{self, Switch};
use gpui_luma::controls::textfield::{self, TextField, TextFieldEvent};

use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_description, notify_entity};

const INTRO_DESCRIPTION: &str = concat!(
    "A control-dense landing page built with real interactive gpui-luma controls. ",
    "Use it as a compositional reference for form-heavy product screens."
);

#[derive(Clone)]
struct PaymentPanel {
    submit_button: Entity<Button>,
    cancel_button: Entity<Button>,
    name_field: TextField,
    email_field: TextField,
    card_field: TextField,
    same_as_shipping_checkbox: Checkbox,
}

#[derive(Clone)]
struct WorkspacePanel {
    workspace_popup_menu: Entity<PopupMenu>,
    workspace_layout_choice_group: ChoiceGroup,
    workspace_icon_demo_choice_group: ChoiceGroup,
    workspace_density_choice_group: ChoiceGroup,
}

#[derive(Clone)]
struct SystemPanel {
    terms_checkbox: Checkbox,
    social_checkbox: Checkbox,
    referral_checkbox: Checkbox,
    two_factor_switch: Switch,
    budget_slider: Slider,
    completion_progress: Progress,
}

#[derive(Clone)]
pub(in crate::gallery) struct IntroductionPane {
    payment: PaymentPanel,
    workspace: WorkspacePanel,
    system: SystemPanel,

    name_value: SharedString,
    email_value: SharedString,
    card_value: SharedString,

    same_as_shipping: bool,
    accepted_terms: bool,
    social_source: bool,
    referral_source: bool,
    two_factor_enabled: bool,

    workspace_layout: SharedString,
    workspace_density: SharedString,
    workspace_icon_demo: SharedString,
    workspace_action: SharedString,

    budget: f32,
    completion: f32,
    clicks_submit: usize,
    clicks_cancel: usize,
    last_event: SharedString,
}

#[derive(Clone, Copy)]
enum IntroButton {
    Submit,
    Cancel,
}

#[derive(Clone, Copy)]
enum IntroField {
    Name,
    Email,
    Card,
}

impl IntroductionPane {
    // ===== Construction =====

    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let checkbox_template = theme.checkbox_template();

        let payment = PaymentPanel {
            submit_button: Button::new("intro-submit").label("Submit").kind(ButtonKind::Prominent).spawn(cx),
            cancel_button: Button::new("intro-cancel").label("Cancel").spawn(cx),
            name_field: textfield::new("intro-name")
                .placeholder("Name on card")
                .full_width(true)
                .clean_on_escape(true)
                .spawn(cx),
            email_field: textfield::new("intro-email")
                .placeholder("Email address")
                .full_width(true)
                .clean_on_escape(true)
                .spawn(cx),
            card_field: textfield::new("intro-card")
                .placeholder("1234 5678 9012 3456")
                .full_width(true)
                .clean_on_escape(true)
                .spawn(cx),
            same_as_shipping_checkbox: checkbox::new("intro-same-as-shipping")
                .data(true)
                .content(|_, _| div().child("Same as shipping address").into_any_element())
                .template(checkbox_template.clone())
                .spawn(cx),
        };

        let workspace = WorkspacePanel {
            workspace_layout_choice_group: choice_group::toolbar_icons_multiple("intro-workspace-layout")
                .managed_selected("grid")
                .items(workspace_layout_items())
                .content(|item, _| {
                    let icon = match item.item_id.as_ref() {
                        "grid" => LucideIcon::PanelTop,
                        "list" => LucideIcon::List,
                        "kanban" => LucideIcon::Columns3,
                        _ => LucideIcon::Settings,
                    };

                    div().font_family("lucide").child(char::from(icon).to_string()).into_any_element()
                })
                .spawn(cx),
            workspace_density_choice_group: choice_group::radio_group("intro-workspace-density")
                .items(workspace_density_items())
                .selected("balanced")
                //.vertical()
                .bool_button_template_factory(|_| {
                    std::sync::Arc::new(
                        radio_button::ThemedRadioButtonTemplate::new(gpui_luma::theme::default_radio_button_theme())
                            .with_modifier(|element, _| element.min_h(px(22.0)).py(px(0.0))),
                    )
                })
                .with_modifier(|element, _| {
                    let transparent = gpui::hsla(0.0, 0.0, 0.0, 0.0);
                    element.bg(transparent).border_color(transparent)
                })
                .spawn(cx),
            workspace_popup_menu: PopupMenu::new("intro-workspace-popup")
                .label("Workspace Menu")
                .items(workspace_menu_items())
                .placement(PopupMenuPlacement::BelowStart)
                .spawn(cx),
            workspace_icon_demo_choice_group: choice_group::toolbar_icons("intro-workspace-icon-demo")
                .managed_selected("left")
                .items(workspace_icon_demo_items())
                .content(|item, _| {
                    let icon = match item.item_id.as_ref() {
                        "left" => LucideIcon::List,
                        "center" => LucideIcon::PanelTop,
                        "right" => LucideIcon::Columns3,
                        _ => LucideIcon::Settings,
                    };

                    div().font_family("lucide").child(char::from(icon).to_string()).into_any_element()
                })
                .spawn(cx),
        };

        let system = SystemPanel {
            terms_checkbox: checkbox::new("intro-terms")
                .data(false)
                .content(|_, _| div().child("I agree to the terms and conditions").into_any_element())
                .template(checkbox_template.clone())
                .spawn(cx),
            social_checkbox: checkbox::new("intro-social-source")
                .data(true)
                .content(|_, _| div().child("Social").into_any_element())
                .template(checkbox_template.clone())
                .spawn(cx),
            referral_checkbox: checkbox::new("intro-referral-source")
                .data(false)
                .content(|_, _| div().child("Referral").into_any_element())
                .template(checkbox_template)
                .spawn(cx),
            two_factor_switch: switch::new("intro-two-factor")
                .content(|_, _| div().child("Two-factor authentication").into_any_element())
                .spawn(cx),
            budget_slider: slider::new("intro-budget").range(0..100).step(5).value(40).spawn(cx),
            completion_progress: progress::new("intro-completion").range(0..100).value(30).spawn(cx),
        };

        Self {
            payment,
            workspace,
            system,

            name_value: SharedString::default(),
            email_value: SharedString::default(),
            card_value: SharedString::default(),

            same_as_shipping: true,
            accepted_terms: false,
            social_source: true,
            referral_source: false,
            two_factor_enabled: false,

            workspace_layout: SharedString::from("Grid"),
            workspace_density: SharedString::from("Balanced"),
            workspace_icon_demo: SharedString::from("Left"),
            workspace_action: SharedString::from("None"),

            budget: 40.0,
            completion: 30.0,
            clicks_submit: 0,
            clicks_cancel: 0,
            last_event: SharedString::from("Ready"),
        }
    }

    // ===== Wiring =====

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        self.subscribe_payment_panel(cx, subscriptions);
        self.subscribe_workspace_panel(cx, subscriptions);
        self.subscribe_system_panel(cx, subscriptions);
    }

    fn subscribe_payment_panel(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.payment.submit_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_button_event(IntroButton::Submit, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.payment.cancel_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_button_event(IntroButton::Cancel, event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.payment.name_field, |app, _, event: &TextFieldEvent, cx| {
            app.panes.introduction.handle_textfield_event(IntroField::Name, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.payment.email_field, |app, _, event: &TextFieldEvent, cx| {
            app.panes.introduction.handle_textfield_event(IntroField::Email, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.payment.card_field, |app, _, event: &TextFieldEvent, cx| {
            app.panes.introduction.handle_textfield_event(IntroField::Card, event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.payment.same_as_shipping_checkbox, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_same_as_shipping_event(event, cx);
        }));
    }

    fn subscribe_workspace_panel(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.workspace.workspace_popup_menu, |app, _, event: &PopupMenuEvent, cx| {
            app.panes.introduction.handle_popup_menu_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(
            &self.workspace.workspace_layout_choice_group,
            |app, _, event: &ChoiceGroupEvent, cx| {
                app.panes.introduction.handle_workspace_layout_event(event, cx);
            },
        ));
        subscriptions.push(cx.subscribe(
            &self.workspace.workspace_icon_demo_choice_group,
            |app, _, event: &ChoiceGroupEvent, cx| {
                app.panes.introduction.handle_workspace_icon_demo_event(event, cx);
            },
        ));
        subscriptions.push(cx.subscribe(
            &self.workspace.workspace_density_choice_group,
            |app, _, event: &ChoiceGroupEvent, cx| {
                app.panes.introduction.handle_workspace_density_event(event, cx);
            },
        ));
    }

    fn subscribe_system_panel(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.system.terms_checkbox, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_terms_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.system.social_checkbox, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_social_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.system.referral_checkbox, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_referral_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.system.two_factor_switch, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_two_factor_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.system.budget_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.introduction.handle_budget_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.notify_payment_panel(cx);
        self.notify_workspace_panel(cx);
        self.notify_system_panel(cx);
    }

    fn notify_payment_panel(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.payment.submit_button, cx);
        notify_entity(&self.payment.cancel_button, cx);
        notify_entity(&self.payment.name_field, cx);
        notify_entity(&self.payment.email_field, cx);
        notify_entity(&self.payment.card_field, cx);
        notify_entity(&self.payment.same_as_shipping_checkbox, cx);
    }

    fn notify_workspace_panel(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.workspace.workspace_popup_menu, cx);
        notify_entity(&self.workspace.workspace_layout_choice_group, cx);
        notify_entity(&self.workspace.workspace_icon_demo_choice_group, cx);
        notify_entity(&self.workspace.workspace_density_choice_group, cx);
    }

    fn notify_system_panel(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.system.terms_checkbox, cx);
        notify_entity(&self.system.social_checkbox, cx);
        notify_entity(&self.system.referral_checkbox, cx);
        notify_entity(&self.system.two_factor_switch, cx);
        notify_entity(&self.system.budget_slider, cx);
        notify_entity(&self.system.completion_progress, cx);
    }

    // ===== Render =====

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_description(
            "Introduction",
            Some(INTRO_DESCRIPTION),
            div()
                .w_full()
                .flex()
                .justify_center()
                .child(
                    div()
                        .w_full()
                        .max_w(px(1180.0))
                        .flex()
                        .flex_col()
                        .gap(px(24.0))
                        .child(self.render_intro_header(chrome.title_text, chrome.body_text))
                        .child(self.render_panel_row(theme))
                        .child(self.render_status_line(chrome.muted_text)),
                )
                .into_any_element(),
            theme,
        )
    }

    fn render_intro_header(&self, title_color: gpui::Hsla, body_color: gpui::Hsla) -> AnyElement {
        div()
            .w_full()
            .max_w(px(860.0))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(10.0))
            .child(
                div()
                    .text_size(px(44.0))
                    .line_height(px(52.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(title_color)
                    .child("Foundation Controls for GPUI"),
            )
            .child(
                div().text_size(px(17.0)).line_height(px(26.0)).text_color(body_color).child(
                    "A real, interactive introduction screen that combines command, input, choice, and feedback controls in one composition.",
                ),
            )
            .into_any_element()
    }

    fn render_panel_row(&self, theme: &GalleryThemePack) -> AnyElement {
        div()
            .w_full()
            .flex()
            .flex_wrap()
            .justify_center()
            .items_stretch()
            .gap(px(16.0))
            .child(self.render_payment_card(theme))
            .child(self.render_workspace_card(theme))
            .child(self.render_system_card(theme))
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

    fn render_payment_card(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        card_container(chrome.border, chrome.panel_background)
            .child(card_title(
                "Payment Method",
                "All transactions are secure and encrypted.",
                chrome.title_text,
                chrome.muted_text,
            ))
            .child(self.payment.name_field.clone())
            .child(self.payment.email_field.clone())
            .child(self.payment.card_field.clone())
            .child(self.payment.same_as_shipping_checkbox.clone())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(self.payment.submit_button.clone())
                    .child(self.payment.cancel_button.clone()),
            )
            .into_any_element()
    }

    fn render_workspace_card(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        card_container(chrome.border, chrome.panel_background)
            .child(card_title(
                "Workspace",
                "Toggle groups, radio groups, icon actions, and popup menus.",
                chrome.title_text,
                chrome.muted_text,
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .line_height(px(16.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child(format!("Layout: {}", self.workspace_layout)),
                    )
                    .child(self.workspace.workspace_layout_choice_group.clone()),
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
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child(format!("Density: {}", self.workspace_density)),
                    )
                    .child(self.workspace.workspace_density_choice_group.clone()),
            )
            .child(div().flex().items_center().gap(px(8.0)).child(self.workspace.workspace_popup_menu.clone()))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .line_height(px(15.0))
                            .text_color(chrome.muted_text)
                            .child(format!("Icon demo: {}", self.workspace_icon_demo)),
                    )
                    .child(self.workspace.workspace_icon_demo_choice_group.clone()),
            )
            .child(div().pt(px(2.0)).text_size(px(11.0)).line_height(px(16.0)).text_color(chrome.muted_text).child(
                format!("Workspace action: {} | Icon demo: {}", self.workspace_action, self.workspace_icon_demo),
            ))
            .into_any_element()
    }

    fn render_system_card(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        card_container(chrome.border, chrome.panel_background)
            .child(card_title(
                "System & Preferences",
                "Choice controls plus progress feedback.",
                chrome.title_text,
                chrome.muted_text,
            ))
            .child(self.system.two_factor_switch.clone())
            .child(self.system.terms_checkbox.clone())
            .child(self.system.social_checkbox.clone())
            .child(self.system.referral_checkbox.clone())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .line_height(px(16.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child(format!("Budget: {:.0}%", self.budget)),
                    )
                    .child(self.system.budget_slider.clone()),
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
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child(format!("Profile completion: {:.0}%", self.completion)),
                    )
                    .child(self.system.completion_progress.clone()),
            )
            .into_any_element()
    }

    // ===== Event handling =====

    fn handle_button_event(&mut self, button: IntroButton, _event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match button {
            IntroButton::Submit => {
                self.clicks_submit += 1;
                self.last_event = SharedString::from("Button::Submit");
            }
            IntroButton::Cancel => {
                self.clicks_cancel += 1;
                self.last_event = SharedString::from("Button::Cancel");
            }
        }

        cx.notify();
    }

    fn handle_popup_menu_event(&mut self, event: &PopupMenuEvent, cx: &mut Context<GalleryApp>) {
        match event {
            PopupMenuEvent::Select { label, .. } => {
                self.workspace_action = label.clone();
                self.last_event = SharedString::from("PopupMenu::Select");
            }
        }

        cx.notify();
    }

    fn handle_workspace_layout_event(&mut self, event: &ChoiceGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ChoiceGroupEvent::Change { label, selected_ids, selected, .. } => {
                self.workspace_layout = if *selected {
                    label.clone()
                } else {
                    SharedString::from("None")
                };
                self.last_event = SharedString::from("ChoiceGroup::Layout");

                let next_selected_ids = selected_ids.clone();
                self.workspace
                    .workspace_layout_choice_group
                    .update(cx, move |group, cx| group.set_managed_selected_ids(next_selected_ids.clone(), cx));
            }
        }

        cx.notify();
    }

    fn handle_workspace_icon_demo_event(&mut self, event: &ChoiceGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ChoiceGroupEvent::Change { label, selected_ids, selected, .. } => {
                self.workspace_icon_demo = if *selected {
                    label.clone()
                } else {
                    SharedString::from("None")
                };
                self.last_event = SharedString::from("ChoiceGroup::IconDemo");

                let next_selected_ids = selected_ids.clone();
                self.workspace
                    .workspace_icon_demo_choice_group
                    .update(cx, move |group, cx| group.set_managed_selected_ids(next_selected_ids.clone(), cx));
            }
        }

        cx.notify();
    }

    fn handle_workspace_density_event(&mut self, event: &ChoiceGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ChoiceGroupEvent::Change { label, .. } => {
                self.workspace_density = label.clone();
                self.last_event = SharedString::from("ChoiceGroup::Density");
            }
        }

        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_textfield_event(&mut self, field: IntroField, event: &TextFieldEvent, cx: &mut Context<GalleryApp>) {
        match event {
            TextFieldEvent::Change { value } | TextFieldEvent::Submit { value } => {
                let value: SharedString = value.clone().into();
                match field {
                    IntroField::Name => self.name_value = value,
                    IntroField::Email => self.email_value = value,
                    IntroField::Card => self.card_value = value,
                }
                self.last_event = SharedString::from("TextField::Change");
                self.recompute_completion(cx);
            }
            TextFieldEvent::Focus => {}
            TextFieldEvent::Blur => {}
        }

        cx.notify();
    }

    fn handle_same_as_shipping_event(&mut self, _event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        self.same_as_shipping = !self.same_as_shipping;
        self.payment.same_as_shipping_checkbox.update(cx, |b, cx| b.set_data(self.same_as_shipping, cx));
        self.last_event = SharedString::from("Checkbox::SameAsShipping");
        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_terms_event(&mut self, _event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        self.accepted_terms = !self.accepted_terms;
        self.system.terms_checkbox.update(cx, |b, cx| b.set_data(self.accepted_terms, cx));
        self.last_event = SharedString::from("Checkbox::Terms");
        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_social_event(&mut self, _event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        self.social_source = !self.social_source;
        self.system.social_checkbox.update(cx, |b, cx| b.set_data(self.social_source, cx));
        self.last_event = SharedString::from("Checkbox::Social");
        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_referral_event(&mut self, _event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        self.referral_source = !self.referral_source;
        self.system.referral_checkbox.update(cx, |b, cx| b.set_data(self.referral_source, cx));
        self.last_event = SharedString::from("Checkbox::Referral");
        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_two_factor_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.system.two_factor_switch.update(cx, |button, cx| {
                    let new_on = !*button.data();
                    button.set_data(new_on, cx);
                });
                self.two_factor_enabled = !self.two_factor_enabled;
                self.last_event = SharedString::from("Switch::TwoFactor");
                self.recompute_completion(cx);
                cx.notify();
            }
        }
    }

    fn handle_budget_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        let SliderEvent::Change { value } = event;
        self.budget = *value;
        self.last_event = SharedString::from("Slider::Budget");
        self.recompute_completion(cx);
        cx.notify();
    }

    // ===== Derived state =====

    fn recompute_completion(&mut self, cx: &mut Context<GalleryApp>) {
        let mut score = 0.0f32;
        let mut max_score = 0.0f32;

        max_score += 1.0;
        if !self.name_value.is_empty() {
            score += 1.0;
        }

        max_score += 1.0;
        if !self.email_value.is_empty() {
            score += 1.0;
        }

        max_score += 1.0;
        if !self.card_value.is_empty() {
            score += 1.0;
        }

        max_score += 1.0;
        if self.accepted_terms {
            score += 1.0;
        }

        max_score += 1.0;
        if self.two_factor_enabled {
            score += 1.0;
        }

        max_score += 1.0;
        if self.social_source || self.referral_source {
            score += 1.0;
        }

        max_score += 1.0;
        if !self.workspace_density.is_empty() && self.workspace_density.as_ref() != "None" {
            score += 1.0;
        }

        max_score += 1.0;
        score += self.budget / 100.0;

        self.completion = if max_score > 0.0 {
            (score / max_score) * 100.0
        } else {
            0.0
        };

        let completion = self.completion;
        self.system
            .completion_progress
            .update(cx, move |progress, cx| progress.set_value(completion as f64, cx));
    }
}

fn card_container(border: gpui::Hsla, panel: gpui::Hsla) -> gpui::Div {
    div()
        .w(px(360.0))
        .max_w_full()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .border_1()
        .border_color(border)
        .rounded(px(12.0))
        .bg(panel)
        .p(px(14.0))
}

fn card_title(
    title: &'static str,
    subtitle: &'static str,
    title_color: gpui::Hsla,
    subtitle_color: gpui::Hsla,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .text_size(px(18.0))
                .line_height(px(24.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(title),
        )
        .child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(subtitle_color).child(subtitle))
        .into_any_element()
}

fn workspace_layout_items() -> [ChoiceGroupItem; 4] {
    [
        ChoiceGroupItem::new("grid", "grid").label("Grid"),
        ChoiceGroupItem::new("list", "list").label("List"),
        ChoiceGroupItem::new("kanban", "kanban").label("Kanban"),
        ChoiceGroupItem::new("another", "another").label("Another"),
    ]
}

fn workspace_density_items() -> [ChoiceGroupItem; 3] {
    [
        ChoiceGroupItem::new("compact", "compact").label("Compact"),
        ChoiceGroupItem::new("balanced", "balanced").label("Balanced"),
        ChoiceGroupItem::new("comfortable", "comfortable").label("Comfortable"),
    ]
}

fn workspace_icon_demo_items() -> [ChoiceGroupItem; 3] {
    [
        ChoiceGroupItem::new("left", "left").label("Left"),
        ChoiceGroupItem::new("center", "center").label("Center"),
        ChoiceGroupItem::new("right", "right").label("Right"),
    ]
}

fn workspace_menu_items() -> [MenuItem; 5] {
    [
        MenuItem::new("sync-now").label("Sync now").icon(LucideIcon::RefreshCw),
        MenuItem::new("share-workspace").label("Share workspace").icon(LucideIcon::Share2),
        MenuItem::new("duplicate").label("Duplicate").icon(LucideIcon::Copy),
        MenuItem::new("move").label("Move to…").icon(LucideIcon::FolderInput).submenu([
            MenuItem::new("team-space").label("Team Space").icon(LucideIcon::Users),
            MenuItem::new("archive-space").label("Archive").icon(LucideIcon::Archive),
        ]),
        MenuItem::new("delete").label("Delete").icon(LucideIcon::Trash2),
    ]
}
