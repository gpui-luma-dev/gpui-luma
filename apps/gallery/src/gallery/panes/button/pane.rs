use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*};
use gpui_luma::controls::button::{Button, ButtonEvent, ButtonKind};

use crate::gallery::control::GalleryApp;

use super::super::shared::gallery_pane;

pub(in crate::gallery) struct ButtonPane {
    button: Entity<Button>,
    disabled_button: Entity<Button>,
    clicks: usize,
}

impl ButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            button: Button::new("button-example").label("Click me").kind(ButtonKind::Primary).spawn(cx),
            disabled_button: Button::new("disabled-button").label("Disabled").enabled(false).spawn(cx),
            clicks: 0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self) -> AnyElement {
        gallery_pane(
            "Button",
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(self.button.clone())
                .child(self.disabled_button.clone())
                .into_any_element(),
        )
    }

    fn handle_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.clicks += 1;
                let label = format!("Clicked {}", self.clicks);

                self.button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }
}
