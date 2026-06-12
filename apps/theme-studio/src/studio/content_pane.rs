use std::collections::HashMap;
use std::sync::Arc;

use gpui::{
    Context, DragMoveEvent, Entity, MouseDownEvent, MouseUpEvent, Overflow, Point, Pixels, Render, Subscription,
    Window, div, prelude::*, px,
};
use gpui_luma::controls::tabs_navigation::{TabsNavigation, TabsNavigationEvent, TabsNavigationItem};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use super::app::ThemeStudioApp;
use super::demo_controls::DemoControls;
use super::inspectable::InspectableId;
use super::overrides::StudioOverrides;
use super::panel_layout::DemoPanelDrag;
use super::panels::{PalettePanel, render_demo_board};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ContentTab {
    #[default]
    Cards,
    Dashboard,
    Palette,
}

impl ContentTab {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "cards" => Some(Self::Cards),
            "dashboard" => Some(Self::Dashboard),
            "palette" => Some(Self::Palette),
            _ => None,
        }
    }

    pub fn panels(self) -> Option<&'static [InspectableId]> {
        match self {
            Self::Cards => Some(&InspectableId::CARDS),
            Self::Dashboard => Some(&InspectableId::DASHBOARD),
            Self::Palette => None,
        }
    }

    pub fn contains(self, id: InspectableId) -> bool {
        self.panels().is_some_and(|panels| panels.contains(&id))
    }
}

/// Cached board state — `ContentPaneHost::render` must not read `ThemeStudioApp` (re-entrancy panic).
#[derive(Clone)]
pub struct BoardSnapshot {
    pub selected: Option<InspectableId>,
    pub panel_positions: HashMap<InspectableId, Point<Pixels>>,
    pub panel_z_order: HashMap<InspectableId, u32>,
    pub demos: DemoControls,
    pub look: Arc<ShadcnLook>,
    pub overrides: StudioOverrides,
}

pub struct ContentPaneHost {
    app: Entity<ThemeStudioApp>,
    tabs: Entity<TabsNavigation>,
    palette_panel: Entity<PalettePanel>,
    board: BoardSnapshot,
    active_tab: ContentTab,
    _subscriptions: Vec<Subscription>,
}

impl ContentPaneHost {
    pub fn new(app: Entity<ThemeStudioApp>, board: BoardSnapshot, cx: &mut Context<Self>) -> Self {
        let tabs = board
            .look
            .tabs_navigation("theme-studio-content-tabs")
            .items([
                TabsNavigationItem::new("cards").label("Cards"),
                TabsNavigationItem::new("dashboard").label("Dashboard"),
                TabsNavigationItem::new("palette").label("Palette"),
            ])
            .active("cards")
            .spawn(cx);

        // Subscribe on ContentPaneHost, not ThemeStudioApp: a parent subscription that updates
        // the parent entity re-enters and panics (see ThemeSidebar::wire_subscriptions).
        let tabs_for_sub = tabs.clone();
        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&tabs_for_sub, |host, _, event: &TabsNavigationEvent, cx| {
            let TabsNavigationEvent::Activate { tab_id, .. } = event;
            if let Some(tab) = ContentTab::from_id(tab_id.as_ref()) {
                host.set_active_tab(tab, cx);
            }
        }));

        let palette_panel = cx.new(|cx| PalettePanel::new(cx, board.look.clone(), board.overrides.clone()));

        Self { app, tabs, palette_panel, board, active_tab: ContentTab::Cards, _subscriptions: subscriptions }
    }

    pub fn set_active_tab(&mut self, tab: ContentTab, cx: &mut Context<Self>) {
        if self.active_tab == tab {
            return;
        }
        self.active_tab = tab;

        if tab == ContentTab::Palette {
            let look = self.board.look.clone();
            let overrides = self.board.overrides.clone();
            self.palette_panel.update(cx, |panel, cx| panel.sync_snapshot(look, overrides, cx));
        }

        let clear_selection = self.board.selected.is_some_and(|id| !tab.contains(id));
        if clear_selection {
            self.board.selected = None;
            self.app.update(cx, |app, cx| {
                app.selected = None;
                cx.notify();
            });
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
        self.tabs.update(cx, |tabs, cx| tabs.set_template(look.tabs_navigation_template(), cx));
        self.palette_panel.update(cx, |panel, cx| panel.sync_snapshot(look, overrides, cx));
        cx.notify();
    }

    pub fn begin_panel_drag(&mut self, id: InspectableId, event: &MouseDownEvent, cx: &mut Context<Self>) {
        self.app.update(cx, |app, cx| app.begin_panel_drag(id, event, cx));
        self.pull_board_from_app(cx);
    }

    pub fn handle_panel_drag_move(
        &mut self,
        event: &DragMoveEvent<DemoPanelDrag>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.app.update(cx, |app, cx| app.handle_panel_drag_move(event, window, cx));
        self.pull_board_from_app(cx);
    }

    pub fn end_panel_drag(&mut self, _event: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.app.update(cx, |app, cx| app.end_panel_drag(window, cx));
        self.pull_board_from_app(cx);
    }

    fn pull_board_from_app(&mut self, cx: &mut Context<Self>) {
        let app = self.app.read(cx);
        self.board.selected = app.selected;
        self.board.panel_positions = app.panel_positions.clone();
        self.board.panel_z_order = app.panel_z_order.clone();
        cx.notify();
    }
}

impl Render for ContentPaneHost {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let board = &self.board;
        let chrome = board.look.chrome();
        let board_bg = board.look.token_color("background").unwrap_or(chrome.app_background);
        let active_tab = self.active_tab;

        div()
            .id("theme-studio-content-pane")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(board_bg)
            .child(
                div()
                    .flex_shrink_0()
                    .px(px(24.0))
                    .pt(px(16.0))
                    .pb(px(8.0))
                    .border_b_1()
                    .border_color(chrome.border)
                    .child(self.tabs.clone()),
            )
            .child(match active_tab {
                ContentTab::Cards => scrollable_body().child(div().p(px(24.0)).child(render_demo_board(
                    board.selected,
                    board.panel_positions.clone(),
                    board.panel_z_order.clone(),
                    board.demos.clone(),
                    active_tab.panels(),
                    cx,
                ))),
                ContentTab::Dashboard => dashboard_viewport().child(board.demos.dashboard.clone()),
                ContentTab::Palette => palette_viewport().child(self.palette_panel.clone()),
            })
    }
}

fn scrollable_body() -> gpui::Div {
    let mut panel = div().flex_1().min_h_0().size_full().flex().flex_col();
    panel.style().overflow.y = Some(Overflow::Scroll);
    panel
}

fn dashboard_viewport() -> gpui::Div {
    div().flex_1().min_h_0().size_full().overflow_hidden()
}

fn palette_viewport() -> gpui::Div {
    div().flex_1().min_h_0().size_full().overflow_hidden()
}
