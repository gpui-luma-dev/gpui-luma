use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::overlay_window::OverlayWindow;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::model::{ControlExpositionLayout, PublicInterfaceSpec};
use super::public_interface::render_public_interface_section;
use super::template::render_control_exposition_card;

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "OverlayWindow",
        surface: "Type",
        notes: "Entity<OverlayWindowControl> — modal or modeless overlay host.",
    },
    PublicInterfaceSpec {
        symbol: "OverlayWindowEvent",
        surface: "Event",
        notes: "Non-exhaustive enum: Opened, Dismissed.",
    },
    PublicInterfaceSpec {
        symbol: "overlay_window::new(id)",
        surface: "Factory",
        notes: "Starts an OverlayWindowBuilder with the default dialog template.",
    },
    PublicInterfaceSpec {
        symbol: "OverlayWindowBuilder::mode",
        surface: "Builder",
        notes: "OverlayWindowMode::Modal or Modeless — controls backdrop and focus trap.",
    },
    PublicInterfaceSpec {
        symbol: "OverlayWindowBuilder::content(...)",
        surface: "Builder",
        notes: "Closure receiving OverlayWindowRenderModel; returns the panel body.",
    },
    PublicInterfaceSpec {
        symbol: "OverlayWindowBuilder::theme_children([...])",
        surface: "Builder",
        notes: "Register child control entities for theme invalidation fan-out.",
    },
    PublicInterfaceSpec {
        symbol: "OverlayWindowBuilder::dismiss_policy / dismissible",
        surface: "Builder",
        notes: "Click-away and focus-loss dismissal behavior.",
    },
    PublicInterfaceSpec {
        symbol: "OverlayWindowBuilder::position / width / size",
        surface: "Builder",
        notes: "Placement, fixed width override, and ControlSize chrome tier.",
    },
    PublicInterfaceSpec {
        symbol: "OverlayWindowBuilder::with_template_modifier",
        surface: "Builder",
        notes: "Second-tier template customization on the overlay shell.",
    },
    PublicInterfaceSpec {
        symbol: "OverlayWindowBuilder::spawn(cx)",
        surface: "Builder",
        notes: "Materializes the overlay entity; compose trigger + overlay in the view tree.",
    },
    PublicInterfaceSpec {
        symbol: "OverlayWindow::open / dismiss",
        surface: "Entity",
        notes: "Show or hide the overlay; emits Opened or Dismissed.",
    },
    PublicInterfaceSpec {
        symbol: "OverlayWindow::is_open / set_mode / set_dismiss_policy",
        surface: "Entity",
        notes: "Query open state and update behavior after spawn.",
    },
    PublicInterfaceSpec {
        symbol: "look.overlay_window(id)",
        surface: "Look",
        notes: "ShadcnLookControlExt factory — binds the default Shadcn overlay template.",
    },
];

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
                .items_start()
                .gap(px(12.0))
                .child(self.trigger.clone())
                .child(self.overlay.clone())
                .into_any_element();

            render_control_exposition_card(
                &self.look,
                self.entry,
                preview,
                Some(render_public_interface_section(&self.look, PUBLIC_INTERFACE_SPECS)),
                ControlExpositionLayout::BORDERLESS,
            )
        })
    }
}
