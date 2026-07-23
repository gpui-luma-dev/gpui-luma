use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::overlay_window::OverlayWindow;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::model::ControlExpositionLayout;
use super::template::render_control_exposition_card;

pub struct ModalOverlayControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    trigger: Entity<Button>,
    cancel: Entity<Button>,
    confirm: Entity<Button>,
    overlay: OverlayWindow,
    _subscriptions: Vec<Subscription>,
}

impl ModalOverlayControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("modal-overlay").expect("modal-overlay catalog entry");
        let cancel = look.outline_button("controls-doc-dialog-cancel").label("Cancel").spawn(cx);
        let confirm = look.primary_button("controls-doc-dialog-confirm").label("Confirm").spawn(cx);
        let overlay = look
            .overlay_window("controls-doc-dialog-overlay")
            .mode(gpui_luma::controls::overlay_window::OverlayWindowMode::Modal)
            .content({
                let cancel = cancel.clone();
                let confirm = confirm.clone();
                move |_, _, _| {
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .p(px(20.0))
                        .child("Archive this project? This action cannot be undone.")
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .justify_end()
                                .gap(px(8.0))
                                .child(cancel.clone())
                                .child(confirm.clone()),
                        )
                        .into_any_element()
                }
            })
            .theme_children([cancel.clone(), confirm.clone()])
            .spawn(cx);
        let trigger = look.primary_button("controls-doc-dialog-trigger").label("Open modal").spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&trigger, {
            let overlay = overlay.clone();
            move |_, _, event: &ButtonEvent, cx| {
                if !event.is_click() {
                    return;
                }
                overlay.update(cx, |overlay, cx| overlay.open(cx));
            }
        }));
        subscriptions.push(cx.subscribe(&cancel, {
            let overlay = overlay.clone();
            move |_, _, event: &ButtonEvent, cx| {
                if !event.is_click() {
                    return;
                }
                overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
            }
        }));
        subscriptions.push(cx.subscribe(&confirm, {
            let overlay = overlay.clone();
            move |_, _, event: &ButtonEvent, cx| {
                if !event.is_click() {
                    return;
                }
                overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
            }
        }));

        Self { look, entry, trigger, cancel, confirm, overlay, _subscriptions: subscriptions }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        for button in [&self.trigger, &self.cancel, &self.confirm] {
            button.update(cx, |_, cx| cx.notify());
        }
        self.overlay.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

impl Render for ModalOverlayControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(12.0))
                .child(self.trigger.clone())
                .child(self.overlay.clone())
                .into_any_element();

            render_control_exposition_card(&self.look, self.entry, preview, None, ControlExpositionLayout::BORDERLESS)
        })
    }
}
