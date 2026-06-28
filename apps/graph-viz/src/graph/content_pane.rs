use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::tabs_navigation::{
    TabsNavigation, TabsNavigationEvent, TabsNavigationItem, TabsNavigationWidthMode,
};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};

use super::content_tabs::graph_viz_tabs_navigation_template;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ContentTab {
    #[default]
    Graph,
    Properties,
}

impl ContentTab {
    fn from_id(id: &str) -> Option<Self> {
        match id {
            "graph" => Some(Self::Graph),
            "properties" => Some(Self::Properties),
            _ => None,
        }
    }
}

pub struct ContentPaneHost {
    tabs: Entity<TabsNavigation>,
    look: Arc<ShadcnLook>,
    active_tab: ContentTab,
    _subscriptions: Vec<Subscription>,
}

impl ContentPaneHost {
    pub fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let tabs = look
            .tabs_navigation("graph-viz-content-tabs")
            .size(ControlSize::Lg)
            .width_mode(TabsNavigationWidthMode::Uniform)
            .template(graph_viz_tabs_navigation_template(look.clone(), ControlSize::Lg))
            .items([
                TabsNavigationItem::new("graph").label("Graph"),
                TabsNavigationItem::new("properties").label("Properties"),
            ])
            .active("graph")
            .spawn(cx);

        let tabs_for_sub = tabs.clone();
        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&tabs_for_sub, |host, _, event: &TabsNavigationEvent, cx| {
            let TabsNavigationEvent::Activate { tab_id, .. } = event;
            if let Some(tab) = ContentTab::from_id(tab_id.as_ref()) {
                if host.active_tab != tab {
                    host.active_tab = tab;
                    cx.notify();
                }
            }
        }));

        Self { tabs, look, active_tab: ContentTab::Graph, _subscriptions: subscriptions }
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.tabs.update(cx, |tabs, cx| {
            tabs.set_size(ControlSize::Lg, cx);
            tabs.set_width_mode(TabsNavigationWidthMode::Uniform, cx);
            tabs.set_template(graph_viz_tabs_navigation_template(look, ControlSize::Lg), cx);
        });
        cx.notify();
    }

    pub fn notify_tabs(&self, cx: &mut Context<Self>) {
        self.tabs.update(cx, |_, cx| cx.notify());
    }
}

impl Render for ContentPaneHost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let chrome = self.look.chrome();
        let board_bg = self.look.token_color("background").unwrap_or(chrome.app_background);

        div()
            .id("graph-viz-content-pane")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(board_bg)
            .child(div().flex_shrink_0().pt(px(8.0)).child(div().w_full().child(self.tabs.clone())))
            .child(empty_tab_viewport())
    }
}

fn empty_tab_viewport() -> gpui::Div {
    div().flex_1().min_h_0().size_full().overflow_hidden()
}
