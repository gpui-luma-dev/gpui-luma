use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*};
use gpui_luma::controls::icon_button::{IconButton, IconButtonEvent, IconButtonKind};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::super::shared::gallery_pane;

pub(in crate::gallery) struct IconButtonPane {
    icon_button: Entity<IconButton>,
    disabled_icon_button: Entity<IconButton>,
    icon_clicks: usize,
}

impl IconButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            icon_button: IconButton::new("icon-button-example", LucideIcon::Plus)
                .kind(IconButtonKind::Primary)
                .spawn(cx),
            disabled_icon_button: IconButton::new("disabled-icon-button", LucideIcon::Check).enabled(false).spawn(cx),
            icon_clicks: 0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.icon_button, |app, _, event: &IconButtonEvent, cx| {
            app.panes.icon_button.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self) -> AnyElement {
        gallery_pane(
            "Icon Button",
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(self.icon_button.clone())
                .child(self.disabled_icon_button.clone())
                .into_any_element(),
        )
    }

    fn handle_event(&mut self, event: &IconButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            IconButtonEvent::Click => {
                self.icon_clicks += 1;
                let icon = if self.icon_clicks.is_multiple_of(2) {
                    LucideIcon::Plus
                } else {
                    LucideIcon::Check
                };

                self.icon_button.update(cx, |button, cx| {
                    button.set_icon(icon, cx);
                });
            }
        }
    }
}
