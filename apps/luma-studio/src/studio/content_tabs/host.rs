use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::tabs_navigation::{
    TabsNavigation, TabsNavigationEvent, TabsNavigationItem, TabsNavigationWidthMode,
};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn::prelude::*;

use super::cards::render_demo_board;
use super::controls;
use super::dashboard;
use super::navigation::luma_studio_tabs_navigation_template;
use super::palette;
use super::style_guide;
use super::tab::ContentTab;
use super::theme_usage;
use super::super::app::LumaStudioApp;
use super::super::controls::ControlsPanel;
use super::super::demo_controls::DemoControls;
use super::super::overrides::StudioOverrides;
use super::super::panels::{PalettePanel, ThemeUsagePanel};
use super::super::style::StyleGuidePanel;

/// Cached board state — `ContentPaneHost::render` must not read `LumaStudioApp` (re-entrancy panic).
#[derive(Clone)]
pub struct BoardSnapshot {
    pub demos: DemoControls,
    pub look: Arc<ShadcnLook>,
    pub overrides: StudioOverrides,
}

pub struct ContentPaneHost {
    tabs: Entity<TabsNavigation>,
    style_guide_panel: Entity<StyleGuidePanel>,
    controls_panel: Entity<ControlsPanel>,
    palette_panel: Entity<PalettePanel>,
    theme_usage_panel: Entity<ThemeUsagePanel>,
    board: BoardSnapshot,
    active_tab: ContentTab,
    _subscriptions: Vec<Subscription>,
}

impl ContentPaneHost {
    pub fn new(_app: Entity<LumaStudioApp>, board: BoardSnapshot, cx: &mut Context<Self>) -> Self {
        let tabs = board
            .look
            .tabs_navigation("luma-studio-content-tabs")
            .size(ControlSize::Lg)
            .width_mode(TabsNavigationWidthMode::Uniform)
            .template(luma_studio_tabs_navigation_template(board.look.clone(), ControlSize::Lg))
            .items([
                TabsNavigationItem::new("cards").label("Cards"),
                TabsNavigationItem::new("dashboard").label("Dashboard"),
                TabsNavigationItem::new("typography").label("Style Guide"),
                TabsNavigationItem::new("controls").label("Controls"),
                TabsNavigationItem::new("palette").label("Palette"),
                TabsNavigationItem::new("theme-usage").label("Theme Usage"),
            ])
            .active("cards")
            .spawn(cx);

        let tabs_for_sub = tabs.clone();
        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&tabs_for_sub, |host, _, event: &TabsNavigationEvent, cx| {
            let TabsNavigationEvent::Activate { tab_id, .. } = event;
            if let Some(tab) = ContentTab::from_id(tab_id.as_ref()) {
                host.set_active_tab(tab, cx);
            }
        }));

        let style_guide_panel = cx.new(|cx| StyleGuidePanel::new(cx, board.look.clone()));
        let controls_panel = cx.new(|cx| ControlsPanel::new(cx, board.look.clone()));
        let palette_panel = cx.new(|cx| PalettePanel::new(cx, board.look.clone(), board.overrides.clone()));
        let theme_usage_panel = cx.new(|cx| ThemeUsagePanel::new(cx, board.look.clone()));

        Self {
            tabs,
            style_guide_panel,
            controls_panel,
            palette_panel,
            theme_usage_panel,
            board,
            active_tab: ContentTab::Cards,
            _subscriptions: subscriptions,
        }
    }

    pub fn set_active_tab(&mut self, tab: ContentTab, cx: &mut Context<Self>) {
        if self.active_tab == tab {
            return;
        }
        self.active_tab = tab;

        if tab == ContentTab::Typography {
            let look = self.board.look.clone();
            self.style_guide_panel.update(cx, |panel, cx| panel.sync_snapshot(look, cx));
        }

        if tab == ContentTab::Controls {
            let look = self.board.look.clone();
            self.controls_panel.update(cx, |panel, cx| panel.sync_snapshot(look, cx));
        }

        if tab == ContentTab::Palette {
            let look = self.board.look.clone();
            let overrides = self.board.overrides.clone();
            self.palette_panel.update(cx, |panel, cx| panel.sync_snapshot(look, overrides, cx));
        }

        if tab == ContentTab::ThemeUsage {
            let look = self.board.look.clone();
            self.theme_usage_panel.update(cx, |panel, cx| panel.sync_snapshot(look, cx));
        }
        cx.notify();
    }

    pub fn notify_tabs(&self, cx: &mut Context<Self>) {
        self.tabs.update(cx, |_, cx| cx.notify());
    }

    pub fn sync_board_snapshot(&mut self, board: BoardSnapshot, cx: &mut Context<Self>) {
        let look = board.look.clone();
        let overrides = board.overrides.clone();
        self.board = board;
        self.tabs.update(cx, |tabs, cx| {
            tabs.set_size(ControlSize::Lg, cx);
            tabs.set_width_mode(TabsNavigationWidthMode::Uniform, cx);
            tabs.set_template(luma_studio_tabs_navigation_template(look.clone(), ControlSize::Lg), cx);
        });
        self.style_guide_panel.update(cx, |panel, cx| panel.sync_snapshot(look.clone(), cx));
        self.controls_panel.update(cx, |panel, cx| panel.sync_snapshot(look.clone(), cx));
        self.palette_panel.update(cx, |panel, cx| panel.sync_snapshot(look.clone(), overrides, cx));
        self.theme_usage_panel.update(cx, |panel, cx| panel.sync_snapshot(look, cx));
        cx.notify();
    }
}

impl Render for ContentPaneHost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let board = &self.board;
        let chrome = board.look.chrome();
        let board_bg = board.look.token_color("background").unwrap_or(chrome.app_background);
        let active_tab = self.active_tab;

        div()
            .id("luma-studio-content-pane")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(board_bg)
            .child(div().flex_shrink_0().pt(px(8.0)).child(div().w_full().child(self.tabs.clone())))
            .child(match active_tab {
                ContentTab::Cards => {
                    scrollable_body().child(
                        div().id("luma-studio-cards-content").flex_1().min_h_0().overflow_y_scroll().child(
                            div().p(px(24.0)).child(render_demo_board(board.demos.clone(), active_tab.panels())),
                        ),
                    )
                }
                ContentTab::Dashboard => dashboard::viewport().child(board.demos.dashboard.clone()),
                ContentTab::Typography => style_guide::viewport().child(self.style_guide_panel.clone()),
                ContentTab::Controls => controls::viewport().child(self.controls_panel.clone()),
                ContentTab::Palette => palette::viewport().child(self.palette_panel.clone()),
                ContentTab::ThemeUsage => theme_usage::viewport().child(self.theme_usage_panel.clone()),
            })
    }
}

fn scrollable_body() -> gpui::Div {
    div().flex_1().min_h_0().size_full().flex().flex_col()
}
