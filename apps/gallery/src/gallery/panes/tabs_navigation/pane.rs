use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::tabs_navigation::{TabsNavigation, TabsNavigationEvent, TabsNavigationItem};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct TabsNavigationPane {
    tabs: Entity<TabsNavigation>,
    disabled_tabs: Entity<TabsNavigation>,
    active_label: String,
}

impl TabsNavigationPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            tabs: TabsNavigation::new("project-tabs")
                .items(project_tabs())
                .active("activity")
                .template(theme.tabs_navigation_template())
                .spawn(cx),
            disabled_tabs: TabsNavigation::new("disabled-project-tabs")
                .items(project_tabs())
                .active("activity")
                .enabled(false)
                .template(theme.tabs_navigation_template())
                .spawn(cx),
            active_label: "Activity".to_string(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.tabs, |app, _, event: &TabsNavigationEvent, cx| {
            app.panes.tabs_navigation.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        gallery_pane(
            "Tabs Navigation",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_3()
                        .child(self.tabs.clone())
                        .child(render_tab_content(&self.active_label, theme)),
                )
                .child(self.disabled_tabs.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.tabs, cx);
        notify_entity(&self.disabled_tabs, cx);
    }

    fn handle_event(&mut self, event: &TabsNavigationEvent, cx: &mut Context<GalleryApp>) {
        match event {
            TabsNavigationEvent::Activate { label, .. } => {
                self.active_label = label.to_string();
                cx.notify();
            }
        }
    }
}

fn project_tabs() -> [TabsNavigationItem; 4] {
    [
        TabsNavigationItem::new("overview").label("Overview"),
        TabsNavigationItem::new("activity").label("Activity"),
        TabsNavigationItem::new("metrics").label("Metrics"),
        TabsNavigationItem::new("settings").label("Settings").enabled(false),
    ]
}

fn render_tab_content(active_label: &str, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

    div()
        .w(px(360.0))
        .min_h(px(112.0))
        .flex()
        .flex_col()
        .gap_2()
        .rounded(px(8.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.panel_background)
        .p(px(16.0))
        .text_color(chrome.body_text)
        .child(
            div()
                .text_size(px(14.0))
                .line_height(px(20.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(format!("{active_label} tab")),
        )
        .child(
            div()
                .text_size(px(13.0))
                .line_height(px(18.0))
                .child("The pane stays responsible for focus after a tab activation."),
        )
        .into_any_element()
}
