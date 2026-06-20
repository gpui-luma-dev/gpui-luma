use gpui::{Context, Entity, Subscription};
use gpui_luma::controls::selector::SelectorEvent;

use super::panels::{ColorsPanel, OtherPanel};
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

        let colors_panel = sidebar.read(cx).colors_panel.clone();
        ColorsPanel::wire_subscriptions(&colors_panel, cx, subscriptions);

        let other_panel = sidebar.read(cx).other_panel.clone();
        OtherPanel::wire_subscriptions(&other_panel, cx, subscriptions);
    }
}
