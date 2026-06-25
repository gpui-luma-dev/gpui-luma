use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Focusable, Subscription, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::dialog::{Dialog, DialogEvent, DialogMode, DialogPosition};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{gallery_pane_with_description, notify_entity};

#[derive(Clone, Copy)]
pub(in crate::gallery) enum DialogDemoKind {
    Modal,
    Modeless,
    Positioning,
    Draggable,
}

#[derive(Clone)]
pub(in crate::gallery) struct DialogPane {
    modal_trigger: Entity<Button>,
    modeless_trigger: Entity<Button>,
    positioning_center_trigger: Entity<Button>,
    positioning_corner_trigger: Entity<Button>,
    positioning_absolute_trigger: Entity<Button>,
    draggable_trigger: Entity<Button>,
    background_counter: Entity<Button>,
    modal_dialog: Dialog,
    modeless_dialog: Dialog,
    positioning_dialog: Dialog,
    draggable_dialog: Dialog,
    background_clicks: usize,
    modal_status: String,
    modeless_status: String,
    positioning_status: String,
    draggable_status: String,
}

impl DialogPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let modal_dialog = look
            .dialog("dialog-modal")
            .title("Archive project?")
            .description("Modal dialogs should trap focus, dim the workspace, and return focus to the opener.")
            .body(|_, _, _| {
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .child(
                        div().w_full().child("Use this path for confirm/cancel work that should block the background."),
                    )
                    .child(div().w_full().child("Escape, the close button, and Cancel should all dismiss the surface."))
                    .into_any_element()
            })
            .spawn(cx);

        let modeless_dialog = look
            .dialog("dialog-modeless")
            .title("Modeless Notes")
            .description("This surface should stay out of the way while the background remains interactive.")
            .mode(DialogMode::Modeless)
            .position(DialogPosition::TopRight)
            .body(|_, _, _| {
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .child(div().w_full().child("Leave this open, then keep clicking the background counter."))
                    .child(div().w_full().child("That proves the pane is still interactive under a modeless dialog."))
                    .into_any_element()
            })
            .spawn(cx);

        let positioning_dialog = look
            .dialog("dialog-positioning")
            .title("Positioned Dialog")
            .description("One pane, one variable: the same modeless dialog reopened at different anchors.")
            .mode(DialogMode::Modeless)
            .body(|model, _, _| {
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .child(div().w_full().child(format!("Current position: {:?}", model.position)))
                    .child(
                        div().w_full().child("Use the launch buttons below to reopen this dialog in a different spot."),
                    )
                    .into_any_element()
            })
            .spawn(cx);

        let draggable_dialog = look
            .dialog("dialog-draggable")
            .title("Draggable Tool Palette")
            .description("Only the header should initiate drag. The rest of the body stays normal content.")
            .mode(DialogMode::Modeless)
            .position(DialogPosition::Absolute(gpui::point(px(240.0), px(180.0))))
            .draggable(true)
            .body(|model, _, _| {
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .child(div().w_full().child(format!("Position: {:?}", model.position)))
                    .child(div().w_full().child("Drag this panel by its header to validate modeless repositioning."))
                    .into_any_element()
            })
            .spawn(cx);

        Self {
            modal_trigger: look.primary_button("dialog-modal-trigger").label("Open Modal").spawn(cx),
            modeless_trigger: look.primary_button("dialog-modeless-trigger").label("Open Modeless").spawn(cx),
            positioning_center_trigger: look.secondary_button("dialog-position-center").label("Center").spawn(cx),
            positioning_corner_trigger: look.secondary_button("dialog-position-corner").label("Top Right").spawn(cx),
            positioning_absolute_trigger: look.secondary_button("dialog-position-absolute").label("Absolute").spawn(cx),
            draggable_trigger: look.primary_button("dialog-draggable-trigger").label("Open Draggable").spawn(cx),
            background_counter: look.outline_button("dialog-background-counter").label("Background 0").spawn(cx),
            modal_dialog,
            modeless_dialog,
            positioning_dialog,
            draggable_dialog,
            background_clicks: 0,
            modal_status: "Closed".into(),
            modeless_status: "Closed".into(),
            positioning_status: "Closed".into(),
            draggable_status: "Closed".into(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.modal_trigger, |app, _, _: &ButtonEvent, cx| {
            app.panes.dialog.open_modal(cx);
        }));
        subscriptions.push(cx.subscribe(&self.modeless_trigger, |app, _, _: &ButtonEvent, cx| {
            app.panes.dialog.open_modeless(cx);
        }));
        subscriptions.push(cx.subscribe(&self.positioning_center_trigger, |app, _, _: &ButtonEvent, cx| {
            app.panes.dialog.open_positioned(DialogPosition::Center, cx);
        }));
        subscriptions.push(cx.subscribe(&self.positioning_corner_trigger, |app, _, _: &ButtonEvent, cx| {
            app.panes.dialog.open_positioned(DialogPosition::TopRight, cx);
        }));
        subscriptions.push(cx.subscribe(&self.positioning_absolute_trigger, |app, _, _: &ButtonEvent, cx| {
            app.panes.dialog.open_positioned(DialogPosition::Absolute(gpui::point(px(180.0), px(280.0))), cx);
        }));
        subscriptions.push(cx.subscribe(&self.draggable_trigger, |app, _, _: &ButtonEvent, cx| {
            app.panes.dialog.open_draggable(cx);
        }));
        subscriptions.push(cx.subscribe(&self.background_counter, |app, _, _: &ButtonEvent, cx| {
            app.panes.dialog.increment_background(cx);
        }));
        subscriptions.push(cx.subscribe(&self.modal_dialog, |app, _, event: &DialogEvent, cx| {
            app.panes.dialog.handle_modal_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.modeless_dialog, |app, _, event: &DialogEvent, cx| {
            app.panes.dialog.handle_modeless_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.positioning_dialog, |app, _, event: &DialogEvent, cx| {
            app.panes.dialog.handle_positioning_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.draggable_dialog, |app, _, event: &DialogEvent, cx| {
            app.panes.dialog.handle_draggable_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, kind: DialogDemoKind, look: &ShadcnLook) -> AnyElement {
        match kind {
            DialogDemoKind::Modal => gallery_pane_with_description(
                "Dialog: Modal",
                Some("Single-purpose page for focus trap, backdrop, and focus restore."),
                render_stage(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(16.0))
                        .child(self.modal_trigger.clone())
                        .child(render_status("Status", &self.modal_status, look))
                        .child(self.modal_dialog.clone())
                        .into_any_element(),
                ),
                look,
            ),
            DialogDemoKind::Modeless => gallery_pane_with_description(
                "Dialog: Modeless",
                Some("Single-purpose page for non-blocking overlay behavior."),
                render_stage(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(16.0))
                        .child(self.modeless_trigger.clone())
                        .child(self.background_counter.clone())
                        .child(render_status("Status", &self.modeless_status, look))
                        .child(self.modeless_dialog.clone())
                        .into_any_element(),
                ),
                look,
            ),
            DialogDemoKind::Positioning => gallery_pane_with_description(
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
                                .child(self.positioning_center_trigger.clone())
                                .child(self.positioning_corner_trigger.clone())
                                .child(self.positioning_absolute_trigger.clone()),
                        )
                        .child(render_status("Status", &self.positioning_status, look))
                        .child(self.positioning_dialog.clone())
                        .into_any_element(),
                ),
                look,
            ),
            DialogDemoKind::Draggable => gallery_pane_with_description(
                "Dialog: Draggable",
                Some("Single-purpose page for header drag on a modeless tool panel."),
                render_stage(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(16.0))
                        .child(self.draggable_trigger.clone())
                        .child(render_status("Status", &self.draggable_status, look))
                        .child(self.draggable_dialog.clone())
                        .into_any_element(),
                ),
                look,
            ),
        }
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        for entity in [
            &self.modal_trigger,
            &self.modeless_trigger,
            &self.positioning_center_trigger,
            &self.positioning_corner_trigger,
            &self.positioning_absolute_trigger,
            &self.draggable_trigger,
            &self.background_counter,
        ] {
            notify_entity(entity, cx);
        }
        for dialog in [&self.modal_dialog, &self.modeless_dialog, &self.positioning_dialog, &self.draggable_dialog] {
            notify_entity(dialog, cx);
        }
    }

    fn open_modal(&mut self, cx: &mut Context<GalleryApp>) {
        let focus = self.modal_trigger.read(cx).focus_handle(cx);
        self.modal_dialog.update(cx, |dialog, cx| dialog.open_from(Some(focus), cx));
    }

    fn open_modeless(&mut self, cx: &mut Context<GalleryApp>) {
        let focus = self.modeless_trigger.read(cx).focus_handle(cx);
        self.modeless_dialog.update(cx, |dialog, cx| dialog.open_from(Some(focus), cx));
    }

    fn open_positioned(&mut self, position: DialogPosition, cx: &mut Context<GalleryApp>) {
        let focus = self.positioning_center_trigger.read(cx).focus_handle(cx);
        self.positioning_dialog.update(cx, |dialog, cx| {
            dialog.set_position(position, cx);
            dialog.open_from(Some(focus), cx);
        });
    }

    fn open_draggable(&mut self, cx: &mut Context<GalleryApp>) {
        let focus = self.draggable_trigger.read(cx).focus_handle(cx);
        self.draggable_dialog.update(cx, |dialog, cx| {
            dialog.set_position(DialogPosition::Absolute(gpui::point(px(240.0), px(180.0))), cx);
            dialog.open_from(Some(focus), cx);
        });
    }

    fn increment_background(&mut self, cx: &mut Context<GalleryApp>) {
        self.background_clicks += 1;
        self.background_counter.update(cx, |button, cx| {
            button.set_label(format!("Background {}", self.background_clicks), cx);
        });
        self.modeless_status = format!("Background still interactive: {} clicks.", self.background_clicks);
        cx.notify();
    }

    fn handle_modal_event(&mut self, event: &DialogEvent, cx: &mut Context<GalleryApp>) {
        self.modal_status = status_for(event, "modal dialog");
        cx.notify();
    }

    fn handle_modeless_event(&mut self, event: &DialogEvent, cx: &mut Context<GalleryApp>) {
        self.modeless_status = status_for(event, "modeless dialog");
        cx.notify();
    }

    fn handle_positioning_event(&mut self, event: &DialogEvent, cx: &mut Context<GalleryApp>) {
        self.positioning_status = status_for(event, "positioned dialog");
        cx.notify();
    }

    fn handle_draggable_event(&mut self, event: &DialogEvent, cx: &mut Context<GalleryApp>) {
        self.draggable_status = status_for(event, "draggable dialog");
        cx.notify();
    }
}

fn render_stage(content: AnyElement) -> AnyElement {
    div()
        .relative()
        .w_full()
        .max_w(px(860.0))
        .min_h(px(420.0))
        .rounded(px(24.0))
        .border_1()
        .border_color(gpui::transparent_black())
        .occlude()
        .child(div().absolute().inset_0().bg(gpui::hsla(0.58, 0.14, 0.97, 1.0)).opacity(0.72))
        .child(div().relative().size_full().flex().items_center().justify_center().p(px(28.0)).child(content))
        .into_any_element()
}

fn render_status(label: &'static str, value: &str, _look: &ShadcnLook) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(div().text_xs().child(label))
        .child(div().text_sm().child(value.to_string()))
        .into_any_element()
}

fn status_for(event: &DialogEvent, label: &str) -> String {
    match event {
        DialogEvent::Opened => format!("Opened {label}."),
        DialogEvent::Dismissed => format!("Dismissed {label}."),
        DialogEvent::Confirmed => format!("Confirmed {label}."),
    }
}
