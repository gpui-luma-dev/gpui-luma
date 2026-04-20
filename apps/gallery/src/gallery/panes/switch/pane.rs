use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, rgb};
use gpui_luma::controls::switch::{Switch, SwitchEvent};

use crate::gallery::control::GalleryApp;

use super::super::shared::gallery_pane;

#[derive(Clone)]
pub(in crate::gallery) struct SwitchPane {
    switch: Entity<Switch>,
    disabled_switch: Entity<Switch>,
    on: bool,
}

impl SwitchPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            switch: Switch::new("switch-example").on(true).spawn(cx),
            disabled_switch: Switch::new("disabled-switch").on(true).enabled(false).spawn(cx),
            on: true,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.switch, |app, _, event: &SwitchEvent, cx| {
            app.panes.switch.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self) -> AnyElement {
        gallery_pane(
            "Switch",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(
                    div().flex().items_center().gap_3().child(self.switch.clone()).child(self.disabled_switch.clone()),
                )
                .child(div().text_color(rgb(0x334155)).child(format!("On: {}", self.on)))
                .into_any_element(),
        )
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
