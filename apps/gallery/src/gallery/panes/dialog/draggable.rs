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
pub(in crate::gallery::panes::dialog) struct DraggableDialogDemo {
    trigger: Entity<Button>,
    close: Entity<Button>,
    dialog: Entity<Dialog>,
    status: String,
}

impl DraggableDialogDemo {
    pub(in crate::gallery::panes::dialog) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let close = look.ghost_icon_button("dialog-draggable-close", LucideIcon::X).spawn(cx);
        let dialog = cx.new(|cx| Dialog::spawn(look.clone(), "dialog-draggable", default_dialog_template(), cx));
        dialog.update(cx, |dialog, cx| {
            dialog.set_title("Draggable tool palette", cx);
            dialog.set_mode(OverlayWindowMode::Modeless, cx);
            dialog.set_position(OverlayWindowPosition::Absolute(gpui::point(px(240.0), px(180.0))), cx);
            dialog.set_draggable(true, cx);
            dialog.set_header_end(close.clone(), cx);
            dialog.set_body_render(
                |model, _, _| {
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(div().w_full().child(
                            "Only the top strip should initiate drag. The rest of the body stays normal content.",
                        ))
                        .child(div().w_full().child(format!("Position: {:?}", model.position)))
                        .child(
                            div()
                                .w_full()
                                .child("Drag this panel by its top strip to validate modeless repositioning."),
                        )
                        .into_any_element()
                },
                cx,
            );
        });

        Self {
            trigger: look.primary_button("dialog-draggable-trigger").label("Open Draggable").spawn(cx),
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
        subscriptions.push(cx.subscribe(&self.trigger, |app, _, _: &ButtonEvent, cx| {
            app.panes.dialog.draggable.open(cx);
        }));
        subscriptions.push(cx.subscribe(&self.close, |app, _, _: &ButtonEvent, cx| {
            app.panes.dialog.draggable.dismiss("Dismissed draggable overlay.", cx);
        }));
        subscriptions.push(cx.subscribe(&self.dialog, |app, _, event: &OverlayWindowEvent, cx| {
            app.panes.dialog.draggable.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery::panes::dialog) fn render(&self, look: &ShadcnLook) -> AnyElement {
        gallery_pane_with_description(
            "Dialog: Draggable",
            Some("Single-purpose page for header drag on a modeless tool panel."),
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
        for entity in [&self.trigger, &self.close] {
            notify_entity(entity, cx);
        }
        notify_entity(&self.dialog, cx);
    }

    pub(in crate::gallery::panes::dialog) fn open(&mut self, cx: &mut Context<GalleryApp>) {
        let focus = self.trigger.read(cx).focus_handle(cx);
        self.dialog.update(cx, |dialog, cx| {
            dialog.set_position(OverlayWindowPosition::Absolute(gpui::point(px(240.0), px(180.0))), cx);
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
        self.status = status_for(event, "draggable overlay");
        cx.notify();
    }
}
