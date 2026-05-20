use std::cell::Cell;
use std::sync::Arc;

use gpui::{Context, IntoElement, MouseButton, Window, div, prelude::*};
use gpui_luma::controls::checkbox;
use gpui_luma::controls::choice_group::{self, ChoiceGroupItem};
use gpui_luma::controls::combobox::{self, SelectionItem, TypingPolicy};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use gpui_luma::controls::command::button::{Button, ButtonKind};
use gpui_luma::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::popup_menu::{PopupMenu, PopupMenuPlacement};
use gpui_luma::controls::progress;
use gpui_luma::controls::radio_button;
use gpui_luma::controls::radio_group::{
    self as radio_group, RadioGroupItem, RadioGroupItemLike, RadioGroupRenderModel, RadioGroupTemplate,
    RadioGroupTemplateHandlers,
};
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

pub(in crate::gallery) fn build_workspace_panel(
    cx: &mut Context<GalleryApp>,
    theme: &GalleryThemePack,
) -> WorkspacePanel {
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
        workspace_density_radio_group: radio_group::horizontal("intro-workspace-density")
            .items(workspace_density_items())
            .selected("balanced")
            .template(workspace_density_template(theme.radio_button_template()))
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

fn workspace_density_items() -> [RadioGroupItem; 3] {
    [
        RadioGroupItem::new("compact").label("Compact"),
        RadioGroupItem::new("balanced").label("Balanced"),
        RadioGroupItem::new("comfortable").label("Comfortable"),
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

fn workspace_density_template(button_template: Arc<dyn ButtonTemplate<bool>>) -> RadioGroupTemplate<RadioGroupItem> {
    Arc::new(move |model, handlers, window, cx| {
        render_workspace_density_group(model, handlers, &button_template, window, cx)
    })
}

fn render_workspace_density_group(
    model: &RadioGroupRenderModel<'_, RadioGroupItem>,
    handlers: RadioGroupTemplateHandlers,
    button_template: &Arc<dyn ButtonTemplate<bool>>,
    window: &mut Window,
    cx: &mut gpui::App,
) -> gpui::Stateful<gpui::Div> {
    let RadioGroupTemplateHandlers { item_hovers, item_mouse_downs, item_mouse_ups, item_mouse_up_outs, item_clicks } =
        handlers;

    let mut item_hovers = item_hovers.into_iter();
    let mut item_mouse_downs = item_mouse_downs.into_iter();
    let mut item_mouse_ups = item_mouse_ups.into_iter();
    let mut item_mouse_up_outs = item_mouse_up_outs.into_iter();
    let mut item_clicks = item_clicks.into_iter();

    let mut root = div().id(model.id.clone()).flex().items_center().gap_3();

    for item in &model.items {
        let Some(item_hover) = item_hovers.next() else {
            break;
        };
        let Some(item_mouse_down) = item_mouse_downs.next() else {
            break;
        };
        let Some(item_mouse_up) = item_mouse_ups.next() else {
            break;
        };
        let Some(item_mouse_up_out) = item_mouse_up_outs.next() else {
            break;
        };
        let Some(item_click) = item_clicks.next() else {
            break;
        };

        let render_model = ButtonRenderModel {
            id: format!("{}-{}", model.id, RadioGroupItemLike::id(item.item)).into(),
            data: item.selected,
            content: Arc::new({
                let label = RadioGroupItemLike::label(item.item).clone();
                move |_, _| div().child(label.clone()).into_any_element()
            }),
            kind: ButtonKind::Standard,
            role: ButtonFamilyRole::Text,
            size: ButtonSize::Md,
            state: item.state.interaction_state(),
            round: false,
            radius_override: Cell::new(None),
        };

        let mut button = button_template
            .render(&render_model, window, cx)
            .on_hover(item_hover)
            .on_mouse_down(MouseButton::Left, item_mouse_down)
            .on_mouse_up(MouseButton::Left, item_mouse_up)
            .on_mouse_up_out(MouseButton::Left, item_mouse_up_out)
            .on_click(item_click);

        if item.enabled {
            button = button.cursor_pointer();
        } else {
            button = button.opacity(0.56);
        }

        root = root.child(button);
    }

    root
}
