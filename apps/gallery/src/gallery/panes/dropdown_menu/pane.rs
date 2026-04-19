use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, rgb};
use gpui_luma::controls::dropdown_menu::{DropdownMenu, DropdownMenuEvent, DropdownMenuItem};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::super::shared::gallery_pane;

pub(in crate::gallery) struct DropdownMenuPane {
    dropdown_menu: Entity<DropdownMenu>,
    disabled_dropdown_menu: Entity<DropdownMenu>,
    selection: String,
}

impl DropdownMenuPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            dropdown_menu: DropdownMenu::new("dropdown-menu-example").label("Actions").items(menu_items()).spawn(cx),
            disabled_dropdown_menu: DropdownMenu::new("disabled-dropdown-menu-example")
                .label("Disabled actions")
                .items(disabled_menu_items())
                .enabled(false)
                .spawn(cx),
            selection: "none".to_string(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.dropdown_menu, |app, _, event: &DropdownMenuEvent, cx| {
            app.panes.dropdown_menu.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self) -> AnyElement {
        gallery_pane(
            "Dropdown Menu",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(self.dropdown_menu.clone())
                        .child(self.disabled_dropdown_menu.clone()),
                )
                .child(div().text_color(rgb(0x334155)).child(format!("Selected: {}", self.selection)))
                .into_any_element(),
        )
    }

    fn handle_event(&mut self, event: &DropdownMenuEvent, cx: &mut Context<GalleryApp>) {
        match event {
            DropdownMenuEvent::Select { label, .. } => {
                self.selection = label.to_string();
                cx.notify();
            }
        }
    }
}

fn menu_items() -> [DropdownMenuItem; 5] {
    [
        DropdownMenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        DropdownMenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        DropdownMenuItem::new("archive").label("Archive"),
        DropdownMenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
            DropdownMenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
            DropdownMenuItem::new("email").label("Email").icon(LucideIcon::Mail),
        ]),
        DropdownMenuItem::new("disabled").label("Unavailable").icon(LucideIcon::ArchiveX).enabled(false),
    ]
}

fn disabled_menu_items() -> [DropdownMenuItem; 3] {
    [
        DropdownMenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        DropdownMenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        DropdownMenuItem::new("archive").label("Archive"),
    ]
}
