//! Variant-by-state matrix shared by style guide sections.

use gpui::{AnyElement, FontWeight, Hsla, IntoElement, SharedString, div, prelude::*, px};

pub struct TableStyle {
    pub variant_column_width: f32,
    pub state_column_width: f32,
    pub header_height: f32,
    pub row_height: f32,
    pub title: Hsla,
    pub muted: Hsla,
}

impl TableStyle {
    pub fn new(title: Hsla, muted: Hsla) -> Self {
        Self {
            variant_column_width: 188.0,
            state_column_width: 92.0,
            header_height: 48.0,
            row_height: 52.0,
            title,
            muted,
        }
    }
}

pub struct TableRow {
    pub label: SharedString,
    pub description: SharedString,
    pub cells: Vec<AnyElement>,
}

pub fn render(
    style: &TableStyle,
    group_label: impl Into<SharedString>,
    headers: Vec<AnyElement>,
    rows: Vec<TableRow>,
) -> AnyElement {
    let width = style.variant_column_width + style.state_column_width * headers.len() as f32;

    div()
        .w(px(width))
        .max_w_full()
        .flex_shrink_0()
        .child(header_row(style, group_label.into(), headers))
        .children(rows.into_iter().map(|row| variant_row(style, row)))
        .into_any_element()
}

fn header_row(style: &TableStyle, group_label: SharedString, headers: Vec<AnyElement>) -> AnyElement {
    div()
        .flex()
        .items_stretch()
        .child(cell(
            style.variant_column_width,
            style.header_height,
            div()
                .w_full()
                .h_full()
                .flex()
                .flex_col()
                .justify_end()
                .items_start()
                .px(px(16.0))
                .pb(px(8.0))
                .text_xs()
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(style.muted)
                .child(group_label),
        ))
        .children(headers.into_iter().map(|header| cell(style.state_column_width, style.header_height, header)))
        .into_any_element()
}

fn variant_row(style: &TableStyle, row: TableRow) -> AnyElement {
    div()
        .flex()
        .items_stretch()
        .child(cell(style.variant_column_width, style.row_height, variant_label(style, row.label, row.description)))
        .children(row.cells.into_iter().map(|content| {
            cell(
                style.state_column_width,
                style.row_height,
                div().w_full().h_full().flex().items_center().justify_center().child(content),
            )
        }))
        .into_any_element()
}

fn variant_label(style: &TableStyle, label: SharedString, description: SharedString) -> AnyElement {
    div()
        .w_full()
        .min_w(px(0.0))
        .px(px(16.0))
        .py(px(8.0))
        .flex()
        .flex_col()
        .justify_center()
        .gap(px(2.0))
        .child(
            div()
                .text_sm()
                .line_height(px(18.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(style.title)
                .truncate()
                .child(label),
        )
        .children(
            (!description.is_empty())
                .then(|| div().text_xs().line_height(px(15.0)).text_color(style.muted).truncate().child(description)),
        )
        .into_any_element()
}

fn cell(width: f32, min_height: f32, content: impl IntoElement) -> AnyElement {
    div()
        .w(px(width))
        .min_h(px(min_height))
        .flex_shrink_0()
        .overflow_hidden()
        .flex()
        .items_stretch()
        .child(content)
        .into_any_element()
}
