use gpui::{AnyElement, FontWeight, div, prelude::*, px};
use gpui_luma::{GridLayout, GridTrack};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use super::event_reference::render_event_reference_section;
use super::model::{EventReferenceSpec, PublicInterfaceSpec};
use super::template::controls_mono_font;

const INTERFACE_SURFACE_WIDTH: f32 = 48.0;
const INTERFACE_GRID_GAP_X: f32 = 10.0;
const INTERFACE_GRID_GAP_Y: f32 = 8.0;

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
    render_public_interface_block(look, None, &[("", specs)])
}

pub(crate) fn render_public_interface_block(
    look: &ShadcnLook,
    intro: Option<&str>,
    groups: &[(&str, &[PublicInterfaceSpec])],
) -> AnyElement {
    let chrome = look.chrome();
    let body_style = look.typography_scale(ShadcnTextSize::Xs);
    let mono = controls_mono_font();

    let mut root = div().w_full().flex().flex_col().items_start().gap(px(10.0)).child(
        div()
            .typography_style(body_style)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(chrome.title_text)
            .child("Public interface"),
    );

    if let Some(intro) = intro {
        root = root.child(div().typography_style(body_style).text_color(chrome.muted_text).child(intro.to_string()));
    }

    for (index, (title, specs)) in groups.iter().enumerate() {
        if specs.is_empty() {
            continue;
        }

        let mut group = div().w_full().flex().flex_col().items_start().gap(px(6.0));

        if !title.is_empty() {
            group = group.child(
                div()
                    .typography_style(body_style)
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(chrome.title_text)
                    .child(title.to_string()),
            );
        }

        group = group.child(render_public_interface_table(
            body_style,
            mono.clone(),
            chrome.title_text,
            chrome.muted_text,
            specs,
        ));

        if index > 0 {
            group = group.mt(px(12.0));
        }

        root = root.child(group);
    }

    root.into_any_element()
}

fn render_public_interface_table(
    body_style: gpui_luma::theme::LumaTextStyle,
    mono: gpui::SharedString,
    header_color: gpui::Hsla,
    row_color: gpui::Hsla,
    specs: &[PublicInterfaceSpec],
) -> AnyElement {
    let row_count = specs.len() + 1;
    let mut grid = GridLayout::new()
        .rows(row_count)
        .columns([GridTrack::Star(2.0), GridTrack::Px(INTERFACE_SURFACE_WIDTH), GridTrack::Star(3.0)])
        .gap_x(INTERFACE_GRID_GAP_X)
        .gap_y(INTERFACE_GRID_GAP_Y);

    grid = grid
        .child(interface_cell("Symbol", body_style, None, header_color, FontWeight::SEMIBOLD), 0, 0)
        .child(interface_cell("Surface", body_style, None, header_color, FontWeight::SEMIBOLD), 0, 1)
        .child(interface_cell("Notes", body_style, None, header_color, FontWeight::SEMIBOLD), 0, 2);

    for (index, spec) in specs.iter().enumerate() {
        let row = index + 1;
        grid = grid
            .child(interface_cell(spec.symbol, body_style, Some(mono.clone()), row_color, FontWeight::NORMAL), row, 0)
            .child(interface_cell(spec.surface, body_style, None, row_color, FontWeight::NORMAL), row, 1)
            .child(interface_cell(spec.notes, body_style, None, row_color, FontWeight::NORMAL), row, 2);
    }

    grid.into_any_element()
}

fn interface_cell(
    text: &str,
    body_style: gpui_luma::theme::LumaTextStyle,
    mono: Option<gpui::SharedString>,
    text_color: gpui::Hsla,
    weight: FontWeight,
) -> AnyElement {
    let mut cell = div().min_w(px(0.0)).typography_style(body_style).font_weight(weight).text_color(text_color);

    if let Some(mono) = mono {
        cell = cell.font_family(mono);
    }

    cell.child(text.to_string()).into_any_element()
}
