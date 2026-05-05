use gpui::{Context, IntoElement, div, prelude::*, px};
use gpui_luma::controls::checkbox;
use gpui_luma::controls::choice_group::{self, ChoiceGroupItem};
use gpui_luma::controls::combobox::{self, SelectionItem, TypingPolicy};
use gpui_luma::controls::command::button::{Button, ButtonKind};
use gpui_luma::controls::content_presenter::HasContent;
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::popup_menu::{PopupMenu, PopupMenuPlacement};
use gpui_luma::controls::progress;
use gpui_luma::controls::radio_button;
use gpui_luma::controls::slider;
use gpui_luma::controls::switch;
use gpui_luma::controls::textfield;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::pane::{PaymentPanel, SystemPanel, WorkspacePanel};

pub(in crate::gallery) fn build_payment_panel(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> PaymentPanel {
    let checkbox_template = theme.checkbox_template();

    PaymentPanel {
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
        payment_combobox: combobox::new("intro-payment-combobox", payment_method_items())
            .placeholder("Select payment method…")
            .full_width(true)
            .clean_on_escape(true)
            .typing_policy(TypingPolicy::Strict)
            .show_down_arrow(true)
            .show_clear_button(false)
            .textfield_template(theme.textfield_template())
            .scrollbar_template(theme.scrollbar_template())
            .spawn(cx),
        same_as_shipping_checkbox: checkbox::new("intro-same-as-shipping")
            .data(true)
            .content(|_, _| div().child("Same as shipping address").into_any_element())
            .template(checkbox_template.clone())
            .spawn(cx),
        payment_method_radio: radio_button::new("intro-payment-method-radio")
            .data(true)
            .content(|_, _| div().child("Use this as default payment method").into_any_element())
            .template(theme.radio_button_template())
            .spawn(cx),
    }
}

pub(in crate::gallery) fn build_workspace_panel(cx: &mut Context<GalleryApp>) -> WorkspacePanel {
    WorkspacePanel {
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
        workspace_density_choice_group: choice_group::single_select("intro-workspace-density")
            .items(workspace_density_items())
            .selected("balanced")
            .bool_button_template_factory(|_| {
                std::sync::Arc::new(
                    radio_button::ThemedRadioButtonTemplate::new(
                        gpui_luma::controls::radio_button::default_radio_button_theme(),
                    )
                    .with_modifier(|element, _| element.min_h(px(22.0)).py(px(0.0)).px(px(2.0))),
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
    }
}

pub(in crate::gallery) fn build_system_panel(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> SystemPanel {
    let checkbox_template = theme.checkbox_template();

    SystemPanel {
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
    }
}

fn payment_method_items() -> Vec<SelectionItem> {
    vec![
        SelectionItem::new("visa", "Visa •••• 4242"),
        SelectionItem::new("mastercard", "Mastercard •••• 4444"),
        SelectionItem::new("amex", "Amex •••• 0005"),
        SelectionItem::new("apple-pay", "Apple Pay"),
        SelectionItem::new("google-pay", "Google Pay"),
    ]
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
