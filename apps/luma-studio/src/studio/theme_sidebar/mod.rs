mod model;
pub(crate) mod panels;
mod parsing;
mod subscriptions;
mod sync;
mod theme_selector;

use std::sync::{Arc, RwLock};

use gpui::{Context, Entity, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::search_selector::SearchSelector;
use gpui_luma::controls::tabs_navigation::{
    TabsNavigation, TabsNavigationEvent, TabsNavigationItem, TabsNavigationWidthMode,
};

use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};

use self::model::{SidebarTab, TOKEN_CATEGORIES};
use self::panels::{ColorsPanel, OtherPanel, PanelContextMenuHost, TypographyPanel};
use self::parsing::token_color_with_fallback;
use self::theme_selector::{
    ThemeSelectorSwatchCache, render_theme_search_selector_item, theme_search_selector_panel_template,
    theme_search_selector_template, theme_selector_state,
};
use super::content_tabs::luma_studio_tabs_navigation_template;
use crate::studio::app::LumaStudioApp;
use crate::studio::overrides::StudioOverrides;

pub struct ThemeSidebar {
    look: std::sync::Arc<ShadcnLook>,
    global_overrides: std::collections::HashMap<String, gpui::Hsla>,
    theme_selector: SearchSelector,
    theme_selector_swatches: Arc<RwLock<ThemeSelectorSwatchCache>>,
    theme_selector_selected_id: Arc<RwLock<SharedString>>,
    tabs: Entity<TabsNavigation>,
    active_tab: SidebarTab,
    colors_panel: Entity<ColorsPanel>,
    colors_host: Entity<PanelContextMenuHost>,
    other_host: Entity<PanelContextMenuHost>,
    typography_panel: Entity<TypographyPanel>,
    other_panel: Entity<OtherPanel>,
    _subscriptions: Vec<Subscription>,
}

impl ThemeSidebar {
    pub fn new(
        _app: Entity<LumaStudioApp>,
        look: std::sync::Arc<ShadcnLook>,
        active_theme_id: impl Into<gpui::SharedString>,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) -> Self {
        let active_theme_id = active_theme_id.into();
        let global_overrides = overrides.global_color_overrides.clone();
        let colors_panel = cx.new(|cx| ColorsPanel::new(look.clone(), overrides, cx));
        let colors_target = colors_panel.clone();
        let colors_host = cx.new(|cx| {
            PanelContextMenuHost::new(
                "luma-studio-colors-context-menu",
                move |_| colors_target.clone(),
                panels::colors_context_menu_items(),
                look.clone(),
                cx,
            )
        });
        let typography_panel = cx.new(|cx| TypographyPanel::new(look.clone(), cx));
        let other_panel = cx.new(|cx| OtherPanel::new(look.clone(), overrides, cx));
        let other_target = other_panel.clone();
        let other_host = cx.new(|cx| {
            PanelContextMenuHost::new(
                "luma-studio-other-context-menu",
                move |_| other_target.clone(),
                panels::other_context_menu_items(),
                look.clone(),
                cx,
            )
        });
        let theme_selector_swatches = Arc::new(RwLock::new(ThemeSelectorSwatchCache::empty()));
        let theme_selector_selected_id = Arc::new(RwLock::new(active_theme_id.clone()));
        let (theme_items, swatch_cache) = theme_selector_state(look.as_ref());
        *theme_selector_swatches.write().expect("theme selector swatches lock") = swatch_cache;
        let swatches_for_template = theme_selector_swatches.clone();
        let theme_selector = look
            .search_selector("luma-studio-theme-selector", theme_items)
            .placeholder("Theme")
            .search_placeholder("Search themes...")
            .selected_id(active_theme_id)
            .full_width(true)
            .fill_popup_viewport(true)
            .template(theme_search_selector_template(
                &look,
                theme_selector_swatches.clone(),
                theme_selector_selected_id.clone(),
            ))
            .panel_template(theme_search_selector_panel_template())
            .with_item_template(move |model, cx| render_theme_search_selector_item(model, &swatches_for_template, cx))
            .spawn(cx);

        let tabs = look
            .tabs_navigation("luma-studio-sidebar-tabs")
            .size(ControlSize::Lg)
            .width_mode(TabsNavigationWidthMode::Uniform)
            .template(luma_studio_tabs_navigation_template(look.clone(), ControlSize::Lg))
            .items([
                TabsNavigationItem::new("colors").label("Colors"),
                TabsNavigationItem::new("typography").label("Typography"),
                TabsNavigationItem::new("other").label("Other"),
            ])
            .active("colors")
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&tabs, |sidebar, _, event: &TabsNavigationEvent, cx| {
            let TabsNavigationEvent::Activate { tab_id, .. } = event else {
                return;
            };
            if let Some(tab) = SidebarTab::from_id(tab_id.as_ref()) {
                sidebar.active_tab = tab;
                cx.notify();
            }
        }));

        Self {
            look,
            global_overrides,
            theme_selector,
            theme_selector_swatches,
            theme_selector_selected_id,
            tabs,
            active_tab: SidebarTab::Colors,
            colors_panel,
            colors_host,
            other_host,
            typography_panel,
            other_panel,
            _subscriptions: subscriptions,
        }
    }
}

pub(crate) fn palette_tokens() -> Vec<&'static str> {
    TOKEN_CATEGORIES.iter().flat_map(|(_, tokens)| tokens.iter().map(|(token, _)| *token)).collect()
}

impl Render for ThemeSidebar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let sidebar_bg =
            token_color_with_fallback(&self.look, &self.global_overrides, "sidebar", chrome.panel_background);

        let tab_body = match self.active_tab {
            SidebarTab::Colors => self.colors_host.clone().into_any_element(),
            SidebarTab::Typography => self.typography_panel.clone().into_any_element(),
            SidebarTab::Other => self.other_host.clone().into_any_element(),
        };

        div()
            .id("luma-studio-sidebar")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(sidebar_bg)
            .child(
                div()
                    .id("luma-studio-sidebar-theme-selector-shell")
                    .w_full()
                    .h(px(48.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .px(px(24.0))
                    .border_b_1()
                    .border_color(chrome.border)
                    .child(div().flex().items_center().w_full().h_full().child(self.theme_selector.clone())),
            )
            .child(div().flex_shrink_0().pt(px(8.0)).child(div().w_full().child(self.tabs.clone())))
            .child(
                div()
                    .id("luma-studio-sidebar-scroll")
                    .flex_1()
                    .min_h(px(0.0))
                    .w_full()
                    .overflow_y_scroll()
                    .p(px(12.0))
                    .child(tab_body),
            )
    }
}
