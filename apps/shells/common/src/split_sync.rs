use gpui::{Context, Entity, Subscription};
use gpui_luma::controls::sidebar::{SidebarControl, SidebarPresentation};
use gpui_luma::controls::split_view::{SplitView, SplitViewEvent};

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
                    sidebar.set_presentation(
                        if *collapsed {
                            SidebarPresentation::Icons
                        } else {
                            SidebarPresentation::Expanded
                        },
                        cx,
                    );
                });
            }
        }
    }));
}
