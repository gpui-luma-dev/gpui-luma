use gpui::{Context, Entity, Subscription};
use luma::controls::sidebar::{SidebarControl, SidebarEvent};
use luma::controls::split_view::{SplitView, SplitViewEvent};

pub fn wire_split_nav_sync<T: 'static>(
    subscriptions: &mut Vec<Subscription>,
    split_view: Entity<SplitView>,
    sidebar: Entity<SidebarControl>,
    cx: &mut Context<T>,
) {
    subscriptions.push(cx.subscribe(&split_view, {
        let sidebar = sidebar.clone();
        move |_, _, event: &SplitViewEvent, cx| {
            if let SplitViewEvent::CollapsedChanged { collapsed } = event {
                sidebar.update(cx, |sidebar, cx| {
                    sidebar.set_open(!*collapsed, cx);
                });
            }
        }
    }));

    subscriptions.push(cx.subscribe(&sidebar, {
        let split_view = split_view.clone();
        move |_, _, event: &SidebarEvent, cx| {
            if let SidebarEvent::OpenChanged { open, .. } = event {
                split_view.update(cx, |split_view, cx| {
                    split_view.set_collapsed(!*open, cx);
                });
            }
        }
    }));
}
