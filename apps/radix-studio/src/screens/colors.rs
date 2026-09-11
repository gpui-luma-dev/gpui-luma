//! Colors tab screen — Radix Colors catalog matrix.

use gpui::{Hsla, div, hsla, linear_color_stop, linear_gradient, prelude::*, px};
use luma::hstack;
use luma::vstack;
use luma::theme::ThemeMode;
use luma_color::ColorSwatch;
use luma_look_radix::{BLACK_ALPHA_STEPS, RadixLook, WHITE_ALPHA_STEPS, color_families, parse_color};

const COLOR_MATRIX_LABEL_W: f32 = 112.0;
const COLOR_MATRIX_CELL_W: f32 = 95.0;
const COLOR_MATRIX_CELL_H: f32 = 50.0;

pub fn page(look: &RadixLook, fg: Hsla, muted: Hsla) -> gpui::AnyElement {
    let mut matrix = div().flex().flex_col().gap(px(3.0)).flex_none();
    let mut groups = hstack! { gap=3; div().w(px(COLOR_MATRIX_LABEL_W)).flex_none() };
    for (label, span) in [("Backgrounds", 2), ("Interactive", 3), ("Borders", 3), ("Solid", 2), ("Text", 2)] {
        groups = groups.child(
            div()
                .w(px(COLOR_MATRIX_CELL_W * span as f32 + 3.0 * (span as f32 - 1.0)))
                .flex_none()
                .text_xs()
                .text_center()
                .text_color(muted)
                .border_b_1()
                .border_color(muted.opacity(0.45))
                .child(label),
        );
    }
    matrix = matrix.child(groups);

    let mut steps = hstack! { gap=3; div().w(px(COLOR_MATRIX_LABEL_W)).flex_none() };
    for step in 1..=12 {
        steps = steps.child(
            div()
                .w(px(COLOR_MATRIX_CELL_W))
                .flex_none()
                .text_xs()
                .text_center()
                .text_color(muted)
                .child(format!("{step}")),
        );
    }
    matrix = matrix.child(steps);

    for family in color_families(look.mode()) {
        matrix = matrix.child(color_catalog_row(family, muted));
    }

    let mut alpha_matrix = div().flex().flex_col().gap(px(3.0)).flex_none();
    let mut alpha_header = hstack! { gap=3; div().w(px(COLOR_MATRIX_LABEL_W)).flex_none() };
    for step in 1..=12 {
        alpha_header = alpha_header.child(
            div()
                .w(px(COLOR_MATRIX_CELL_W))
                .flex_none()
                .text_xs()
                .text_center()
                .text_color(muted)
                .child(format!("{step}")),
        );
    }
    alpha_matrix = alpha_matrix.child(alpha_header);
    alpha_matrix = alpha_matrix.child(alpha_catalog_row("Black", &BLACK_ALPHA_STEPS, muted));
    alpha_matrix = alpha_matrix.child(alpha_catalog_row("White", &WHITE_ALPHA_STEPS, muted));

    vstack! {
        gap=16;
        div().text_2xl().font_weight(gpui::FontWeight::BOLD).text_color(fg).child("Colors"),
        div().text_sm().text_color(muted).child("Radix Colors v3.0.0 · sRGB catalog"),
        div().child(matrix),
        div().text_lg().font_weight(gpui::FontWeight::SEMIBOLD).text_color(fg).child("Shadows, highlights, and overlays"),
        div().child(alpha_matrix),
    }
    .w_full()
    .into_any_element()
}

pub fn page_background(mode: ThemeMode) -> gpui::Background {
    match mode {
        ThemeMode::Light => page_gradient(parse_color("#ffeaf1"), parse_color("#ffffff")),
        ThemeMode::Dark => page_gradient(hsla(0.92156863, 0.5, 0.13333334, 1.0), hsla(0.0, 0.0, 0.043137256, 1.0)),
    }
}

fn page_gradient(start: Hsla, end: Hsla) -> gpui::Background {
    linear_gradient(180.0, linear_color_stop(start, 0.0), linear_color_stop(end, 1.0 / 3.0))
}

fn color_catalog_row(family: &luma_look_radix::RadixColorScale, muted: Hsla) -> gpui::Div {
    let mut row = hstack! {
        gap=3;
        div().w(px(COLOR_MATRIX_LABEL_W)).flex_none().text_sm().text_color(muted).child(family.family),
    };
    for value in family.steps {
        row = row.child(color_catalog_cell(value));
    }
    row
}

fn color_catalog_cell(value: &str) -> gpui::Div {
    div().w(px(COLOR_MATRIX_CELL_W)).h(px(COLOR_MATRIX_CELL_H)).flex_none().bg(parse_color(value))
}

fn alpha_catalog_row(label: &'static str, values: &[&'static str; 12], muted: Hsla) -> gpui::Div {
    let mut row = hstack! {
        gap=3;
        div().w(px(COLOR_MATRIX_LABEL_W)).flex_none().text_sm().text_color(muted).child(label),
    };
    for value in values.iter() {
        row = row.child(
            div()
                .w(px(COLOR_MATRIX_CELL_W))
                .h(px(COLOR_MATRIX_CELL_H))
                .flex_none()
                .flex()
                .justify_center()
                .child(
                    ColorSwatch::new(parse_color(value))
                        .size(luma::theme::ControlSize::Lg)
                        .height(px(COLOR_MATRIX_CELL_H))
                        .rounded(px(0.0))
                        .bordered(false)
                        .checkerboard(true),
                ),
        );
    }
    row
}
