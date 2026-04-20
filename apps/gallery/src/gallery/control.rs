use gpui::{Context, Entity, FocusHandle, Subscription, Window, px};
use gpui_luma::controls::nav_view::{NavView, NavViewEvent};
use gpui_luma::controls::split_view::{SplitView, SplitViewEvent};

use super::panes::registry::GalleryPanes;

pub struct GalleryApp {
    pub(super) focus_scope: FocusHandle,
    pub(super) split_view: Entity<SplitView>,
    pub(super) nav_view: Entity<NavView>,
    pub(super) panes: GalleryPanes,
    pub(super) nav_selection: String,
    pub(super) nav_toggle: String,
    pub(super) split_sidebar_width: f32,
    pub(super) split_sidebar_collapsed: bool,
    _subscriptions: Vec<Subscription>,
}

impl GalleryApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_scope = cx.focus_handle();
        window.focus(&focus_scope, cx);

        let split_view = SplitView::new("gallery-shell")
            .sidebar_width(px(280.0))
            .sidebar_min_width(px(220.0))
            .sidebar_max_width(px(420.0))
            .sidebar_collapsed_width(px(0.0))
            .collapsed(false)
            .resizable(true)
            .spawn(cx);
        let initial_selection = GalleryPanes::initial_selection();
        let nav_view = NavView::new("gallery-nav")
            .items(GalleryPanes::nav_items())
            .bottom_items(GalleryPanes::bottom_nav_items())
            .selected(initial_selection)
            .spawn(cx);
        let panes = GalleryPanes::new(cx);

        let mut subscriptions = vec![
            cx.subscribe(&split_view, |this, _, event: &SplitViewEvent, cx| {
                this.handle_split_view_event(event, cx);
            }),
            cx.subscribe(&nav_view, |this, _, event: &NavViewEvent, cx| {
                this.handle_nav_view_event(event, cx);
            }),
        ];
        panes.subscribe(cx, &mut subscriptions);

        Self {
            focus_scope,
            split_view,
            nav_view,
            panes,
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
                cx.notify();
            }
        }
    }

    fn handle_nav_view_event(&mut self, event: &NavViewEvent, cx: &mut Context<Self>) {
        match event {
            NavViewEvent::Activate { item_id } => {
                self.nav_selection = item_id.to_string();
                cx.notify();
            }
            NavViewEvent::ToggleNode { node_id, expanded } => {
                self.nav_toggle = format!("{}: {}", node_id, expanded);
                cx.notify();
            }
        }
    }
}
