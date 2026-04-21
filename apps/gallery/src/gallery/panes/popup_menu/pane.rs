use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::popup_menu::{PopupMenu, PopupMenuEvent, PopupMenuPlacement};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::notify_entity;

#[derive(Clone)]
pub(in crate::gallery) struct PopupMenuPane {
    popup_smart: Entity<PopupMenu>,
    popup_below: Entity<PopupMenu>,
    popup_above: Entity<PopupMenu>,
    popup_centered: Entity<PopupMenu>,
    disabled_popup: Entity<PopupMenu>,
    selection: String,
}

impl PopupMenuPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            popup_smart: PopupMenu::new("popup-menu-smart-example")
                .label("Smart popup")
                .items(menu_items())
                .placement(PopupMenuPlacement::Smart)
                .template(theme.popup_menu_template())
                .spawn(cx),
            popup_below: PopupMenu::new("popup-menu-below-example")
                .label("Below popup")
                .items(menu_items())
                .placement(PopupMenuPlacement::BelowStart)
                .template(theme.popup_menu_template())
                .spawn(cx),
            popup_above: PopupMenu::new("popup-menu-above-example")
                .label("Above popup")
                .items(menu_items())
                .placement(PopupMenuPlacement::AboveStart)
                .template(theme.popup_menu_template())
                .spawn(cx),
            popup_centered: PopupMenu::new("popup-menu-centered-example")
                .label("Centered popup")
                .items(menu_items())
                .placement(PopupMenuPlacement::CenteredOnTrigger)
                .template(theme.popup_menu_template())
                .spawn(cx),
            disabled_popup: PopupMenu::new("disabled-popup-menu-example")
                .label("Disabled popup")
                .items(disabled_menu_items())
                .enabled(false)
                .template(theme.popup_menu_template())
                .spawn(cx),
            selection: "none".to_string(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.popup_smart, |app, _, event: &PopupMenuEvent, cx| {
            app.panes.popup_menu.handle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.popup_below, |app, _, event: &PopupMenuEvent, cx| {
            app.panes.popup_menu.handle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.popup_above, |app, _, event: &PopupMenuEvent, cx| {
            app.panes.popup_menu.handle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.popup_centered, |app, _, event: &PopupMenuEvent, cx| {
            app.panes.popup_menu.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .items_center()
            .overflow_hidden()
            .px(px(32.0))
            .pt(px(32.0))
            .pb(px(12.0))
            .bg(chrome.content_background)
            .child(div().text_size(px(20.0)).line_height(px(28.0)).text_color(chrome.title_text).child("Popup Menu"))
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(self.popup_below.clone())
                            .child(self.popup_above.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(self.popup_centered.clone())
                            .child(self.disabled_popup.clone()),
                    )
                    .child(div().text_color(chrome.body_text).child(format!("Selected: {}", self.selection))),
            )
            .child(div().w_full().flex().items_center().justify_center().pb(px(8.0)).child(self.popup_smart.clone()))
            .into_any_element()
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.popup_smart, cx);
        notify_entity(&self.popup_below, cx);
        notify_entity(&self.popup_above, cx);
        notify_entity(&self.popup_centered, cx);
        notify_entity(&self.disabled_popup, cx);
    }

    fn handle_event(&mut self, event: &PopupMenuEvent, cx: &mut Context<GalleryApp>) {
        match event {
            PopupMenuEvent::Select { label, .. } => {
                self.handle_selection(label, cx);
            }
        }
    }

    fn handle_selection(&mut self, label: &gpui::SharedString, cx: &mut Context<GalleryApp>) {
        self.selection = label.to_string();
        cx.notify();
    }
}

fn menu_items() -> [MenuItem; 5] {
    [
        MenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        MenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        MenuItem::new("archive").label("Archive"),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
            MenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
            MenuItem::new("email").label("Email").icon(LucideIcon::Mail),
        ]),
        MenuItem::new("disabled").label("Unavailable").icon(LucideIcon::ArchiveX).enabled(false),
    ]
}

fn disabled_menu_items() -> [MenuItem; 3] {
    [
        MenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        MenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        MenuItem::new("archive").label("Archive"),
    ]
}
