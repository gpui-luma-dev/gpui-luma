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
pub(in crate::gallery::panes::dialog) struct PositioningDialogDemo {
    center_trigger: Entity<Button>,
    corner_trigger: Entity<Button>,
    absolute_trigger: Entity<Button>,
    close: Entity<Button>,
    dialog: Entity<Dialog>,
    status: String,
}

impl PositioningDialogDemo {
    pub(in crate::gallery::panes::dialog) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let close = look.ghost_icon_button("dialog-positioning-close", LucideIcon::X).spawn(cx);
        let dialog = cx.new(|cx| Dialog::spawn(look.clone(), "dialog-positioning", default_dialog_template(), cx));
        dialog.update(cx, |dialog, cx| {
            dialog.set_title("Positioned overlay", cx);
            dialog.set_mode(OverlayWindowMode::Modeless, cx);
            dialog.set_header_end(close.clone(), cx);
            dialog.set_body_render(
                |model, _, _| {
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div().w_full().child(
                                "One pane, one variable: the same modeless overlay reopened at different anchors.",
                            ),
                        )
                        .child(div().w_full().child(format!("Current position: {:?}", model.position)))
                        .child(
                            div()
                                .w_full()
                                .child("Use the launch buttons below to reopen this surface in a different spot."),
                        )
                        .into_any_element()
                },
                cx,
            );
        });

        Self {
            center_trigger: look.secondary_button("dialog-position-center").label("Center").spawn(cx),
            corner_trigger: look.secondary_button("dialog-position-corner").label("Top Right").spawn(cx),
            absolute_trigger: look.secondary_button("dialog-position-absolute").label("Absolute").spawn(cx),
            close,
            dialog,
            status: "Closed".into(),
        }
    }

    pub(in crate::gallery::panes::dialog) fn subscribe(
        &self,
        cx: &mut Context<GalleryApp>,
        subscriptions: &mut Vec<Subscription>,
    ) {
        subscriptions.push(cx.subscribe(&self.center_trigger, |app, _, event: &ButtonEvent, cx| {
            if !event.is_click() {
                return;
            }
            app.panes.dialog.positioning.open(OverlayWindowPosition::Center, cx);
        }));
        subscriptions.push(cx.subscribe(&self.corner_trigger, |app, _, event: &ButtonEvent, cx| {
            if !event.is_click() {
                return;
            }
            app.panes.dialog.positioning.open(OverlayWindowPosition::TopRight, cx);
        }));
        subscriptions.push(cx.subscribe(&self.absolute_trigger, |app, _, event: &ButtonEvent, cx| {
            if !event.is_click() {
                return;
            }
            app.panes
                .dialog
                .positioning
                .open(OverlayWindowPosition::Absolute(gpui::point(px(180.0), px(280.0))), cx);
        }));
        subscriptions.push(cx.subscribe(&self.close, |app, _, event: &ButtonEvent, cx| {
            if !event.is_click() {
                return;
            }
            app.panes.dialog.positioning.dismiss("Dismissed positioned overlay.", cx);
        }));
        subscriptions.push(cx.subscribe(&self.dialog, |app, _, event: &OverlayWindowEvent, cx| {
            app.panes.dialog.positioning.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery::panes::dialog) fn render(&self, look: &ShadcnLook) -> AnyElement {
        gallery_pane_with_description(
            "Dialog: Positioning",
            Some("Single-purpose page for center, corner, and absolute placement."),
            render_stage(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(16.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(12.0))
                            .child(self.center_trigger.clone())
                            .child(self.corner_trigger.clone())
                            .child(self.absolute_trigger.clone()),
                    )
                    .child(render_status("Status", &self.status, look))
                    .child(self.dialog.clone())
                    .into_any_element(),
            ),
            look,
        )
    }

    pub(in crate::gallery::panes::dialog) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        for entity in [&self.center_trigger, &self.corner_trigger, &self.absolute_trigger, &self.close] {
            notify_entity(entity, cx);
        }
        notify_entity(&self.dialog, cx);
    }

    pub(in crate::gallery::panes::dialog) fn open(
        &mut self,
        position: OverlayWindowPosition,
        cx: &mut Context<GalleryApp>,
    ) {
        let focus = self.center_trigger.read(cx).focus_handle(cx);
        self.dialog.update(cx, |dialog, cx| {
            dialog.set_position(position, cx);
            dialog.open(Some(focus), cx);
        });
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
        self.status = status_for(event, "positioned overlay");
        cx.notify();
    }
}
