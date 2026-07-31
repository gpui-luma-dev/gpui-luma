use std::sync::Arc;

use gpui::{Context, Entity, Focusable, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::overlay_window::{OverlayWindow, OverlayWindowEvent, OverlayWindowMode, OverlayWindowPosition};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::ControlExpositionLayout;
use super::overlay_demo::{overlay_status_for, render_overlay_dialog_panel, render_overlay_status};
use super::overlay_window_inspector_adapter::{OverlayWindowInspectorAdapter, OVERLAY_WINDOW_INSPECTOR_SPEC};
use super::shell_theme_inspectors::OverlayWindowThemeInspector;
use super::template::render_control_exposition_card;

pub struct OverlayPositioningControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<OverlayPositioningExpositionLeftPane>,
    theme_inspector: Entity<OverlayWindowThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct OverlayPositioningExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    center_trigger: Entity<Button>,
    corner_trigger: Entity<Button>,
    absolute_trigger: Entity<Button>,
    close: Entity<Button>,
    overlay: OverlayWindow,
    status: String,
}

impl OverlayPositioningExpositionLeftPane {
    fn open_at(&mut self, position: OverlayWindowPosition, cx: &mut Context<Self>) {
        let focus = self.center_trigger.read(cx).focus_handle(cx);
        self.overlay.update(cx, |overlay, cx| {
            overlay.set_position(position, cx);
            overlay.open_from(Some(focus), cx);
        });
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        for button in [&self.center_trigger, &self.corner_trigger, &self.absolute_trigger, &self.close] {
            button.update(cx, |_, cx| cx.notify());
        }
        self.overlay.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

impl Render for OverlayPositioningExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(12.0))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(8.0))
                        .child(self.center_trigger.clone())
                        .child(self.corner_trigger.clone())
                        .child(self.absolute_trigger.clone()),
                )
                .child(render_overlay_status("Status", &self.status, &self.look))
                .child(self.overlay.clone())
                .into_any_element();

            div()
                .id("controls-doc-overlay-positioning-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    &self.look,
                    self.entry,
                    preview,
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl OverlayPositioningControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("overlay-positioning").expect("overlay-positioning catalog entry");
        let close = look.content_only_icon_button("controls-doc-positioning-close", LucideIcon::X).spawn(cx);
        let overlay = look
            .overlay_window("controls-doc-positioning-overlay")
            .mode(OverlayWindowMode::Modeless)
            .content({
                let close = close.clone();
                let look = look.clone();
                move |model, _, _| {
                    render_overlay_dialog_panel(
                        &look,
                        "Positioned overlay",
                        Some(close.clone()),
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child("Reopen this surface at different anchors using the launch buttons.")
                            .child(format!("Current position: {:?}", model.position)),
                    )
                }
            })
            .theme_children([close.clone()])
            .spawn(cx);
        let center_trigger = look.secondary_button("controls-doc-position-center").label("Center").spawn(cx);
        let corner_trigger = look.secondary_button("controls-doc-position-corner").label("Top right").spawn(cx);
        let absolute_trigger = look.secondary_button("controls-doc-position-absolute").label("Absolute").spawn(cx);

        let left_pane = cx.new(|_| OverlayPositioningExpositionLeftPane {
            look: look.clone(),
            entry,
            center_trigger: center_trigger.clone(),
            corner_trigger: corner_trigger.clone(),
            absolute_trigger: absolute_trigger.clone(),
            close: close.clone(),
            overlay: overlay.clone(),
            status: "Closed".into(),
        });

        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-overlay-positioning-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &OVERLAY_WINDOW_INSPECTOR_SPEC,
            OverlayWindowInspectorAdapter::shared(),
        );

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&center_trigger, {
            let left_pane = left_pane.clone();
            move |_, _, event: &ButtonEvent, cx| {
                if !event.is_click() {
                    return;
                }
                left_pane.update(cx, |pane, cx| pane.open_at(OverlayWindowPosition::Center, cx));
            }
        }));
        subscriptions.push(cx.subscribe(&corner_trigger, {
            let left_pane = left_pane.clone();
            move |_, _, event: &ButtonEvent, cx| {
                if !event.is_click() {
                    return;
                }
                left_pane.update(cx, |pane, cx| pane.open_at(OverlayWindowPosition::TopRight, cx));
            }
        }));
        subscriptions.push(cx.subscribe(&absolute_trigger, {
            let left_pane = left_pane.clone();
            move |_, _, event: &ButtonEvent, cx| {
                if !event.is_click() {
                    return;
                }
                left_pane.update(cx, |pane, cx| {
                    pane.open_at(OverlayWindowPosition::Absolute(gpui::point(px(180.0), px(280.0))), cx);
                });
            }
        }));
        subscriptions.push(cx.subscribe(&close, {
            let left_pane = left_pane.clone();
            move |_, _, event: &ButtonEvent, cx| {
                if !event.is_click() {
                    return;
                }
                left_pane.update(cx, |pane, cx| {
                    pane.status = "Dismissed positioned overlay.".into();
                    pane.overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
                    cx.notify();
                });
            }
        }));
        subscriptions.push(cx.subscribe(&overlay, {
            let left_pane = left_pane.clone();
            move |_, _, event: &OverlayWindowEvent, cx| {
                left_pane.update(cx, |pane, cx| {
                    pane.status = overlay_status_for(event, "positioned overlay");
                    cx.notify();
                });
            }
        }));

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: subscriptions }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for OverlayPositioningControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-overlay-positioning-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}
