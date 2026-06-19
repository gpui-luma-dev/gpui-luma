use gpui::{AnyElement, Hsla, IntoElement, SharedString, TextRun, Window, div, font, prelude::*, px};
use gpui_luma::controls::slider::Slider;
use gpui_luma::controls::textfield::TextField;
use gpui_luma::{GridTrack, grid_layout, hstack, vstack};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnTextSize};

use super::super::ThemeSidebar;
use super::super::model::{METRIC_FIELD_WIDTH, SHADOW_COLOR_FIELD_WIDTH, SHADOW_COLOR_SWATCH_SIZE, SHADOW_SECTION_GAP};

const HSL_GRID_LABELS: [&str; 3] = ["Hue", "Saturation", "Lightness"];
const HS_ADJUSTMENT_LEFT_LABELS: [&str; 2] = ["Neutral", "Warmer"];
const HS_ADJUSTMENT_RIGHT_LABELS: [&str; 2] = ["Vivid", "Cooler"];
const SHADOW_GRID_LABELS: [&str; 5] = ["Opacity", "Blur", "Spread", "Offset X", "Offset Y"];
const SHADOW_GRID_UNIT_WIDTH: f32 = 24.0;
const SLIDER_FIELD_GRID_GAP_X: f32 = 10.0;

pub(in crate::studio::theme_sidebar) fn render_other_panel(sidebar: &ThemeSidebar) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(10.0))
        .child(div().w_full().child(sidebar.other_accordion.clone()))
        .into_any_element()
}

pub(in crate::studio::theme_sidebar) fn other_category_content(
    sidebar: &ThemeSidebar,
    category: &str,
    window: &mut Window,
) -> AnyElement {
    match category {
        "HSL ADJUSTMENTS" => hsl_adjustments_category_content(sidebar, window),
        "HS MIXER" => hs_adjustments_category_content(sidebar, window),
        "RADIUS" => metric_category_content(
            sidebar,
            "Radius",
            sidebar.vm.radius_slider.clone(),
            sidebar.vm.radius_field.clone(),
        ),
        "SPACING" => metric_category_content(
            sidebar,
            "Spacing",
            sidebar.vm.spacing_slider.clone(),
            sidebar.vm.spacing_field.clone(),
        ),
        "SHADOW" => shadow_category_content(sidebar, window),
        _ => category_placeholder_content(sidebar, category),
    }
}

fn category_placeholder_content(sidebar: &ThemeSidebar, category: &str) -> AnyElement {
    let theme = &sidebar.vm.look;
    let chrome = theme.chrome();

    div()
        .w_full()
        .pt(px(2.0))
        .pb(px(6.0))
        .text_xs()
        .text_color(chrome.muted_text)
        .child(format!("{category} controls coming soon."))
        .into_any_element()
}

fn hsl_adjustments_category_content(sidebar: &ThemeSidebar, window: &mut Window) -> AnyElement {
    hsl_adjustments_grid(sidebar, window)
}

fn hsl_adjustments_grid(sidebar: &ThemeSidebar, window: &mut Window) -> AnyElement {
    grid_layout! {
        rows: 3,
        columns: [
            GridTrack::Px(max_label_width(sidebar, window, &HSL_GRID_LABELS)),
            GridTrack::Star(1.0),
            GridTrack::Px(METRIC_FIELD_WIDTH),
            GridTrack::Px(SHADOW_GRID_UNIT_WIDTH),
        ],
        gap_x: SLIDER_FIELD_GRID_GAP_X,
        gap_y: SHADOW_SECTION_GAP;
        [0, 0] => slider_field_grid_label(sidebar, "Hue"),
        [0, 1] => sidebar.vm.palette_hue_slider.clone(),
        [0, 2] => sidebar.vm.palette_hue_field.clone(),
        [0, 3] => slider_field_grid_unit(sidebar, "deg"),
        [1, 0] => slider_field_grid_label(sidebar, "Saturation"),
        [1, 1] => sidebar.vm.palette_saturation_slider.clone(),
        [1, 2] => sidebar.vm.palette_saturation_field.clone(),
        [1, 3] => slider_field_grid_unit(sidebar, "x"),
        [2, 0] => slider_field_grid_label(sidebar, "Lightness"),
        [2, 1] => sidebar.vm.palette_lightness_slider.clone(),
        [2, 2] => sidebar.vm.palette_lightness_field.clone(),
        [2, 3] => slider_field_grid_unit(sidebar, "x"),
    }
    .into_any_element()
}

fn hs_adjustments_category_content(sidebar: &ThemeSidebar, window: &mut Window) -> AnyElement {
    let left_label_width = max_label_width_for_size(sidebar, window, &HS_ADJUSTMENT_LEFT_LABELS, ShadcnTextSize::Sm);
    let right_label_width = max_label_width_for_size(sidebar, window, &HS_ADJUSTMENT_RIGHT_LABELS, ShadcnTextSize::Sm);

    div()
        .w_full()
        .min_w(px(0.0))
        .child(grid_layout! {
            rows: 2,
            columns: [
                GridTrack::Px(left_label_width),
                GridTrack::Star(1.0),
                GridTrack::Px(right_label_width),
            ],
            gap_x: 16.0,
            gap_y: 0.0;
            [0, 0] => hs_adjustment_label(sidebar, "Neutral", false),
            [0, 1] => div().w_full().min_w(px(0.0)).child(sidebar.vm.palette_vividness_slider.clone()),
            [0, 2] => hs_adjustment_label(sidebar, "Vivid", true),
            [1, 0] => hs_adjustment_label(sidebar, "Warmer", false),
            [1, 1] => div().w_full().min_w(px(0.0)).child(sidebar.vm.palette_temperature_slider.clone()),
            [1, 2] => hs_adjustment_label(sidebar, "Cooler", true),
        })
        .into_any_element()
}

fn metric_category_content(sidebar: &ThemeSidebar, label: &str, slider: Slider, field: TextField) -> AnyElement {
    slider_field_row(sidebar, label, slider, field, "rem")
}

fn hs_adjustment_label(sidebar: &ThemeSidebar, label: &str, right_aligned: bool) -> AnyElement {
    let theme = &sidebar.vm.look;
    let chrome = theme.chrome();
    let label_style = theme.typography_scale(ShadcnTextSize::Sm);

    div()
        .w_full()
        .when(right_aligned, |this| this.text_right())
        .typography_style(label_style)
        .text_color(chrome.body_text)
        .child(label.to_string())
        .into_any_element()
}

fn shadow_category_content(sidebar: &ThemeSidebar, window: &mut Window) -> AnyElement {
    let theme = &sidebar.vm.look;
    let chrome = theme.chrome();
    let swatch_color = Hsla { a: 1.0, ..sidebar.vm.shadow_override.color };

    vstack! {
        gap=SHADOW_SECTION_GAP;
        hstack! {
            gap=10 align=center;
            div()
                .size(px(SHADOW_COLOR_SWATCH_SIZE))
                .flex_shrink_0()
                .rounded(px(8.0))
                .bg(swatch_color)
                .border_1()
                .border_color(chrome.border),
            div()
                .w(px(SHADOW_COLOR_FIELD_WIDTH))
                .max_w_full()
                .child(sidebar.vm.shadow_color_field.clone()),
        },
        shadow_slider_grid(sidebar, window),
    }
    .w_full()
    .into_any_element()
}

fn shadow_slider_grid(sidebar: &ThemeSidebar, window: &mut Window) -> AnyElement {
    grid_layout! {
        rows: 5,
        columns: [
            GridTrack::Px(max_label_width(sidebar, window, &SHADOW_GRID_LABELS)),
            GridTrack::Star(1.0),
            GridTrack::Px(METRIC_FIELD_WIDTH),
            GridTrack::Px(SHADOW_GRID_UNIT_WIDTH),
        ],
        gap_x: SLIDER_FIELD_GRID_GAP_X,
        gap_y: SHADOW_SECTION_GAP;
        [0, 0] => slider_field_grid_label(sidebar, "Opacity"),
        [0, 1] => sidebar.vm.shadow_opacity_slider.clone(),
        [0, 2] => sidebar.vm.shadow_opacity_field.clone(),
        [0, 3] => slider_field_grid_unit(sidebar, ""),
        [1, 0] => slider_field_grid_label(sidebar, "Blur"),
        [1, 1] => sidebar.vm.shadow_blur_slider.clone(),
        [1, 2] => sidebar.vm.shadow_blur_field.clone(),
        [1, 3] => slider_field_grid_unit(sidebar, "px"),
        [2, 0] => slider_field_grid_label(sidebar, "Spread"),
        [2, 1] => sidebar.vm.shadow_spread_slider.clone(),
        [2, 2] => sidebar.vm.shadow_spread_field.clone(),
        [2, 3] => slider_field_grid_unit(sidebar, "px"),
        [3, 0] => slider_field_grid_label(sidebar, "Offset X"),
        [3, 1] => sidebar.vm.shadow_offset_x_slider.clone(),
        [3, 2] => sidebar.vm.shadow_offset_x_field.clone(),
        [3, 3] => slider_field_grid_unit(sidebar, "px"),
        [4, 0] => slider_field_grid_label(sidebar, "Offset Y"),
        [4, 1] => sidebar.vm.shadow_offset_y_slider.clone(),
        [4, 2] => sidebar.vm.shadow_offset_y_field.clone(),
        [4, 3] => slider_field_grid_unit(sidebar, "px"),
    }
    .into_any_element()
}

fn max_label_width(sidebar: &ThemeSidebar, window: &mut Window, labels: &[&str]) -> f32 {
    max_label_width_for_size(sidebar, window, labels, ShadcnTextSize::Xs)
}

fn max_label_width_for_size(
    sidebar: &ThemeSidebar,
    window: &mut Window,
    labels: &[&str],
    text_size: ShadcnTextSize,
) -> f32 {
    let theme = &sidebar.vm.look;
    let row_label_typography = theme.typography_scale(text_size);
    let mut label_font = font(theme.mode_tokens().typography.font.sans.family.clone());
    label_font.weight = row_label_typography.weight;
    let mut max_width = 0.0_f32;

    for label in labels {
        let run = TextRun {
            len: label.len(),
            font: label_font.clone(),
            color: theme.chrome().body_text,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = window.text_system().shape_line(
            SharedString::from((*label).to_owned()),
            px(row_label_typography.size),
            &[run],
            None,
        );
        max_width = max_width.max(line.x_for_index(label.len()).as_f32());
    }

    max_width.ceil()
}

fn slider_field_grid_label(sidebar: &ThemeSidebar, label: &str) -> AnyElement {
    let theme = &sidebar.vm.look;
    let chrome = theme.chrome();
    let row_label_typography = theme.typography_scale(ShadcnTextSize::Xs);

    div()
        .w_full()
        .typography_style(row_label_typography)
        .text_color(chrome.body_text)
        .child(label.to_string())
        .into_any_element()
}

fn slider_field_grid_unit(sidebar: &ThemeSidebar, unit: &'static str) -> AnyElement {
    let theme = &sidebar.vm.look;
    let chrome = theme.chrome();
    let unit_style = theme.typography_scale(ShadcnTextSize::Sm);

    div().typography_style(unit_style).text_color(chrome.muted_text).child(unit).into_any_element()
}

fn slider_field_row(
    sidebar: &ThemeSidebar,
    label: &str,
    slider: Slider,
    field: TextField,
    unit: &'static str,
) -> AnyElement {
    slider_field_row_with_padding(sidebar, label, slider, field, unit, 6.0, 8.0)
}

fn slider_field_row_with_padding(
    sidebar: &ThemeSidebar,
    label: &str,
    slider: Slider,
    field: TextField,
    unit: &'static str,
    padding_top: f32,
    padding_bottom: f32,
) -> AnyElement {
    let theme = &sidebar.vm.look;
    let chrome = theme.chrome();
    let row_label_typography = theme.typography_scale(ShadcnTextSize::Xs);
    let unit_style = theme.typography_scale(ShadcnTextSize::Sm);
    let label = label.to_string();

    hstack! {
        gap=10 align=center;
        div()
            .typography_style(row_label_typography)
            .text_color(chrome.body_text)
            .child(label),
        div()
            .flex_1()
            .min_w(px(0.0))
            .child(slider),
        div()
            .w(px(METRIC_FIELD_WIDTH))
            .child(field),
        div()
            .typography_style(unit_style)
            .text_color(chrome.muted_text)
            .child(unit),
    }
    .w_full()
    .min_w(px(0.0))
    .pt(px(padding_top))
    .pb(px(padding_bottom))
    .into_any_element()
}
