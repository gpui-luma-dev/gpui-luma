use gpui::{Context, Entity, Subscription};
use gpui_luma::controls::selector::SelectorEvent;

use super::panels::{wire_color_subscriptions, wire_other_subscriptions};
use super::ThemeSidebar;
use crate::studio::app::ThemeStudioApp;

impl ThemeSidebar {
    /// Wire selector and token field events on the app entity.
    ///
    /// Subscriptions must not be registered on `ThemeSidebar` itself: `cx.subscribe` re-enters
    /// the subscriber entity, and handlers call `theme_sidebar.update`, which panics.
    pub fn wire_subscriptions(
        sidebar: &Entity<Self>,
        cx: &mut Context<ThemeStudioApp>,
        subscriptions: &mut Vec<Subscription>,
    ) {
        let theme_selector = sidebar.read(cx).theme_selector.clone();
        subscriptions.push(cx.subscribe(&theme_selector, |app, _, event: &SelectorEvent, cx| {
            let SelectorEvent::Change { item_id, .. } = event;
            app.change_theme(item_id.as_ref(), cx);
        }));

        wire_color_subscriptions(sidebar, cx, subscriptions);
        wire_other_subscriptions(sidebar, cx, subscriptions);
    }
}
