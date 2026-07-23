use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Focusable, Subscription, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::overlay_window::OverlayWindowEvent;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{gallery_pane_with_description, notify_entity};

use super::control::{Dialog, default_dialog_template};
use super::shared::{render_stage, render_status, status_for};

#[derive(Clone)]
pub(in crate::gallery::panes::dialog) struct ModalDialogDemo {
    trigger: Entity<Button>,
    cancel: Entity<Button>,
    archive: Entity<Button>,
    dialog: Entity<Dialog>,
    status: String,
}

impl ModalDialogDemo {
    pub(in crate::gallery::panes::dialog) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let cancel = look.outline_button("dialog-modal-cancel").label("Cancel").spawn(cx);
        let archive = look.primary_button("dialog-modal-archive").label("Archive").spawn(cx);
        let dialog = cx.new(|cx| Dialog::spawn(look.clone(), "dialog-modal", default_dialog_template(), cx));
        dialog.update(cx, |dialog, cx| {
            dialog.set_title("Archive project?", cx);
            dialog.set_body_render(
                |_, _, _| {
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(div().w_full().child(
                            "Modal dialogs should trap focus, dim the workspace, and return focus to the opener.",
                        ))
                        .child(
                            div()
                                .w_full()
                                .child("Use this path for confirm/cancel work that should block the background."),
                        )
                        .child(
                            div()
                                .w_full()
                                .child("Escape, click-away, or your own composed actions can dismiss the surface."),
                        )
                        .into_any_element()
                },
                cx,
            );
            dialog.set_footer_render(
                {
                    let cancel = cancel.clone();
                    let archive = archive.clone();
                    move |_, _, _| {
                        div()
                            .w_full()
                            .flex()
                            .justify_end()
                            .gap(px(12.0))
                            .child(cancel.clone())
                            .child(archive.clone())
                            .into_any_element()
                    }
                },
                cx,
            );
        });

        Self {
            trigger: look.primary_button("dialog-modal-trigger").label("Open Modal").spawn(cx),
            cancel,
            archive,
            dialog,
            status: "Closed".into(),
        }
    }

    pub(in crate::gallery::panes::dialog) fn subscribe(
        &self,
        cx: &mut Context<GalleryApp>,
        subscriptions: &mut Vec<Subscription>,
    ) {
        subscriptions.push(cx.subscribe(&self.trigger, |app, _, event: &ButtonEvent, cx| {
            if !event.is_click() {
                return;
            }
            app.panes.dialog.modal.open(cx);
        }));
        subscriptions.push(cx.subscribe(&self.cancel, |app, _, event: &ButtonEvent, cx| {
            if !event.is_click() {
                return;
            }
            app.panes.dialog.modal.dismiss("Dismissed modal overlay.", cx);
        }));
        subscriptions.push(cx.subscribe(&self.archive, |app, _, event: &ButtonEvent, cx| {
            if !event.is_click() {
                return;
            }
            app.panes.dialog.modal.dismiss("Archived project from composed action.", cx);
        }));
        subscriptions.push(cx.subscribe(&self.dialog, |app, _, event: &OverlayWindowEvent, cx| {
            app.panes.dialog.modal.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery::panes::dialog) fn render(&self, look: &ShadcnLook) -> AnyElement {
        gallery_pane_with_description(
            "Dialog: Modal",
            Some("Single-purpose page for focus trap, backdrop, and focus restore."),
            render_stage(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(16.0))
                    .child(self.trigger.clone())
                    .child(render_status("Status", &self.status, look))
                    .child(self.dialog.clone())
                    .into_any_element(),
            ),
            look,
        )
    }

    pub(in crate::gallery::panes::dialog) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        for entity in [&self.trigger, &self.cancel, &self.archive] {
            notify_entity(entity, cx);
        }
        notify_entity(&self.dialog, cx);
    }

    pub(in crate::gallery::panes::dialog) fn open(&mut self, cx: &mut Context<GalleryApp>) {
        let focus = self.trigger.read(cx).focus_handle(cx);
        self.dialog.update(cx, |dialog, cx| dialog.open(Some(focus), cx));
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
        self.status = status_for(event, "modal overlay");
        cx.notify();
    }
}
