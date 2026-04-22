use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*};
use gpui_luma::controls::switch::{Switch, SwitchEvent};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct SwitchPane {
    switch: Entity<Switch>,
    disabled_switch: Entity<Switch>,
    on: bool,
}

impl SwitchPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            switch: Switch::new("switch-example").on(true).template(theme.switch_template()).spawn(cx),
            disabled_switch: Switch::new("disabled-switch")
                .on(true)
                .enabled(false)
                .template(theme.switch_template())
                .spawn(cx),
            on: true,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.switch, |app, _, event: &SwitchEvent, cx| {
            app.panes.switch.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage(
            "Switch",
            "Switch",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(
                    div().flex().items_center().gap_3().child(self.switch.clone()).child(self.disabled_switch.clone()),
                )
                .child(div().text_color(chrome.body_text).child(format!("On: {}", self.on)))
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.switch, cx);
        notify_entity(&self.disabled_switch, cx);
    }

    fn handle_event(&mut self, event: &SwitchEvent, cx: &mut Context<GalleryApp>) {
        match event {
            SwitchEvent::Change { on } => {
                self.on = *on;
                cx.notify();
            }
        }
    }
}
