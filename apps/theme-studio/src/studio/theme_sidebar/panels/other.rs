use gpui::{AnyElement, Hsla, IntoElement, div, prelude::*, px};
use gpui_luma::controls::slider::Slider;
use gpui_luma::controls::textfield::TextField;
use gpui_luma::{hstack, vstack};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnTextSize};

use super::super::ThemeSidebar;
use super::super::model::{
    METRIC_FIELD_WIDTH, SHADOW_COLOR_FIELD_WIDTH, SHADOW_COLOR_SWATCH_SIZE, SHADOW_ROW_PADDING_BOTTOM,
    SHADOW_ROW_PADDING_TOP, SHADOW_SECTION_GAP,
};

pub(in crate::studio::theme_sidebar) fn render_other_panel(sidebar: &ThemeSidebar) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(10.0))
        .child(div().w_full().child(sidebar.other_accordion.clone()))
        .into_any_element()
}

pub(in crate::studio::theme_sidebar) fn other_category_content(sidebar: &ThemeSidebar, category: &str) -> AnyElement {
    match category {
        "HSL ADJUSTMENTS" => hsl_adjustments_category_content(sidebar),
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
        "SHADOW" => shadow_category_content(sidebar),
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

fn hsl_adjustments_category_content(sidebar: &ThemeSidebar) -> AnyElement {
    vstack! {
        gap=SHADOW_SECTION_GAP;
        slider_field_row_compact(
            sidebar,
            "Hue",
            sidebar.vm.palette_hue_slider.clone(),
            sidebar.vm.palette_hue_field.clone(),
            "deg",
        ),
        slider_field_row_compact(
            sidebar,
            "Saturation",
            sidebar.vm.palette_saturation_slider.clone(),
            sidebar.vm.palette_saturation_field.clone(),
            "x",
        ),
        slider_field_row_compact(
            sidebar,
            "Lightness",
            sidebar.vm.palette_lightness_slider.clone(),
            sidebar.vm.palette_lightness_field.clone(),
            "x",
        ),
    }
    .w_full()
    .into_any_element()
}

fn metric_category_content(sidebar: &ThemeSidebar, label: &str, slider: Slider, field: TextField) -> AnyElement {
    slider_field_row(sidebar, label, slider, field, "rem")
}

fn shadow_category_content(sidebar: &ThemeSidebar) -> AnyElement {
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
        slider_field_row_compact(sidebar, "Opacity", sidebar.vm.shadow_opacity_slider.clone(), sidebar.vm.shadow_opacity_field.clone(), ""),
        slider_field_row_compact(sidebar, "Blur", sidebar.vm.shadow_blur_slider.clone(), sidebar.vm.shadow_blur_field.clone(), "px"),
        slider_field_row_compact(sidebar, "Spread", sidebar.vm.shadow_spread_slider.clone(), sidebar.vm.shadow_spread_field.clone(), "px"),
        slider_field_row_compact(sidebar, "Offset X", sidebar.vm.shadow_offset_x_slider.clone(), sidebar.vm.shadow_offset_x_field.clone(), "px"),
        slider_field_row_compact(sidebar, "Offset Y", sidebar.vm.shadow_offset_y_slider.clone(), sidebar.vm.shadow_offset_y_field.clone(), "px"),
    }
    .w_full()
    .into_any_element()
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

fn slider_field_row_compact(
    sidebar: &ThemeSidebar,
    label: &str,
    slider: Slider,
    field: TextField,
    unit: &'static str,
) -> AnyElement {
    slider_field_row_with_padding(
        sidebar,
        label,
        slider,
        field,
        unit,
        SHADOW_ROW_PADDING_TOP,
        SHADOW_ROW_PADDING_BOTTOM,
    )
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
