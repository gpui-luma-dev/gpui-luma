use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::progress::{Progress, ProgressRenderModel, ProgressTemplate};
use gpui_luma::controls::value::ControlRange;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_progress_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ProgressPane {
    progress: Progress,
    state_preview: Entity<ProgressStatePreview>,
    inspector: Entity<ColorInspectorShell>,
}

impl ProgressPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree("progress-inspector-tree", look.clone(), build_progress_inspect_tree, cx);
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "progress-inspector",
                "progress-inspector-split",
                "progress-inspector-detail",
                build_progress_inspect_tree,
                cx,
            )
        });
        Self {
            progress: look.progress("progress-example").range(1..100).value(41).spawn(cx),
            state_preview: cx.new(|_| ProgressStatePreview::new(look)),
            inspector,
        }
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        gallery_pane_with_inspector(
            "Progress",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_4()
                .child(self.progress.clone())
                .child(self.state_preview.clone())
                .into_any_element(),
            self.inspector.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.progress, cx);
        notify_entity(&self.state_preview, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }
}

#[derive(Clone)]
struct ProgressStatePreview {
    look: Arc<ShadcnLook>,
    template: Arc<dyn ProgressTemplate>,
}

struct ProgressStateSample {
    id: &'static str,
    label: &'static str,
    value: f32,
    enabled: bool,
}

impl ProgressStatePreview {
    fn new(look: Arc<ShadcnLook>) -> Self {
        Self { template: look.progress_template(), look }
    }
}

impl Render for ProgressStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let samples = [
            ProgressStateSample { id: "empty", label: "Empty", value: 0.0, enabled: true },
            ProgressStateSample { id: "quarter", label: "25%", value: 25.0, enabled: true },
            ProgressStateSample { id: "half", label: "50%", value: 50.0, enabled: true },
            ProgressStateSample { id: "complete", label: "Complete", value: 100.0, enabled: true },
            ProgressStateSample { id: "disabled", label: "Disabled", value: 50.0, enabled: false },
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(10.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template state preview"),
            )
            .child(
                div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                    samples.into_iter().map(|sample| {
                        render_progress_state_sample(&self.template, sample, chrome.muted_text, window, cx)
                    }),
                ),
            )
    }
}

fn render_progress_state_sample(
    template: &Arc<dyn ProgressTemplate>,
    sample: ProgressStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("progress-preview-{}", sample.id));
    let range = ControlRange::from(0..100);
    let percentage = range.percentage(sample.value);
    let model = ProgressRenderModel { id: &id, range, value: sample.value, percentage, enabled: sample.enabled };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}
