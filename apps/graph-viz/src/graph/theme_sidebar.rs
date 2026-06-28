use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use gpui_luma::controls::selector::{Selector, SelectorItem};
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};

use crate::theme::available_themes;

pub struct ThemeSidebar {
    look: std::sync::Arc<ShadcnLook>,
    theme_selector: Entity<Selector>,
}

impl ThemeSidebar {
    pub fn new(
        look: std::sync::Arc<ShadcnLook>,
        active_theme_id: impl Into<gpui::SharedString>,
        cx: &mut Context<Self>,
    ) -> Self {
        let active_theme_id = active_theme_id.into();
        let theme_selector = look
            .selector("graph-viz-theme-selector")
            .label("Theme")
            .items(theme_selector_items())
            .selected_id(active_theme_id)
            .spawn(cx);

        Self { look, theme_selector }
    }

    pub fn theme_selector(&self) -> Entity<Selector> {
        self.theme_selector.clone()
    }

    pub fn apply_theme_snapshot(&mut self, look: std::sync::Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        self.sync_theme_selector_template(cx);
        cx.notify();
    }

    pub fn sync_theme_selector(&mut self, active_theme_id: impl Into<gpui::SharedString>, cx: &mut Context<Self>) {
        let active_theme_id = active_theme_id.into();
        self.theme_selector.update(cx, |selector, cx| {
            selector.set_selected_id(active_theme_id, cx);
        });
    }

    fn sync_theme_selector_template(&self, cx: &mut Context<Self>) {
        let theme = self.look.clone();
        self.theme_selector.update(cx, |selector, cx| {
            selector.set_template(theme.selector_template(), cx);
        });
    }
}

fn theme_selector_items() -> Vec<SelectorItem> {
    let mut items = vec![SelectorItem::new("default").label("Default")];
    for theme in available_themes() {
        items.push(SelectorItem::new(theme.id).label(theme.display_name()));
    }
    items
}

impl Render for ThemeSidebar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let sidebar_bg = self.look.token_color("sidebar").unwrap_or(chrome.panel_background);

        div()
            .id("graph-viz-theme-sidebar")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(sidebar_bg)
            .child(
                div()
                    .id("graph-viz-theme-selector-shell")
                    .w_full()
                    .h(px(48.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .px(px(24.0))
                    .border_b_1()
                    .border_color(chrome.border)
                    .child(self.theme_selector.clone()),
            )
            .child(div().flex_1().min_h_0())
    }
}
