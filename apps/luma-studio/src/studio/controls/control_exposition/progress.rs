//! Progress control exposition — gallery-aligned interactive bar and static value samples.

use std::sync::Arc;

use gpui::{App, Context, Entity, Render, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::progress::{Progress, ProgressRenderModel, ProgressTemplate};
use gpui_luma::controls::value::ControlRange;
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout, PublicInterfaceSpec};
use super::progress_inspector_adapter::{ProgressInspectorAdapter, PROGRESS_INSPECTOR_SPEC};
use super::public_interface::render_exposition_doc_sections;
use super::standalone_theme_inspectors::ProgressThemeInspector;
use super::template::render_control_exposition_card;

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "Progress",
        surface: "Type",
        notes: "Entity<ProgressControl> — read-only progress indicator; no user events.",
    },
    PublicInterfaceSpec {
        symbol: "look.progress(id)",
        surface: "Look",
        notes: "ShadcnLookControlExt factory with range, value, and size.",
    },
    PublicInterfaceSpec {
        symbol: "ProgressControl::set_value / set_enabled",
        surface: "API",
        notes: "Update programmatically from parent state; template re-renders on notify.",
    },
];

pub struct ProgressControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ProgressExpositionLeftPane>,
    theme_inspector: Entity<ProgressThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct ProgressExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    progress: Progress,
}

impl ProgressExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.progress.update(cx, |progress, cx| {
            progress.set_template(look.progress_template(), cx);
        });
        cx.notify();
    }
}

impl Render for ProgressExpositionLeftPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();
            let template = look.progress_template();

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(
                    div()
                        .w(px(360.0))
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .child(section_label("Interactive", chrome.muted_text))
                        .child(self.progress.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .child(section_label("Value samples", chrome.muted_text))
                        .child(
                            div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                                [
                                    ("empty", "Empty", 0.0, true),
                                    ("quarter", "25%", 25.0, true),
                                    ("half", "50%", 50.0, true),
                                    ("complete", "Complete", 100.0, true),
                                    ("disabled", "Disabled", 50.0, false),
                                ]
                                .into_iter()
                                .map(|(id, label, value, enabled)| {
                                    render_progress_sample(
                                        ProgressSample {
                                            template: &template,
                                            id,
                                            label,
                                            value,
                                            enabled,
                                            label_color: chrome.muted_text,
                                        },
                                        window,
                                        cx,
                                    )
                                }),
                            ),
                        ),
                );

            div()
                .id("controls-doc-progress-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    look,
                    self.entry,
                    preview.into_any_element(),
                    Some(render_exposition_doc_sections(look, &[], PUBLIC_INTERFACE_SPECS)),
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl ProgressControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("progress").expect("progress catalog entry");
        let progress = look.progress("controls-doc-progress").range(1..100).value(41).spawn(cx);
        let left_pane = cx.new(|_| ProgressExpositionLeftPane { look: look.clone(), entry, progress });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-progress-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &PROGRESS_INSPECTOR_SPEC,
            ProgressInspectorAdapter::shared(),
        );

        Self { look, entry, left_pane, theme_inspector, inspector_split }
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

impl Render for ProgressControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-progress-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn section_label(label: &'static str, color: gpui::Hsla) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(color)
        .child(label)
}

struct ProgressSample<'a> {
    template: &'a Arc<dyn ProgressTemplate>,
    id: &'a str,
    label: &'static str,
    value: f32,
    enabled: bool,
    label_color: gpui::Hsla,
}

fn render_progress_sample(sample: ProgressSample<'_>, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
    let ProgressSample { template, id, label, value, enabled, label_color } = sample;
    let id = SharedString::from(format!("controls-doc-progress-{id}"));
    let range = ControlRange::from(0..100);
    let model = ProgressRenderModel {
        id: &id,
        range,
        value,
        percentage: range.percentage(value),
        size: ControlSize::Md,
        enabled,
    };

    div()
        .w(px(120.0))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(label))
        .into_any_element()
}
