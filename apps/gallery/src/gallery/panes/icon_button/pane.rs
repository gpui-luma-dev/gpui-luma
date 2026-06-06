use std::sync::Arc;

use gpui::{AnyElement, Context, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::command::button::ButtonEvent;
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct IconButtonPane {
    secondary_icon_button: IconButton,
    outline_icon_button: IconButton,
    ghost_icon_button: IconButton,
    primary_icon_button: IconButton,
    secondary_icon_clicks: usize,
    outline_icon_clicks: usize,
    ghost_icon_clicks: usize,
    primary_icon_clicks: usize,
}

impl IconButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        Self {
            secondary_icon_button: look
                .secondary_icon_button("icon-button-secondary-example", LucideIcon::Plus)
                .spawn(cx),
            outline_icon_button: look.outline_icon_button("icon-button-outline-example", LucideIcon::Plus).spawn(cx),
            ghost_icon_button: look.ghost_icon_button("icon-button-ghost-example", LucideIcon::Plus).spawn(cx),
            primary_icon_button: look.primary_icon_button("icon-button-primary-example", LucideIcon::Plus).spawn(cx),
            secondary_icon_clicks: 0,
            outline_icon_clicks: 0,
            ghost_icon_clicks: 0,
            primary_icon_clicks: 0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.secondary_icon_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.icon_button.handle_secondary_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.outline_icon_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.icon_button.handle_outline_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.ghost_icon_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.icon_button.handle_ghost_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.primary_icon_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.icon_button.handle_primary_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        gallery_pane_with_usage(
            "Command (Icon)",
            "Icon Button",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(self.primary_icon_button.clone())
                        .child(self.secondary_icon_button.clone())
                        .child(self.outline_icon_button.clone())
                        .child(self.ghost_icon_button.clone()),
                )
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.secondary_icon_button, cx);
        notify_entity(&self.outline_icon_button, cx);
        notify_entity(&self.ghost_icon_button, cx);
        notify_entity(&self.primary_icon_button, cx);
    }

    fn toggle_icon(button: &IconButton, clicks: &mut usize, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                *clicks += 1;
                let icon = if clicks.is_multiple_of(2) {
                    LucideIcon::Plus
                } else {
                    LucideIcon::Check
                };

                button.update(cx, |button, cx| {
                    let content: gpui_luma::controls::command::button::ControlPresenter<
                        gpui_luma::controls::command::button::ButtonRenderModel<()>,
                    > = Arc::new(move |_, _| {
                        div()
                            .font_family("lucide")
                            .text_size(px(16.0))
                            .child(char::from(icon).to_string())
                            .into_any_element()
                    });
                    button.set_presenter(content, cx);
                });
            }
        }
    }

    fn handle_secondary_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        Self::toggle_icon(&self.secondary_icon_button, &mut self.secondary_icon_clicks, event, cx);
    }

    fn handle_outline_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        Self::toggle_icon(&self.outline_icon_button, &mut self.outline_icon_clicks, event, cx);
    }

    fn handle_ghost_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        Self::toggle_icon(&self.ghost_icon_button, &mut self.ghost_icon_clicks, event, cx);
    }

    fn handle_primary_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        Self::toggle_icon(&self.primary_icon_button, &mut self.primary_icon_clicks, event, cx);
    }
}
