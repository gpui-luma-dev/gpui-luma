use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::context_menu::{ContextMenu, ContextMenuEvent};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::theme::RadixTheme;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::radial::radial_context_menu_template;
use super::super::shared::{gallery_pane_with_usage, notify_entity};
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
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let context_menu_theme = radix_theme.context_menu_theme();
        Self {
            default_context_menu: ContextMenu::new("context-menu-default-example")
                .label("Right-click me: Default")
                .items(default_context_menu_items())
                .template(gallery_context_menu_template(context_menu_theme.clone()))
                .spawn(cx),
            radial_context_menu: ContextMenu::new("context-menu-radial-example")
                .label("Right-click me: Radial")
                .items(radial_context_menu_items())
                .template(radial_context_menu_template(context_menu_theme))
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

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        let chrome = radix_theme.chrome();

        gallery_pane_with_usage(
            "Context Menu",
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
                    div().text_color(chrome.body_text).child(format!(
                        "Selected: default={}, radial={}",
                        self.default_selection, self.radial_selection
                    )),
                )
                .into_any_element(),
            radix_theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.default_context_menu, cx);
        notify_entity(&self.radial_context_menu, cx);
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

fn default_context_menu_items() -> [MenuItem; 4] {
    [
        MenuItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        MenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
        MenuItem::new("inspect").label("Inspect"),
        MenuItem::new("more").label("More").icon(LucideIcon::Ellipsis).submenu([
            MenuItem::new("download").label("Download").icon(LucideIcon::Download),
            MenuItem::new("external").label("Open externally").icon(LucideIcon::ExternalLink),
        ]),
    ]
}

fn radial_context_menu_items() -> [MenuItem; 5] {
    [
        MenuItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        MenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
        MenuItem::new("inspect").label("Inspect").icon(LucideIcon::ScanSearch),
        MenuItem::new("download").label("Download").icon(LucideIcon::Download),
        MenuItem::new("external").label("Open externally").icon(LucideIcon::ExternalLink),
    ]
}
