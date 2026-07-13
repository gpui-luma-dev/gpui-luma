use gpui::{Context, Entity, Subscription};
use gpui_luma::controls::navigation_sidebar::{NavigationSidebar, NavigationSidebarEvent};
use gpui_luma::controls::split_view::{SplitView, SplitViewEvent};

pub fn wire_split_nav_sync<T: 'static>(
    subscriptions: &mut Vec<Subscription>,
    split_view: Entity<SplitView>,
    navigation_sidebar: Entity<NavigationSidebar>,
    cx: &mut Context<T>,
) {
    subscriptions.push(cx.subscribe(&split_view, {
        let navigation_sidebar = navigation_sidebar.clone();
        move |_, _, event: &SplitViewEvent, cx| {
            if let SplitViewEvent::CollapsedChanged { collapsed } = event {
                navigation_sidebar.update(cx, |sidebar, cx| {
                    sidebar.set_collapsed(*collapsed, cx);
                });
            }
        }
    }));

    subscriptions.push(cx.subscribe(&navigation_sidebar, {
        let split_view = split_view.clone();
        move |_, _, event: &NavigationSidebarEvent, cx| {
            if let NavigationSidebarEvent::CollapsedChanged { collapsed } = event {
                split_view.update(cx, |split_view, cx| {
                    split_view.set_collapsed(*collapsed, cx);
                });
            }
        }
    }));
}
