use std::sync::Arc;

use gpui::{Context, Entity, Focusable, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::overlay_window::{OverlayWindow, OverlayWindowEvent, OverlayWindowMode, OverlayWindowPosition};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::ControlExpositionLayout;
use super::overlay_demo::{overlay_status_for, render_overlay_dialog_panel, render_overlay_status};
use super::overlay_window_inspector_adapter::{OverlayWindowInspectorAdapter, OVERLAY_WINDOW_INSPECTOR_SPEC};
use super::shell_theme_inspectors::OverlayWindowThemeInspector;
use super::template::render_control_exposition_card;

pub struct DraggableOverlayControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<DraggableOverlayExpositionLeftPane>,
    theme_inspector: Entity<OverlayWindowThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct DraggableOverlayExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    trigger: Entity<Button>,
    close: Entity<Button>,
    overlay: OverlayWindow,
    status: String,
}

impl DraggableOverlayExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        for button in [&self.trigger, &self.close] {
            button.update(cx, |_, cx| cx.notify());
        }
        self.overlay.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

impl Render for DraggableOverlayExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(12.0))
                .child(self.trigger.clone())
                .child(render_overlay_status("Status", &self.status, &self.look))
                .child(self.overlay.clone())
                .into_any_element();

            div()
                .id("controls-doc-draggable-overlay-left-pane")
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

impl DraggableOverlayControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("draggable-overlay").expect("draggable-overlay catalog entry");
        let close = look.content_only_icon_button("controls-doc-draggable-close", LucideIcon::X).spawn(cx);
        let overlay = look
            .overlay_window("controls-doc-draggable-overlay")
            .mode(OverlayWindowMode::Modeless)
            .absolute_position(240.0, 180.0)
            .draggable(true)
            .content({
                let close = close.clone();
                let look = look.clone();
                move |_, _, _| {
                    render_overlay_dialog_panel(
                        &look,
                        "Draggable tool palette",
                        Some(close.clone()),
                        "Drag the panel by its top strip to reposition it.",
                    )
                }
            })
            .theme_children([close.clone()])
            .spawn(cx);
        let trigger = look.primary_button("controls-doc-draggable-trigger").label("Open draggable").spawn(cx);

        let left_pane = cx.new(|_| DraggableOverlayExpositionLeftPane {
            look: look.clone(),
            entry,
            trigger: trigger.clone(),
            close: close.clone(),
            overlay: overlay.clone(),
            status: "Closed".into(),
        });

        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-draggable-overlay-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &OVERLAY_WINDOW_INSPECTOR_SPEC,
            OverlayWindowInspectorAdapter::shared(),
        );

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&trigger, {
            let left_pane = left_pane.clone();
            move |_, _, event: &ButtonEvent, cx| {
                if !event.is_click() {
                    return;
                }
                left_pane.update(cx, |pane, cx| {
                    let focus = pane.trigger.read(cx).focus_handle(cx);
                    pane.overlay.update(cx, |overlay, cx| {
                        overlay.set_position(OverlayWindowPosition::Absolute(gpui::point(px(240.0), px(180.0))), cx);
                        overlay.set_draggable(true, cx);
                        overlay.open_from(Some(focus), cx);
                    });
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
                    pane.status = "Dismissed draggable overlay.".into();
                    pane.overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
                    cx.notify();
                });
            }
        }));
        subscriptions.push(cx.subscribe(&overlay, {
            let left_pane = left_pane.clone();
            move |_, _, event: &OverlayWindowEvent, cx| {
                left_pane.update(cx, |pane, cx| {
                    pane.status = overlay_status_for(event, "draggable overlay");
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

impl Render for DraggableOverlayControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-draggable-overlay-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}
