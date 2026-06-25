use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Focusable, Subscription, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::overlay_window::{OverlayWindowEvent, OverlayWindowMode, OverlayWindowPosition};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{gallery_pane_with_description, notify_entity};

use super::control::{Dialog, default_dialog_template};
use super::shared::{render_stage, render_status, status_for};

#[derive(Clone)]
pub(in crate::gallery::panes::dialog) struct ModelessDialogDemo {
    trigger: Entity<Button>,
    background_counter: Entity<Button>,
    close: Entity<Button>,
    dialog: Entity<Dialog>,
    background_clicks: usize,
    status: String,
}

impl ModelessDialogDemo {
    pub(in crate::gallery::panes::dialog) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let close = look.ghost_icon_button("dialog-modeless-close", LucideIcon::X).spawn(cx);
        let dialog = cx.new(|cx| Dialog::spawn(look.clone(), "dialog-modeless", default_dialog_template(), cx));
        dialog.update(cx, |dialog, cx| {
            dialog.set_title("Modeless notes", cx);
            dialog.set_mode(OverlayWindowMode::Modeless, cx);
            dialog.set_position(OverlayWindowPosition::TopRight, cx);
            dialog.set_header_end(close.clone(), cx);
            dialog.set_body_render(
                |_, _, _| {
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div().w_full().child(
                                "This surface should stay out of the way while the background remains interactive.",
                            ),
                        )
                        .child(div().w_full().child("Leave this open, then keep clicking the background counter."))
                        .child(
                            div().w_full().child("That proves the pane is still interactive under a modeless overlay."),
                        )
                        .into_any_element()
                },
                cx,
            );
        });

        Self {
            trigger: look.primary_button("dialog-modeless-trigger").label("Open Modeless").spawn(cx),
            background_counter: look.outline_button("dialog-background-counter").label("Background 0").spawn(cx),
            close,
            dialog,
            background_clicks: 0,
            status: "Closed".into(),
        }
    }

    pub(in crate::gallery::panes::dialog) fn subscribe(
        &self,
        cx: &mut Context<GalleryApp>,
        subscriptions: &mut Vec<Subscription>,
    ) {
        subscriptions.push(cx.subscribe(&self.trigger, |app, _, _: &ButtonEvent, cx| {
            app.panes.dialog.modeless.open(cx);
        }));
        subscriptions.push(cx.subscribe(&self.background_counter, |app, _, _: &ButtonEvent, cx| {
            app.panes.dialog.modeless.increment_background(cx);
        }));
        subscriptions.push(cx.subscribe(&self.close, |app, _, _: &ButtonEvent, cx| {
            app.panes.dialog.modeless.dismiss("Dismissed modeless overlay.", cx);
        }));
        subscriptions.push(cx.subscribe(&self.dialog, |app, _, event: &OverlayWindowEvent, cx| {
            app.panes.dialog.modeless.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery::panes::dialog) fn render(&self, look: &ShadcnLook) -> AnyElement {
        gallery_pane_with_description(
            "Dialog: Modeless",
            Some("Single-purpose page for non-blocking overlay behavior."),
            render_stage(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(16.0))
                    .child(self.trigger.clone())
                    .child(self.background_counter.clone())
                    .child(render_status("Status", &self.status, look))
                    .child(self.dialog.clone())
                    .into_any_element(),
            ),
            look,
        )
    }

    pub(in crate::gallery::panes::dialog) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        for entity in [&self.trigger, &self.background_counter, &self.close] {
            notify_entity(entity, cx);
        }
        notify_entity(&self.dialog, cx);
    }

    pub(in crate::gallery::panes::dialog) fn open(&mut self, cx: &mut Context<GalleryApp>) {
        let focus = self.trigger.read(cx).focus_handle(cx);
        self.dialog.update(cx, |dialog, cx| dialog.open(Some(focus), cx));
    }

    pub(in crate::gallery::panes::dialog) fn increment_background(&mut self, cx: &mut Context<GalleryApp>) {
        self.background_clicks += 1;
        self.background_counter.update(cx, |button, cx| {
            button.set_label(format!("Background {}", self.background_clicks), cx);
        });
        self.status = format!("Background still interactive: {} clicks.", self.background_clicks);
        cx.notify();
    }

    pub(in crate::gallery::panes::dialog) fn dismiss(
        &mut self,
        status: impl Into<String>,
        cx: &mut Context<GalleryApp>,
    ) {
        self.status = status.into();
        self.dialog.update(cx, |dialog, cx| dialog.dismiss(cx));
        cx.notify();
    }

    pub(in crate::gallery::panes::dialog) fn handle_event(
        &mut self,
        event: &OverlayWindowEvent,
        cx: &mut Context<GalleryApp>,
    ) {
        self.status = status_for(event, "modeless overlay");
        cx.notify();
    }
}
