use std::sync::Arc;

use gpui::{AnyElement, App, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::progress::{ProgressRenderModel, ProgressTemplate};
use gpui_luma::controls::value::ControlRange;
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};
use lucide_icons::Icon as LucideIcon;

use crate::studio::style::shared::shell::section_shell_with_width;

#[derive(Clone, Copy)]
struct ProgressStateSample {
    id: &'static str,
    label: &'static str,
    value: f32,
    enabled: bool,
}

pub(crate) fn render_feedback_template_section(look: Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let progress_template = look.progress_template();

    section_shell_with_width(
        960.0,
        "Feedback",
        "Badge and progress previews.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .child(render_feedback_heading("Badges", chrome.muted_text))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(px(14.0))
                    .child(div().flex().flex_wrap().items_center().gap(px(10.0)).children([
                        look.badge("work").variant(BadgeVariant::Default).into_any_element(),
                        look.badge("budget").variant(BadgeVariant::Outline).into_any_element(),
                    ]))
                    .child(div().flex().flex_wrap().items_center().gap(px(10.0)).children([
                        look.badge("Default").variant(BadgeVariant::Default).into_any_element(),
                        look.badge("Secondary").variant(BadgeVariant::Secondary).into_any_element(),
                        look.badge("Outline").variant(BadgeVariant::Outline).into_any_element(),
                        look.badge("Ghost").variant(BadgeVariant::Ghost).into_any_element(),
                    ]))
                    .child(div().flex().flex_wrap().items_center().gap(px(10.0)).children([
                        look.badge("Small").size(ControlSize::Sm).variant(BadgeVariant::Secondary).into_any_element(),
                        look.badge("Medium").size(ControlSize::Md).variant(BadgeVariant::Secondary).into_any_element(),
                        look.badge("Large").size(ControlSize::Lg).variant(BadgeVariant::Secondary).into_any_element(),
                    ]))
                    .child(
                        div().flex().flex_wrap().items_center().gap(px(10.0)).children([
                            look.badge("Verified")
                                .variant(BadgeVariant::Default)
                                .start_icon(LucideIcon::BadgeCheck)
                                .into_any_element(),
                            look.badge("Updated")
                                .variant(BadgeVariant::Secondary)
                                .end_icon(LucideIcon::ArrowUpRight)
                                .into_any_element(),
                            look.badge("Draft")
                                .variant(BadgeVariant::Outline)
                                .start_icon(LucideIcon::Pencil)
                                .into_any_element(),
                            look.badge("Muted")
                                .variant(BadgeVariant::Ghost)
                                .end_icon(LucideIcon::Dot)
                                .into_any_element(),
                        ]),
                    ),
            )
            .child(render_feedback_heading("Progress", chrome.muted_text))
            .child(div().flex().flex_wrap().items_start().gap(px(12.0)).children(
                feedback_progress_samples().into_iter().map(|sample| {
                    render_feedback_progress_sample(&progress_template, sample, chrome.muted_text, window, cx)
                }),
            ))
            .into_any_element(),
    )
}

fn render_feedback_heading(title: &'static str, muted_text: gpui::Hsla) -> AnyElement {
    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(muted_text)
        .child(title)
        .into_any_element()
}

fn render_feedback_progress_sample(
    template: &Arc<dyn ProgressTemplate>,
    sample: ProgressStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("theme-studio-feedback-progress-{}", sample.id));
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

fn feedback_progress_samples() -> [ProgressStateSample; 5] {
    [
        ProgressStateSample { id: "empty", label: "Empty", value: 0.0, enabled: true },
        ProgressStateSample { id: "quarter", label: "25%", value: 25.0, enabled: true },
        ProgressStateSample { id: "half", label: "50%", value: 50.0, enabled: true },
        ProgressStateSample { id: "complete", label: "Complete", value: 100.0, enabled: true },
        ProgressStateSample { id: "disabled", label: "Disabled", value: 50.0, enabled: false },
    ]
}
