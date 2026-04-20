use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, px, rgb};
use gpui_luma::controls::context_menu::{ContextMenu, ContextMenuEvent};
use gpui_luma::controls::dropdown_menu::DropdownMenuItem;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::radial::radial_context_menu_template;
use super::super::shared::gallery_pane;
use super::template::gallery_context_menu_template;

#[derive(Clone)]
pub(in crate::gallery) struct ContextMenuPane {
    default_context_menu: Entity<ContextMenu>,
    radial_context_menu: Entity<ContextMenu>,
    default_selection: String,
    radial_selection: String,
}

#[derive(Clone, Copy)]
enum ContextMenuPresentation {
    Default,
    Radial,
}

impl ContextMenuPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            default_context_menu: ContextMenu::new("context-menu-default-example")
                .label("Right-click me: Default")
                .items(default_context_menu_items())
                .template(gallery_context_menu_template())
                .spawn(cx),
            radial_context_menu: ContextMenu::new("context-menu-radial-example")
                .label("Right-click me: Radial")
                .items(radial_context_menu_items())
                .template(radial_context_menu_template())
                .spawn(cx),
            default_selection: "none".to_string(),
            radial_selection: "none".to_string(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.default_context_menu, |app, _, event: &ContextMenuEvent, cx| {
            app.panes.context_menu.handle_event(ContextMenuPresentation::Default, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.radial_context_menu, |app, _, event: &ContextMenuEvent, cx| {
            app.panes.context_menu.handle_event(ContextMenuPresentation::Radial, event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self) -> AnyElement {
        gallery_pane(
            "Context Menu",
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2()
                .min_h(px(220.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .justify_center()
                        .child(self.default_context_menu.clone())
                        .child(
                            div()
                                .min_w(px(260.0))
                                .min_h(px(220.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(self.radial_context_menu.clone()),
                        ),
                )
                .child(
                    div().text_color(rgb(0x334155)).child(format!(
                        "Selected: default={}, radial={}",
                        self.default_selection, self.radial_selection
                    )),
                )
                .into_any_element(),
        )
    }

    fn handle_event(
        &mut self,
        presentation: ContextMenuPresentation,
        event: &ContextMenuEvent,
        cx: &mut Context<GalleryApp>,
    ) {
        match event {
            ContextMenuEvent::Select { label, .. } => {
                match presentation {
                    ContextMenuPresentation::Default => {
                        self.default_selection = label.to_string();
                    }
                    ContextMenuPresentation::Radial => {
                        self.radial_selection = label.to_string();
                    }
                }
                cx.notify();
            }
        }
    }
}

fn default_context_menu_items() -> [DropdownMenuItem; 4] {
    [
        DropdownMenuItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        DropdownMenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
        DropdownMenuItem::new("inspect").label("Inspect"),
        DropdownMenuItem::new("more").label("More").icon(LucideIcon::Ellipsis).submenu([
            DropdownMenuItem::new("download").label("Download").icon(LucideIcon::Download),
            DropdownMenuItem::new("external").label("Open externally").icon(LucideIcon::ExternalLink),
        ]),
    ]
}

fn radial_context_menu_items() -> [DropdownMenuItem; 5] {
    [
        DropdownMenuItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        DropdownMenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
        DropdownMenuItem::new("inspect").label("Inspect").icon(LucideIcon::ScanSearch),
        DropdownMenuItem::new("download").label("Download").icon(LucideIcon::Download),
        DropdownMenuItem::new("external").label("Open externally").icon(LucideIcon::ExternalLink),
    ]
}
