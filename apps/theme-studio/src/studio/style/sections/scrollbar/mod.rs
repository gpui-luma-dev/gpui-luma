use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarRenderModel, ScrollbarTemplate};
use gpui_luma::controls::value::ControlRange;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::style::shared::input_samples::{InputInteractionSample, input_interaction_samples};
use crate::studio::style::shared::preview_handlers::input_scrollbar_handlers;
use crate::studio::style::shared::shell::section_shell_with_width;

pub(crate) fn render_scrollbar_template_section(
    look: Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    section_shell_with_width(
        960.0,
        "Scrollbar",
        "Horizontal and vertical scrollbar interaction states.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        render_input_scrollbar_body(&look, window, cx),
    )
}

fn render_input_scrollbar_body(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let samples = input_interaction_samples();
    let template = look.scrollbar_template();

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(12.0))
        .child(render_input_scrollbar_row(
            &template,
            "Horizontal",
            ScrollbarOrientation::Horizontal,
            &samples,
            chrome.muted_text,
            window,
            cx,
        ))
        .child(render_input_scrollbar_row(
            &template,
            "Vertical",
            ScrollbarOrientation::Vertical,
            &samples,
            chrome.muted_text,
            window,
            cx,
        ))
        .into_any_element()
}

fn render_input_scrollbar_row(
    template: &Arc<dyn ScrollbarTemplate>,
    row_label: &'static str,
    orientation: ScrollbarOrientation,
    samples: &[InputInteractionSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(row_label))
        .child(
            div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                samples.iter().copied().map(|sample| {
                    render_input_scrollbar_sample(template, orientation, sample, label_color, window, cx)
                }),
            ),
        )
        .into_any_element()
}

fn render_input_scrollbar_sample(
    template: &Arc<dyn ScrollbarTemplate>,
    orientation: ScrollbarOrientation,
    sample: InputInteractionSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let orientation_id = match orientation {
        ScrollbarOrientation::Horizontal => "horizontal",
        ScrollbarOrientation::Vertical => "vertical",
    };
    let id = SharedString::from(format!("theme-studio-scrollbar-preview-{}-{}", orientation_id, sample.id));
    let range = ControlRange::from(0..220);
    let value = 40.0;
    let model = ScrollbarRenderModel {
        id: &id,
        orientation,
        range,
        step: 20.0,
        page_step: 80.0,
        value,
        percentage: range.percentage(value),
        thumb_fraction: 0.54,
        length: None,
        enabled: !sample.state.disabled,
        state: sample.state,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, input_scrollbar_handlers(), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}
