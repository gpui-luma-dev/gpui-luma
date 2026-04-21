use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*};
use gpui_luma::controls::button::{Button, ButtonEvent, ButtonKind};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ButtonPane {
    button: Entity<Button>,
    disabled_button: Entity<Button>,
    clicks: usize,
}

impl ButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            button: Button::new("button-example")
                .label("Click me")
                .kind(ButtonKind::Primary)
                .template(theme.button_template())
                .spawn(cx),
            disabled_button: Button::new("disabled-button")
                .label("Disabled")
                .enabled(false)
                .template(theme.button_template())
                .spawn(cx),
            clicks: 0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.button, |app, _, event: &ButtonEvent, cx| {
            app.panes.button.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        gallery_pane(
            "Button",
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(self.button.clone())
                .child(self.disabled_button.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.button, cx);
        notify_entity(&self.disabled_button, cx);
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
