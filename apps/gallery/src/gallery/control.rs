use gpui::{Context, Entity, FocusHandle, Subscription, Window, px};
use gpui_luma::controls::navigation_sidebar::{NavigationSidebar, NavigationSidebarEvent};
use gpui_luma::controls::split_view::{SplitView, SplitViewEvent};
use gpui_luma::controls::command::button::ButtonEvent;
use gpui_luma::theme::{RadixTheme, ThemeMode, set_active_radix_theme};
use std::sync::Arc;

use super::panes::registry::{GalleryPanes, GalleryRouteButton};
use super::theme::GalleryThemeChoice;

pub struct GalleryApp {
    pub(super) focus_scope: FocusHandle,
    pub(super) pane_focus: FocusHandle,
    pub(super) radix_theme: Arc<RadixTheme>,
    pub(super) split_view: Entity<SplitView>,
    pub(super) navigation_sidebar: Entity<NavigationSidebar>,
    pub(super) panes: GalleryPanes,
    pub(super) nav_route_buttons: Vec<GalleryRouteButton>,
    pub(super) nav_selection: String,
    pub(super) nav_toggle: String,
    pub(super) split_sidebar_width: f32,
    pub(super) split_sidebar_collapsed: bool,
    _subscriptions: Vec<Subscription>,
}

impl GalleryApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, theme_choice: GalleryThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        let pane_focus = cx.focus_handle().tab_stop(true);
        window.focus(&focus_scope, cx);
        let radix_theme = theme_choice.radix_theme();
        radix_theme.set_mode(ThemeMode::Dark);
        set_active_radix_theme(radix_theme.clone());
        let chrome = radix_theme.chrome();

        let split_view = SplitView::new("gallery-shell")
            .sidebar_width(px(280.0))
            .sidebar_min_width(px(220.0))
            .sidebar_max_width(px(420.0))
            .sidebar_collapsed_width(px(56.0))
            .collapsed(false)
            .resizable(true)
            .separator_color(chrome.border)
            .separator_hover_color(chrome.border)
            .spawn(cx);
        let initial_selection = GalleryPanes::initial_selection();
        let navigation = GalleryPanes::navigation(cx, radix_theme.clone());
        let route_buttons = navigation.route_buttons.clone();
        let branch_buttons = navigation.branch_buttons.clone();
        let navigation_sidebar = NavigationSidebar::new("gallery-nav")
            .title("GPUI-Luma")
            .subtitle("Control gallery")
            .collapsible(true)
            .items(navigation.nodes)
            .footer_nodes(navigation.footer_nodes)
            .template(radix_theme.navigation_sidebar_template())
            .scrollbar_template(radix_theme.scrollbar_template())
            .spawn(cx);
        let panes = GalleryPanes::new(cx, radix_theme.clone());

        let mut subscriptions = vec![
            cx.subscribe(&split_view, |this, _, event: &SplitViewEvent, cx| {
                this.handle_split_view_event(event, cx);
            }),
            cx.subscribe(&navigation_sidebar, |this, _, event: &NavigationSidebarEvent, cx| {
                this.handle_navigation_sidebar_event(event, cx);
            }),
        ];
        for route_button in route_buttons.iter().cloned() {
            let page_id = route_button.page_id.to_string();
            subscriptions.push(cx.subscribe(&route_button.button, move |this, _, _: &ButtonEvent, cx| {
                this.set_nav_selection(&page_id, cx);
            }));
        }
        for branch_button in branch_buttons {
            let node_id = branch_button.node_id.to_string();
            subscriptions.push(cx.subscribe(&branch_button.button, move |this, _, _: &ButtonEvent, cx| {
                this.navigation_sidebar.update(cx, |sidebar, cx| {
                    sidebar.toggle_node_expanded(node_id.clone(), cx);
                });
            }));
        }
        panes.subscribe(cx, &mut subscriptions);

        Self {
            focus_scope,
            pane_focus,
            radix_theme,
            split_view,
            navigation_sidebar,
            panes,
            nav_route_buttons: route_buttons,
            nav_selection: initial_selection.to_string(),
            nav_toggle: "none".to_string(),
            split_sidebar_width: 280.0,
            split_sidebar_collapsed: false,
            _subscriptions: subscriptions,
        }
    }

    fn handle_split_view_event(&mut self, event: &SplitViewEvent, cx: &mut Context<Self>) {
        match event {
            SplitViewEvent::ResizeStart => {}
            SplitViewEvent::SidebarWidthChanged { width } | SplitViewEvent::ResizeEnd { width } => {
                self.split_sidebar_width = width.as_f32();
                cx.notify();
            }
            SplitViewEvent::CollapsedChanged { collapsed } => {
                self.split_sidebar_collapsed = *collapsed;
                self.navigation_sidebar.update(cx, |sidebar, cx| {
                    sidebar.set_collapsed(*collapsed, cx);
                });
                cx.notify();
            }
        }
    }

    fn handle_navigation_sidebar_event(&mut self, event: &NavigationSidebarEvent, cx: &mut Context<Self>) {
        match event {
            NavigationSidebarEvent::Activate { node_id, .. } => {
                self.set_nav_selection(node_id, cx);
            }
            NavigationSidebarEvent::BranchExpandedChanged { node_id, expanded } => {
                self.nav_toggle = format!("{}: {}", node_id, expanded);
                cx.notify();
            }
            NavigationSidebarEvent::CollapsedChanged { collapsed } => {
                self.split_view.update(cx, |split_view, cx| {
                    split_view.set_collapsed(*collapsed, cx);
                });
            }
        }
    }

    fn set_nav_selection(&mut self, page_id: &str, cx: &mut Context<Self>) {
        self.nav_selection = page_id.to_string();

        for route_button in &self.nav_route_buttons {
            let selected = route_button.page_id == page_id;
            route_button.button.update(cx, |button, cx| {
                if *button.data() != selected {
                    button.set_data(selected, cx);
                }
            });
        }

        cx.notify();
    }
}
