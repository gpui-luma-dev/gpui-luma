use gpui::{Context, Entity, Subscription};
use gpui_luma::controls::search_selector::SearchSelectorEvent;

use super::panels::{ColorsPanel, OtherPanel, TypographyPanel};
use super::ThemeSidebar;
use crate::studio::app::LumaStudioApp;

impl ThemeSidebar {
    /// Wire selector and token field events on the app entity.
    ///
    /// Subscriptions must not be registered on `ThemeSidebar` itself: `cx.subscribe` re-enters
    /// the subscriber entity, and handlers call `theme_sidebar.update`, which panics.
    pub fn wire_subscriptions(
        sidebar: &Entity<Self>,
        cx: &mut Context<LumaStudioApp>,
        subscriptions: &mut Vec<Subscription>,
    ) {
        let theme_selector = sidebar.read(cx).theme_selector.clone();
        subscriptions.push(cx.subscribe(&theme_selector, |app, _, event: &SearchSelectorEvent, cx| {
            let theme_id = match event {
                SearchSelectorEvent::Select { item_id, .. } | SearchSelectorEvent::Complete { item_id, .. } => item_id,
                _ => return,
            };
            app.change_theme(theme_id.as_ref(), cx);
        }));

        let colors_panel = sidebar.read(cx).colors_panel.clone();
        ColorsPanel::wire_subscriptions(&colors_panel, cx, subscriptions);

        let other_panel = sidebar.read(cx).other_panel.clone();
        OtherPanel::wire_subscriptions(&other_panel, cx, subscriptions);

        let typography_panel = sidebar.read(cx).typography_panel.clone();
        TypographyPanel::wire_subscriptions(&typography_panel, cx, subscriptions);
    }
}
