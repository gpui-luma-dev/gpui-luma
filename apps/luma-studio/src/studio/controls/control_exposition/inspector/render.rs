#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{AnyElement, FontWeight, IntoElement, div, px};
use gpui::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use super::box_model::{MetricFieldHighlight, render_box_model_diagram};
use super::components::{InspectorRow, InspectorSection};
use super::schema::{
    InspectColorRow, InspectElevationSnapshot, InspectLayoutSection, InspectPropertyRow, InspectorCategoryContent,
};
use crate::studio::color_format::format_compact_hsla;
use crate::studio::controls::control_exposition::template::controls_mono_font;

pub mod layout {
    pub const PANEL_PADDING: f32 = 12.0;
    pub const DETAIL_GAP: f32 = 8.0;
}

pub fn render_category_content(look: &Arc<ShadcnLook>, content: InspectorCategoryContent) -> AnyElement {
    match content {
        InspectorCategoryContent::Colors(rows) => render_color_category(look, rows),
        InspectorCategoryContent::Layout(section) => render_layout_category(look, section),
        InspectorCategoryContent::Elevation(elevation) => render_elevation_category(look, elevation),
        InspectorCategoryContent::Typography(rows) => render_typography_category(look, rows),
    }
}

pub fn render_color_category(look: &Arc<ShadcnLook>, rows: Vec<InspectColorRow>) -> AnyElement {
    InspectorSection::new(
        look,
        rows.into_iter().enumerate().map(|(index, row)| render_color_row(row, look, index > 0)).collect(),
    )
    .into_any_element()
}

pub fn render_layout_category(look: &Arc<ShadcnLook>, section: InspectLayoutSection) -> AnyElement {
    let chrome = look.chrome();
    let caption = &look.mode_tokens().typography.text.caption;
    let mono_font = controls_mono_font();

    div()
        .w_full()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(layout::DETAIL_GAP))
        .child(render_box_model_diagram(
            section.box_model_diagram_id,
            &section.box_model,
            section.occupation.as_ref(),
            MetricFieldHighlight::None,
            section.box_model_colors,
            chrome.border,
            section.box_model_label_color,
            caption,
            mono_font,
        ))
        .child(render_property_rows_panel(look, section.metrics))
        .into_any_element()
}

pub fn render_typography_category(look: &Arc<ShadcnLook>, rows: Vec<InspectPropertyRow>) -> AnyElement {
    render_property_rows_panel(look, rows)
}

pub fn render_elevation_category(look: &Arc<ShadcnLook>, elevation: InspectElevationSnapshot) -> AnyElement {
    let chrome = look.chrome();
    let body = &look.mode_tokens().typography.text.body;
    let caption = &look.mode_tokens().typography.text.caption;
    let mono_font = controls_mono_font();

    let mut preview_chip = div()
        .w(px(120.0))
        .h(px(36.0))
        .rounded(px(6.0))
        .bg(chrome.panel_background)
        .border_1()
        .border_color(chrome.border);

    if let Some(shadows) = elevation.preview_shadows.as_ref().filter(|shadows| !shadows.is_empty()) {
        preview_chip = preview_chip.shadow(shadows.clone());
    }

    div()
        .w_full()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(layout::DETAIL_GAP))
        .child(
            div()
                .w_full()
                .border_1()
                .border_color(chrome.border)
                .rounded(px(6.0))
                .bg(chrome.content_background)
                .p(px(10.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    div()
                        .text_size(px(body.size))
                        .line_height(px(body.line_height))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.title_text)
                        .child("Shadow preview"),
                )
                .child(div().w_full().h(px(78.0)).flex().items_center().justify_center().child(preview_chip)),
        )
        .child(render_property_rows_panel(look, elevation.property_rows))
        .when_some(elevation.catalog_value.clone(), |stack, catalog_value| {
            stack.child(
                div()
                    .w_full()
                    .border_1()
                    .border_color(chrome.border)
                    .rounded(px(6.0))
                    .bg(chrome.content_background)
                    .p(px(10.0))
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(body.size))
                            .line_height(px(body.line_height))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.title_text)
                            .child("Catalog value"),
                    )
                    .child(
                        div()
                            .font_family(mono_font.clone())
                            .text_size(px(caption.size))
                            .line_height(px(caption.line_height))
                            .text_color(chrome.muted_text)
                            .child(catalog_value),
                    ),
            )
        })
        .child(render_elevation_layers(look, &elevation.layers))
        .into_any_element()
}

fn render_elevation_layers(look: &Arc<ShadcnLook>, layers: &[super::schema::InspectElevationLayer]) -> AnyElement {
    let chrome = look.chrome();
    let body = &look.mode_tokens().typography.text.body;
    let caption = &look.mode_tokens().typography.text.caption;
    let mono_font = controls_mono_font();

    let mut stack = div()
        .w_full()
        .min_w(px(0.0))
        .border_1()
        .border_color(chrome.border)
        .rounded(px(6.0))
        .bg(chrome.content_background)
        .p(px(10.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .font_weight(FontWeight::MEDIUM)
                .text_color(chrome.title_text)
                .child("Shadow layers"),
        );

    if layers.is_empty() {
        stack = stack.child(
            div()
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .text_color(chrome.muted_text)
                .child("No shadow layers resolved for this state."),
        );
    } else {
        for (index, layer) in layers.iter().enumerate() {
            stack = stack.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(body.size))
                            .line_height(px(body.line_height))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.title_text)
                            .child(format!("layer {}", index + 1)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .size(px(12.0))
                                    .flex_shrink_0()
                                    .rounded_full()
                                    .bg(layer.color)
                                    .border_1()
                                    .border_color(chrome.border),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.0))
                                    .font_family(mono_font.clone())
                                    .text_size(px(caption.size))
                                    .line_height(px(caption.line_height))
                                    .text_color(chrome.muted_text)
                                    .child(layer.display.clone()),
                            ),
                    ),
            );
        }
    }

    stack.into_any_element()
}

pub fn render_property_rows_panel(look: &Arc<ShadcnLook>, rows: Vec<InspectPropertyRow>) -> AnyElement {
    InspectorSection::new(
        look,
        rows.into_iter().enumerate().map(|(index, row)| render_property_row(row, look, index > 0)).collect(),
    )
    .into_any_element()
}

fn render_color_row(row: InspectColorRow, look: &Arc<ShadcnLook>, show_separator: bool) -> AnyElement {
    let chrome = look.chrome();
    let body = &look.mode_tokens().typography.text.body;
    let source = row.detail.unwrap_or(row.source);

    let label = div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .w(px(18.0))
                .h(px(12.0))
                .flex_shrink_0()
                .rounded(px(3.0))
                .border_1()
                .border_color(chrome.border)
                .bg(row.value),
        )
        .child(
            div()
                .min_w(px(0.0))
                .truncate()
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .text_color(chrome.body_text)
                .child(row.label),
        )
        .into_any_element();

    let value = div().child(format_compact_hsla(row.value)).into_any_element();

    InspectorRow::new(look, label, value, source, 180.0, show_separator).into_any_element()
}

fn render_property_row(row: InspectPropertyRow, look: &Arc<ShadcnLook>, show_separator: bool) -> AnyElement {
    let chrome = look.chrome();
    let body = &look.mode_tokens().typography.text.body;

    let label = div()
        .text_size(px(body.size))
        .line_height(px(body.line_height))
        .text_color(chrome.body_text)
        .child(row.label)
        .into_any_element();

    let value = div().text_color(chrome.title_text).child(row.value).into_any_element();
    let source = row.detail.unwrap_or(row.source);

    InspectorRow::new(look, label, value, source, 54.0, show_separator).into_any_element()
}
