use gpui::{AnyElement, FontWeight, div, prelude::*, px};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use super::event_reference::render_event_reference_section;
use super::model::{EventReferenceSpec, PublicInterfaceSpec};
use super::template::controls_mono_font;

pub(crate) fn render_exposition_doc_sections(
    look: &ShadcnLook,
    event_specs: &[EventReferenceSpec],
    public_specs: &[PublicInterfaceSpec],
) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(16.0))
        .child(render_event_reference_section(look, event_specs))
        .child(render_public_interface_section(look, public_specs))
        .into_any_element()
}

pub(crate) fn render_public_interface_section(look: &ShadcnLook, specs: &[PublicInterfaceSpec]) -> AnyElement {
    let chrome = look.chrome();
    let body_style = look.typography_scale(ShadcnTextSize::Xs);
    let mono = controls_mono_font();

    div()
        .w_full()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(6.0))
        .child(
            div()
                .typography_style(body_style)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.title_text)
                .child("Public interface"),
        )
        .child(
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(render_public_interface_row(
                    "Symbol",
                    "Surface",
                    "Notes",
                    body_style,
                    mono.clone(),
                    chrome.title_text,
                    true,
                ))
                .children(specs.iter().map(|spec| {
                    render_public_interface_row(
                        spec.symbol,
                        spec.surface,
                        spec.notes,
                        body_style,
                        mono.clone(),
                        chrome.muted_text,
                        false,
                    )
                })),
        )
        .into_any_element()
}

fn render_public_interface_row(
    symbol: &'static str,
    surface: &'static str,
    notes: &'static str,
    body_style: gpui_luma::theme::LumaTextStyle,
    mono: gpui::SharedString,
    text_color: gpui::Hsla,
    header: bool,
) -> AnyElement {
    div()
        .w_full()
        .grid()
        .grid_cols(12)
        .gap(px(10.0))
        .items_start()
        .child(
            div()
                .col_span(5)
                .flex_shrink_0()
                .overflow_hidden()
                .font_family(mono)
                .typography_style(body_style)
                .font_weight(if header {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(text_color)
                .whitespace_nowrap()
                .child(symbol),
        )
        .child(
            div()
                .col_span(3)
                .typography_style(body_style)
                .font_weight(if header {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(text_color)
                .child(surface),
        )
        .child(
            div()
                .col_span(4)
                .min_w(px(0.0))
                .typography_style(body_style)
                .font_weight(if header {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(text_color)
                .child(notes),
        )
        .into_any_element()
}
