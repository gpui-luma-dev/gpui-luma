use gpui::{AnyElement, FontWeight, div, prelude::*, px};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use super::model::{EventReferenceSpec, EVENT_SECTION_INDENT};
use super::template::controls_mono_font;

pub(crate) fn render_event_reference_section(look: &ShadcnLook, specs: &[EventReferenceSpec]) -> AnyElement {
    let chrome = look.chrome();
    let body_style = look.typography_scale(ShadcnTextSize::Xs);
    let mono = controls_mono_font();

    div()
        .w_full()
        .pl(px(EVENT_SECTION_INDENT))
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .typography_style(body_style)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.title_text)
                .child("Possible events"),
        )
        .child(
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(render_event_reference_row(
                    "Event",
                    "Trigger",
                    "Notes",
                    body_style,
                    mono.clone(),
                    chrome.title_text,
                    true,
                    None,
                ))
                .children(specs.iter().map(|spec| {
                    render_event_reference_row(
                        spec.event,
                        spec.trigger,
                        spec.notes,
                        body_style,
                        mono.clone(),
                        chrome.muted_text,
                        false,
                        Some(spec.sampled),
                    )
                })),
        )
        .into_any_element()
}

fn render_event_reference_row(
    event: &'static str,
    trigger: &'static str,
    notes: &'static str,
    body_style: gpui_luma::theme::LumaTextStyle,
    mono: gpui::SharedString,
    text_color: gpui::Hsla,
    header: bool,
    sampled: Option<bool>,
) -> AnyElement {
    div()
        .w_full()
        .grid()
        .grid_cols(12)
        .gap(px(10.0))
        .items_start()
        .child(
            div()
                .col_span(3)
                .font_family(mono)
                .typography_style(body_style)
                .font_weight(if header {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(text_color)
                .child(event),
        )
        .child(
            div()
                .col_span(4)
                .typography_style(body_style)
                .font_weight(if header {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(text_color)
                .child(trigger),
        )
        .child(
            div()
                .col_span(5)
                .flex()
                .items_start()
                .gap(px(8.0))
                .child(div().flex_1().min_w(px(0.0)).typography_style(body_style).text_color(text_color).child(notes))
                .when_some(sampled.and_then(|sampled| sampled.then_some(())), |row, _| {
                    row.child(
                        div()
                            .flex_shrink_0()
                            .typography_style(body_style)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_color)
                            .opacity(0.72)
                            .child("sampled"),
                    )
                }),
        )
        .into_any_element()
}
