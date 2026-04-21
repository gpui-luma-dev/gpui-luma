use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, px, rgb};
use gpui_luma::controls::popup_menu::{PopupMenu, PopupMenuEvent, PopupMenuItem, PopupMenuPlacement};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

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
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            popup_smart: PopupMenu::new("popup-menu-smart-example")
                .label("Smart popup")
                .items(menu_items())
                .placement(PopupMenuPlacement::Smart)
                .spawn(cx),
            popup_below: PopupMenu::new("popup-menu-below-example")
                .label("Below popup")
                .items(menu_items())
                .placement(PopupMenuPlacement::BelowStart)
                .spawn(cx),
            popup_above: PopupMenu::new("popup-menu-above-example")
                .label("Above popup")
                .items(menu_items())
                .placement(PopupMenuPlacement::AboveStart)
                .spawn(cx),
            popup_centered: PopupMenu::new("popup-menu-centered-example")
                .label("Centered popup")
                .items(menu_items())
                .placement(PopupMenuPlacement::CenteredOnTrigger)
                .spawn(cx),
            disabled_popup: PopupMenu::new("disabled-popup-menu-example")
                .label("Disabled popup")
                .items(disabled_menu_items())
                .enabled(false)
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

    pub(in crate::gallery) fn render(&self) -> AnyElement {
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
            .child(div().text_size(px(20.0)).line_height(px(28.0)).text_color(rgb(0x0f172a)).child("Popup Menu"))
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
                    .child(div().text_color(rgb(0x334155)).child(format!("Selected: {}", self.selection))),
            )
            .child(div().w_full().flex().items_center().justify_center().pb(px(8.0)).child(self.popup_smart.clone()))
            .into_any_element()
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

fn menu_items() -> [PopupMenuItem; 5] {
    [
        PopupMenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        PopupMenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        PopupMenuItem::new("archive").label("Archive"),
        PopupMenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
            PopupMenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
            PopupMenuItem::new("email").label("Email").icon(LucideIcon::Mail),
        ]),
        PopupMenuItem::new("disabled").label("Unavailable").icon(LucideIcon::ArchiveX).enabled(false),
    ]
}

fn disabled_menu_items() -> [PopupMenuItem; 3] {
    [
        PopupMenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        PopupMenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        PopupMenuItem::new("archive").label("Archive"),
    ]
}
