use gpui::{AnyElement, Context, Entity, FontWeight, IntoElement, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent, ButtonKind};
use gpui_luma::controls::checkbox::{Checkbox, CheckboxEvent};
use gpui_luma::controls::command::icon_button::{IconButton, IconButtonEvent, IconButtonKind};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::popup_menu::{PopupMenu, PopupMenuEvent, PopupMenuPlacement};
use gpui_luma::controls::progress::Progress;
use gpui_luma::controls::radio_group::{RadioGroup, RadioGroupEvent, RadioGroupItem};
use gpui_luma::controls::slider::{Slider, SliderEvent};
use gpui_luma::controls::switch::{Switch, SwitchEvent};
use gpui_luma::controls::textfield::{TextField, TextFieldEvent};
use gpui_luma::controls::toggle_group::{ToggleGroup, ToggleGroupEvent, ToggleGroupItem};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_description, notify_entity};

const INTRO_DESCRIPTION: &str = concat!(
    "A control-dense landing page built with real interactive gpui-luma controls. ",
    "Use it as a compositional reference for form-heavy product screens."
);

#[derive(Clone)]
pub(in crate::gallery) struct IntroductionPane {
    submit_button: Entity<Button>,
    cancel_button: Entity<Button>,

    refresh_icon_button: Entity<IconButton>,
    favorite_icon_button: Entity<IconButton>,
    workspace_popup_menu: Entity<PopupMenu>,
    workspace_layout_toggle_group: Entity<ToggleGroup>,
    workspace_density_radio_group: Entity<RadioGroup>,

    name_field: Entity<TextField>,
    email_field: Entity<TextField>,
    card_field: Entity<TextField>,

    same_as_shipping_checkbox: Entity<Checkbox>,
    terms_checkbox: Entity<Checkbox>,
    social_checkbox: Entity<Checkbox>,
    referral_checkbox: Entity<Checkbox>,

    two_factor_switch: Entity<Switch>,
    budget_slider: Entity<Slider>,
    completion_progress: Entity<Progress>,

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
    workspace_action: SharedString,

    budget: f32,
    completion: f32,
    clicks_submit: usize,
    clicks_cancel: usize,
    clicks_refresh: usize,
    clicks_favorite: usize,
    last_event: SharedString,
}

#[derive(Clone, Copy)]
enum IntroButton {
    Submit,
    Cancel,
}

#[derive(Clone, Copy)]
enum IntroIconButton {
    Refresh,
    Favorite,
}

#[derive(Clone, Copy)]
enum IntroField {
    Name,
    Email,
    Card,
}

impl IntroductionPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, _theme: &GalleryThemePack) -> Self {
        Self {
            submit_button: Button::new("intro-submit").label("Submit").kind(ButtonKind::Prominent).spawn(cx),
            cancel_button: Button::new("intro-cancel").label("Cancel").spawn(cx),

            refresh_icon_button: IconButton::new("intro-refresh-workspace", LucideIcon::RefreshCw)
                .kind(IconButtonKind::Prominent)
                .spawn(cx),
            favorite_icon_button: IconButton::new("intro-favorite-workspace", LucideIcon::Star).spawn(cx),
            workspace_popup_menu: PopupMenu::new("intro-workspace-popup")
                .label("Workspace Menu")
                .items(workspace_menu_items())
                .placement(PopupMenuPlacement::BelowStart)
                .spawn(cx),
            workspace_layout_toggle_group: ToggleGroup::new("intro-workspace-layout")
                .items(workspace_layout_items())
                .selected("grid")
                .spawn(cx),
            workspace_density_radio_group: RadioGroup::new("intro-workspace-density")
                .items(workspace_density_items())
                .selected("balanced")
                .spawn(cx),

            name_field: TextField::new("intro-name")
                .placeholder("Name on card")
                .full_width(true)
                .clean_on_escape(true)
                .spawn(cx),
            email_field: TextField::new("intro-email")
                .placeholder("Email address")
                .full_width(true)
                .clean_on_escape(true)
                .spawn(cx),
            card_field: TextField::new("intro-card")
                .placeholder("1234 5678 9012 3456")
                .full_width(true)
                .clean_on_escape(true)
                .spawn(cx),

            same_as_shipping_checkbox: Checkbox::new("intro-same-as-shipping")
                .label("Same as shipping address")
                .checked(true)
                .spawn(cx),
            terms_checkbox: Checkbox::new("intro-terms").label("I agree to the terms and conditions").spawn(cx),
            social_checkbox: Checkbox::new("intro-social-source").label("Social").checked(true).spawn(cx),
            referral_checkbox: Checkbox::new("intro-referral-source").label("Referral").spawn(cx),

            two_factor_switch: Switch::new("intro-two-factor").label("Two-factor authentication").spawn(cx),
            budget_slider: Slider::new("intro-budget").range(0..100).step(5).value(40).spawn(cx),
            completion_progress: Progress::new("intro-completion").range(0..100).value(30).spawn(cx),

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
            workspace_action: SharedString::from("None"),

            budget: 40.0,
            completion: 30.0,
            clicks_submit: 0,
            clicks_cancel: 0,
            clicks_refresh: 0,
            clicks_favorite: 0,
            last_event: SharedString::from("Ready"),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.submit_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_button_event(IntroButton::Submit, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.cancel_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_button_event(IntroButton::Cancel, event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.refresh_icon_button, |app, _, event: &IconButtonEvent, cx| {
            app.panes.introduction.handle_icon_button_event(IntroIconButton::Refresh, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.favorite_icon_button, |app, _, event: &IconButtonEvent, cx| {
            app.panes.introduction.handle_icon_button_event(IntroIconButton::Favorite, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.workspace_popup_menu, |app, _, event: &PopupMenuEvent, cx| {
            app.panes.introduction.handle_popup_menu_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(
            &self.workspace_layout_toggle_group,
            |app, _, event: &ToggleGroupEvent, cx| {
                app.panes.introduction.handle_workspace_layout_event(event, cx);
            },
        ));
        subscriptions.push(cx.subscribe(&self.workspace_density_radio_group, |app, _, event: &RadioGroupEvent, cx| {
            app.panes.introduction.handle_workspace_density_event(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.name_field, |app, _, event: &TextFieldEvent, cx| {
            app.panes.introduction.handle_textfield_event(IntroField::Name, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.email_field, |app, _, event: &TextFieldEvent, cx| {
            app.panes.introduction.handle_textfield_event(IntroField::Email, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.card_field, |app, _, event: &TextFieldEvent, cx| {
            app.panes.introduction.handle_textfield_event(IntroField::Card, event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.same_as_shipping_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.introduction.handle_same_as_shipping_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.terms_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.introduction.handle_terms_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.social_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.introduction.handle_social_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.referral_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.introduction.handle_referral_event(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.two_factor_switch, |app, _, event: &SwitchEvent, cx| {
            app.panes.introduction.handle_two_factor_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.budget_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.introduction.handle_budget_event(event, cx);
        }));
    }

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
                        .child(
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
                                        .text_color(chrome.title_text)
                                        .child("Foundation Controls for GPUI"),
                                )
                                .child(
                                    div()
                                        .text_size(px(17.0))
                                        .line_height(px(26.0))
                                        .text_color(chrome.body_text)
                                        .child(
                                            "A real, interactive introduction screen that combines command, input, choice, and feedback controls in one composition.",
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .flex_wrap()
                                .justify_center()
                                .items_stretch()
                                .gap(px(16.0))
                                .child(self.render_payment_card(theme))
                                .child(self.render_workspace_card(theme))
                                .child(self.render_system_card(theme)),
                        ),
                )
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.submit_button, cx);
        notify_entity(&self.cancel_button, cx);

        notify_entity(&self.refresh_icon_button, cx);
        notify_entity(&self.favorite_icon_button, cx);
        notify_entity(&self.workspace_popup_menu, cx);
        notify_entity(&self.workspace_layout_toggle_group, cx);
        notify_entity(&self.workspace_density_radio_group, cx);

        notify_entity(&self.name_field, cx);
        notify_entity(&self.email_field, cx);
        notify_entity(&self.card_field, cx);

        notify_entity(&self.same_as_shipping_checkbox, cx);
        notify_entity(&self.terms_checkbox, cx);
        notify_entity(&self.social_checkbox, cx);
        notify_entity(&self.referral_checkbox, cx);

        notify_entity(&self.two_factor_switch, cx);
        notify_entity(&self.budget_slider, cx);
        notify_entity(&self.completion_progress, cx);
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
            .child(self.name_field.clone())
            .child(self.email_field.clone())
            .child(self.card_field.clone())
            .child(self.same_as_shipping_checkbox.clone())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(self.submit_button.clone())
                    .child(self.cancel_button.clone()),
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
                    .child(self.workspace_layout_toggle_group.clone()),
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
                    .child(self.workspace_density_radio_group.clone()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(self.refresh_icon_button.clone())
                    .child(self.favorite_icon_button.clone())
                    .child(self.workspace_popup_menu.clone()),
            )
            .child(div().pt(px(2.0)).text_size(px(11.0)).line_height(px(16.0)).text_color(chrome.muted_text).child(
                format!(
                    "Workspace action: {} | Icon clicks: refresh={}, favorite={}",
                    self.workspace_action, self.clicks_refresh, self.clicks_favorite
                ),
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
                            .font_weight(FontWeight::MEDIUM)
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
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child(format!("Profile completion: {:.0}%", self.completion)),
                    )
                    .child(self.completion_progress.clone()),
            )
            .child(div().pt(px(4.0)).text_size(px(11.0)).line_height(px(16.0)).text_color(chrome.muted_text).child(
                format!(
                    "Clicks: submit={}, cancel={} | Last event: {}",
                    self.clicks_submit, self.clicks_cancel, self.last_event
                ),
            ))
            .into_any_element()
    }

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

    fn handle_icon_button_event(
        &mut self,
        button: IntroIconButton,
        _event: &IconButtonEvent,
        cx: &mut Context<GalleryApp>,
    ) {
        match button {
            IntroIconButton::Refresh => {
                self.clicks_refresh += 1;
                self.last_event = SharedString::from("IconButton::Refresh");
            }
            IntroIconButton::Favorite => {
                self.clicks_favorite += 1;
                self.last_event = SharedString::from("IconButton::Favorite");
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

    fn handle_workspace_layout_event(&mut self, event: &ToggleGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ToggleGroupEvent::Change { label, selected, .. } => {
                self.workspace_layout = if *selected {
                    label.clone()
                } else {
                    SharedString::from("None")
                };
                self.last_event = SharedString::from("ToggleGroup::Layout");
            }
        }

        cx.notify();
    }

    fn handle_workspace_density_event(&mut self, event: &RadioGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            RadioGroupEvent::Change { label, .. } => {
                self.workspace_density = label.clone().unwrap_or_else(|| SharedString::from("None"));
                self.last_event = SharedString::from("RadioGroup::Density");
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

    fn handle_same_as_shipping_event(&mut self, event: &CheckboxEvent, cx: &mut Context<GalleryApp>) {
        let CheckboxEvent::Change { checked } = event;
        self.same_as_shipping = *checked;
        self.last_event = SharedString::from("Checkbox::SameAsShipping");
        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_terms_event(&mut self, event: &CheckboxEvent, cx: &mut Context<GalleryApp>) {
        let CheckboxEvent::Change { checked } = event;
        self.accepted_terms = *checked;
        self.last_event = SharedString::from("Checkbox::Terms");
        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_social_event(&mut self, event: &CheckboxEvent, cx: &mut Context<GalleryApp>) {
        let CheckboxEvent::Change { checked } = event;
        self.social_source = *checked;
        self.last_event = SharedString::from("Checkbox::Social");
        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_referral_event(&mut self, event: &CheckboxEvent, cx: &mut Context<GalleryApp>) {
        let CheckboxEvent::Change { checked } = event;
        self.referral_source = *checked;
        self.last_event = SharedString::from("Checkbox::Referral");
        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_two_factor_event(&mut self, event: &SwitchEvent, cx: &mut Context<GalleryApp>) {
        let SwitchEvent::Change { on } = event;
        self.two_factor_enabled = *on;
        self.last_event = SharedString::from("Switch::TwoFactor");
        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_budget_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        let SliderEvent::Change { value } = event;
        self.budget = *value;
        self.last_event = SharedString::from("Slider::Budget");
        self.recompute_completion(cx);
        cx.notify();
    }

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
        self.completion_progress.update(cx, move |progress, cx| progress.set_value(completion as f64, cx));
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

fn workspace_layout_items() -> [ToggleGroupItem; 4] {
    [
        ToggleGroupItem::new("grid").label("Grid"),
        ToggleGroupItem::new("list").label("List"),
        ToggleGroupItem::new("kanban").label("Kanban"),
        ToggleGroupItem::new("another").label("Another"),
    ]
}

fn workspace_density_items() -> [RadioGroupItem; 3] {
    [
        RadioGroupItem::new("compact").label("Compact"),
        RadioGroupItem::new("balanced").label("Balanced"),
        RadioGroupItem::new("comfortable").label("Comfortable"),
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
