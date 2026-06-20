use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, Hsla, IntoElement, SharedString, Subscription, TextRun, Window, div, font, prelude::*,
    px,
};
use gpui_luma::controls::slider::Slider;
use gpui_luma::controls::slider::SliderEvent;
use gpui_luma::controls::textfield::{TextField, TextFieldEvent};
use gpui_luma::{GridTrack, grid_layout, hstack, vstack};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnLookControlExt, ShadcnTextSize};

use super::colors::{apply_token_field_style, token_field_look_override_arc};
use super::super::ThemeSidebar;
use super::super::model::{METRIC_FIELD_WIDTH, SHADOW_COLOR_FIELD_WIDTH, SHADOW_COLOR_SWATCH_SIZE, SHADOW_SECTION_GAP};
use super::super::parsing::{
    effective_radius_rem, effective_spacing_rem, format_metric_rem, format_palette_hsl_multiplier,
    format_palette_hue_deg, format_shadow_color_input, format_shadow_number, parse_metric_rem,
    parse_palette_hsl_number, parse_shadow_color_input,
};
use crate::studio::app::ThemeStudioApp;
use crate::studio::hs_mixer::{ThemePaletteHsOverride, clamp_palette_temperature_amount, clamp_palette_vividness_amount};
use crate::studio::overrides::{
    PALETTE_HUE_DEG_MAX, PALETTE_HUE_DEG_MIN, PALETTE_LIGHTNESS_MULTIPLIER_MAX, PALETTE_LIGHTNESS_MULTIPLIER_MIN,
    PALETTE_SATURATION_MULTIPLIER_MAX, PALETTE_SATURATION_MULTIPLIER_MIN, RADIUS_REM_MAX, RADIUS_REM_MIN,
    SHADOW_BLUR_MAX, SHADOW_BLUR_MIN, SHADOW_OFFSET_X_MAX, SHADOW_OFFSET_X_MIN, SHADOW_OFFSET_Y_MAX,
    SHADOW_OFFSET_Y_MIN, SHADOW_OPACITY_MAX, SHADOW_OPACITY_MIN, SHADOW_SPREAD_MAX, SHADOW_SPREAD_MIN, SPACING_REM_MAX,
    SPACING_REM_MIN, StudioOverrides, ThemePaletteHslOverride, ThemeShadowOverride, clamp_palette_hue_deg,
    clamp_palette_lightness_multiplier, clamp_palette_saturation_multiplier, clamp_radius_rem, clamp_shadow_blur,
    clamp_shadow_offset_x, clamp_shadow_offset_y, clamp_shadow_opacity, clamp_shadow_spread, clamp_spacing_rem,
    resolved_shadow_override,
};

const HSL_GRID_LABELS: [&str; 3] = ["Hue", "Saturation", "Lightness"];
const HS_ADJUSTMENT_LEFT_LABELS: [&str; 2] = ["Neutral", "Warmer"];
const HS_ADJUSTMENT_RIGHT_LABELS: [&str; 2] = ["Vivid", "Cooler"];
const SHADOW_GRID_LABELS: [&str; 5] = ["Opacity", "Blur", "Spread", "Offset X", "Offset Y"];
const SHADOW_GRID_UNIT_WIDTH: f32 = 24.0;
const SLIDER_FIELD_GRID_GAP_X: f32 = 10.0;

pub(in crate::studio::theme_sidebar) struct OtherPanelControls {
    pub palette_hsl: ThemePaletteHslOverride,
    pub palette_hs: ThemePaletteHsOverride,
    pub palette_hue_field: TextField,
    pub palette_saturation_field: TextField,
    pub palette_lightness_field: TextField,
    pub palette_hue_slider: Slider,
    pub palette_saturation_slider: Slider,
    pub palette_lightness_slider: Slider,
    pub palette_vividness_slider: Slider,
    pub palette_temperature_slider: Slider,
    pub radius_field: TextField,
    pub spacing_field: TextField,
    pub radius_slider: Slider,
    pub spacing_slider: Slider,
    pub shadow_override: ThemeShadowOverride,
    pub shadow_color_field: TextField,
    pub shadow_opacity_field: TextField,
    pub shadow_blur_field: TextField,
    pub shadow_spread_field: TextField,
    pub shadow_offset_x_field: TextField,
    pub shadow_offset_y_field: TextField,
    pub shadow_opacity_slider: Slider,
    pub shadow_blur_slider: Slider,
    pub shadow_spread_slider: Slider,
    pub shadow_offset_x_slider: Slider,
    pub shadow_offset_y_slider: Slider,
}

fn build_number_field(
    look: &Arc<ShadcnLook>,
    id: &str,
    value: impl Into<SharedString>,
    cx: &mut Context<ThemeSidebar>,
) -> TextField {
    apply_token_field_style(look.textfield(format!("theme-studio-{id}-field")))
        .value(value)
        .full_width(true)
        .spawn(cx)
}

fn build_slider(
    look: &Arc<ShadcnLook>,
    id: &str,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    cx: &mut Context<ThemeSidebar>,
) -> Slider {
    look.slider(format!("theme-studio-{id}-slider")).range(min..max).step(step).value(value).spawn(cx)
}

pub(in crate::studio::theme_sidebar) fn build_other_panel_controls(
    look: &Arc<ShadcnLook>,
    overrides: &StudioOverrides,
    cx: &mut Context<ThemeSidebar>,
) -> OtherPanelControls {
    let palette_hsl = overrides.palette_hsl(look.mode()).clone();
    let palette_hs = overrides.palette_hs(look.mode()).clone();
    let radius_rem = effective_radius_rem(look, overrides);
    let spacing_rem = effective_spacing_rem(look, overrides);
    let shadow = resolved_shadow_override(look, overrides);

    OtherPanelControls {
        palette_hsl: palette_hsl.clone(),
        palette_hs: palette_hs.clone(),
        palette_hue_field: build_number_field(look, "palette-hue", format_palette_hue_deg(palette_hsl.hue_deg), cx),
        palette_saturation_field: build_number_field(
            look,
            "palette-saturation",
            format_palette_hsl_multiplier(palette_hsl.saturation_multiplier),
            cx,
        ),
        palette_lightness_field: build_number_field(
            look,
            "palette-lightness",
            format_palette_hsl_multiplier(palette_hsl.lightness_multiplier),
            cx,
        ),
        palette_hue_slider: build_slider(
            look,
            "palette-hue",
            PALETTE_HUE_DEG_MIN,
            PALETTE_HUE_DEG_MAX,
            1.0,
            palette_hsl.hue_deg,
            cx,
        ),
        palette_saturation_slider: build_slider(
            look,
            "palette-saturation",
            PALETTE_SATURATION_MULTIPLIER_MIN,
            PALETTE_SATURATION_MULTIPLIER_MAX,
            0.01,
            palette_hsl.saturation_multiplier,
            cx,
        ),
        palette_lightness_slider: build_slider(
            look,
            "palette-lightness",
            PALETTE_LIGHTNESS_MULTIPLIER_MIN,
            PALETTE_LIGHTNESS_MULTIPLIER_MAX,
            0.01,
            palette_hsl.lightness_multiplier,
            cx,
        ),
        palette_vividness_slider: build_slider(
            look,
            "palette-vividness",
            crate::studio::hs_mixer::PALETTE_VIVIDNESS_AMOUNT_MIN,
            crate::studio::hs_mixer::PALETTE_VIVIDNESS_AMOUNT_MAX,
            0.01,
            palette_hs.vividness_amount,
            cx,
        ),
        palette_temperature_slider: build_slider(
            look,
            "palette-temperature",
            crate::studio::hs_mixer::PALETTE_TEMPERATURE_AMOUNT_MIN,
            crate::studio::hs_mixer::PALETTE_TEMPERATURE_AMOUNT_MAX,
            0.01,
            palette_hs.temperature_amount,
            cx,
        ),
        radius_field: build_number_field(look, "radius", format_metric_rem(radius_rem), cx),
        spacing_field: build_number_field(look, "spacing", format_metric_rem(spacing_rem), cx),
        radius_slider: build_slider(look, "radius", RADIUS_REM_MIN, RADIUS_REM_MAX, 0.01, radius_rem, cx),
        spacing_slider: build_slider(look, "spacing", SPACING_REM_MIN, SPACING_REM_MAX, 0.01, spacing_rem, cx),
        shadow_override: shadow.clone(),
        shadow_color_field: build_number_field(look, "shadow-color", format_shadow_color_input(shadow.color), cx),
        shadow_opacity_field: build_number_field(look, "shadow-opacity", format_shadow_number(shadow.opacity()), cx),
        shadow_blur_field: build_number_field(look, "shadow-blur", format_shadow_number(shadow.blur_px), cx),
        shadow_spread_field: build_number_field(look, "shadow-spread", format_shadow_number(shadow.spread_px), cx),
        shadow_offset_x_field: build_number_field(
            look,
            "shadow-offset-x",
            format_shadow_number(shadow.offset_x_px),
            cx,
        ),
        shadow_offset_y_field: build_number_field(
            look,
            "shadow-offset-y",
            format_shadow_number(shadow.offset_y_px),
            cx,
        ),
        shadow_opacity_slider: build_slider(
            look,
            "shadow-opacity",
            SHADOW_OPACITY_MIN,
            SHADOW_OPACITY_MAX,
            0.01,
            shadow.opacity(),
            cx,
        ),
        shadow_blur_slider: build_slider(
            look,
            "shadow-blur",
            SHADOW_BLUR_MIN,
            SHADOW_BLUR_MAX,
            0.01,
            shadow.blur_px,
            cx,
        ),
        shadow_spread_slider: build_slider(
            look,
            "shadow-spread",
            SHADOW_SPREAD_MIN,
            SHADOW_SPREAD_MAX,
            0.01,
            shadow.spread_px,
            cx,
        ),
        shadow_offset_x_slider: build_slider(
            look,
            "shadow-offset-x",
            SHADOW_OFFSET_X_MIN,
            SHADOW_OFFSET_X_MAX,
            0.01,
            shadow.offset_x_px,
            cx,
        ),
        shadow_offset_y_slider: build_slider(
            look,
            "shadow-offset-y",
            SHADOW_OFFSET_Y_MIN,
            SHADOW_OFFSET_Y_MAX,
            0.01,
            shadow.offset_y_px,
            cx,
        ),
    }
}

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

impl ThemeSidebar {
    pub(in crate::studio::theme_sidebar) fn sync_other_panel_templates(
        &self,
        theme: &Arc<ShadcnLook>,
        cx: &mut Context<Self>,
    ) {
        for field in [&self.vm.palette_hue_field, &self.vm.palette_saturation_field, &self.vm.palette_lightness_field] {
            field.update(cx, |field, cx| {
                field.set_template(theme.textfield_template(), cx);
                field.set_look_override(Some(token_field_look_override_arc()), cx);
            });
        }

        for field in [&self.vm.radius_field, &self.vm.spacing_field] {
            field.update(cx, |field, cx| {
                field.set_template(theme.textfield_template(), cx);
                field.set_look_override(Some(token_field_look_override_arc()), cx);
            });
        }

        for field in [
            &self.vm.shadow_color_field,
            &self.vm.shadow_opacity_field,
            &self.vm.shadow_blur_field,
            &self.vm.shadow_spread_field,
            &self.vm.shadow_offset_x_field,
            &self.vm.shadow_offset_y_field,
        ] {
            field.update(cx, |field, cx| {
                field.set_template(theme.textfield_template(), cx);
                field.set_look_override(Some(token_field_look_override_arc()), cx);
            });
        }

        for slider in [
            &self.vm.palette_hue_slider,
            &self.vm.palette_saturation_slider,
            &self.vm.palette_lightness_slider,
            &self.vm.palette_vividness_slider,
            &self.vm.palette_temperature_slider,
            &self.vm.radius_slider,
            &self.vm.spacing_slider,
            &self.vm.shadow_opacity_slider,
            &self.vm.shadow_blur_slider,
            &self.vm.shadow_spread_slider,
            &self.vm.shadow_offset_x_slider,
            &self.vm.shadow_offset_y_slider,
        ] {
            slider.update(cx, |slider, cx| {
                slider.set_template(theme.slider_template(), cx);
            });
        }
    }

    pub(in crate::studio::theme_sidebar) fn sync_other_panel_values(
        &mut self,
        look: &ShadcnLook,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) {
        let palette_hsl = overrides.palette_hsl(self.vm.look.mode()).clone();
        self.vm.palette_hsl = palette_hsl.clone();
        self.vm
            .palette_hue_field
            .update(cx, |field, cx| field.set_value(format_palette_hue_deg(palette_hsl.hue_deg), cx));
        self.vm.palette_saturation_field.update(cx, |field, cx| {
            field.set_value(format_palette_hsl_multiplier(palette_hsl.saturation_multiplier), cx)
        });
        self.vm.palette_lightness_field.update(cx, |field, cx| {
            field.set_value(format_palette_hsl_multiplier(palette_hsl.lightness_multiplier), cx)
        });
        self.vm.palette_hue_slider.update(cx, |slider, cx| slider.set_value(palette_hsl.hue_deg, cx));
        self.vm
            .palette_saturation_slider
            .update(cx, |slider, cx| slider.set_value(palette_hsl.saturation_multiplier, cx));
        self.vm
            .palette_lightness_slider
            .update(cx, |slider, cx| slider.set_value(palette_hsl.lightness_multiplier, cx));

        let palette_hs = overrides.palette_hs(self.vm.look.mode()).clone();
        self.vm.palette_hs = palette_hs.clone();
        self.vm
            .palette_vividness_slider
            .update(cx, |slider, cx| slider.set_value(palette_hs.vividness_amount, cx));
        self.vm
            .palette_temperature_slider
            .update(cx, |slider, cx| slider.set_value(palette_hs.temperature_amount, cx));

        let radius_rem = effective_radius_rem(look, overrides);
        let spacing_rem = effective_spacing_rem(look, overrides);
        self.vm.radius_field.update(cx, |field, cx| field.set_value(format_metric_rem(radius_rem), cx));
        self.vm.spacing_field.update(cx, |field, cx| field.set_value(format_metric_rem(spacing_rem), cx));
        self.vm.radius_slider.update(cx, |slider, cx| slider.set_value(radius_rem, cx));
        self.vm.spacing_slider.update(cx, |slider, cx| slider.set_value(spacing_rem, cx));

        let shadow = resolved_shadow_override(look, overrides);
        self.vm.shadow_override = shadow.clone();
        self.vm
            .shadow_color_field
            .update(cx, |field, cx| field.set_value(format_shadow_color_input(shadow.color), cx));
        self.vm
            .shadow_opacity_field
            .update(cx, |field, cx| field.set_value(format_metric_rem(shadow.opacity()), cx));
        self.vm
            .shadow_blur_field
            .update(cx, |field, cx| field.set_value(format_shadow_number(shadow.blur_px), cx));
        self.vm
            .shadow_spread_field
            .update(cx, |field, cx| field.set_value(format_shadow_number(shadow.spread_px), cx));
        self.vm
            .shadow_offset_x_field
            .update(cx, |field, cx| field.set_value(format_shadow_number(shadow.offset_x_px), cx));
        self.vm
            .shadow_offset_y_field
            .update(cx, |field, cx| field.set_value(format_shadow_number(shadow.offset_y_px), cx));
        self.vm.shadow_opacity_slider.update(cx, |slider, cx| slider.set_value(shadow.opacity(), cx));
        self.vm.shadow_blur_slider.update(cx, |slider, cx| slider.set_value(shadow.blur_px, cx));
        self.vm.shadow_spread_slider.update(cx, |slider, cx| slider.set_value(shadow.spread_px, cx));
        self.vm.shadow_offset_x_slider.update(cx, |slider, cx| slider.set_value(shadow.offset_x_px, cx));
        self.vm.shadow_offset_y_slider.update(cx, |slider, cx| slider.set_value(shadow.offset_y_px, cx));
    }
}

pub(in crate::studio::theme_sidebar) fn wire_other_subscriptions(
    sidebar: &Entity<ThemeSidebar>,
    cx: &mut Context<ThemeStudioApp>,
    subscriptions: &mut Vec<Subscription>,
) {
    let palette_hue_field = sidebar.read(cx).vm.palette_hue_field.clone();
    subscriptions.push(cx.subscribe(&palette_hue_field, |app, _, event: &TextFieldEvent, cx| {
        if let TextFieldEvent::Change { value } = event {
            let Some(hue_deg) =
                parse_palette_hsl_number(value, PALETTE_HUE_DEG_MIN, PALETTE_HUE_DEG_MAX).map(clamp_palette_hue_deg)
            else {
                return;
            };
            app.set_palette_hue_deg(hue_deg, cx);
        }
    }));

    let palette_saturation_field = sidebar.read(cx).vm.palette_saturation_field.clone();
    subscriptions.push(cx.subscribe(&palette_saturation_field, |app, _, event: &TextFieldEvent, cx| {
        if let TextFieldEvent::Change { value } = event {
            let Some(multiplier) =
                parse_palette_hsl_number(value, PALETTE_SATURATION_MULTIPLIER_MIN, PALETTE_SATURATION_MULTIPLIER_MAX)
                    .map(clamp_palette_saturation_multiplier)
            else {
                return;
            };
            app.set_palette_saturation_multiplier(multiplier, cx);
        }
    }));

    let palette_lightness_field = sidebar.read(cx).vm.palette_lightness_field.clone();
    subscriptions.push(cx.subscribe(&palette_lightness_field, |app, _, event: &TextFieldEvent, cx| {
        if let TextFieldEvent::Change { value } = event {
            let Some(multiplier) =
                parse_palette_hsl_number(value, PALETTE_LIGHTNESS_MULTIPLIER_MIN, PALETTE_LIGHTNESS_MULTIPLIER_MAX)
                    .map(clamp_palette_lightness_multiplier)
            else {
                return;
            };
            app.set_palette_lightness_multiplier(multiplier, cx);
        }
    }));

    let palette_hue_slider = sidebar.read(cx).vm.palette_hue_slider.clone();
    subscriptions.push(cx.subscribe(&palette_hue_slider, |app, _, event: &SliderEvent, cx| {
        let SliderEvent::Change { value } = event;
        app.set_palette_hue_deg(*value, cx);
    }));

    let palette_saturation_slider = sidebar.read(cx).vm.palette_saturation_slider.clone();
    subscriptions.push(cx.subscribe(&palette_saturation_slider, |app, _, event: &SliderEvent, cx| {
        let SliderEvent::Change { value } = event;
        app.set_palette_saturation_multiplier(*value, cx);
    }));

    let palette_lightness_slider = sidebar.read(cx).vm.palette_lightness_slider.clone();
    subscriptions.push(cx.subscribe(&palette_lightness_slider, |app, _, event: &SliderEvent, cx| {
        let SliderEvent::Change { value } = event;
        app.set_palette_lightness_multiplier(*value, cx);
    }));

    let palette_vividness_slider = sidebar.read(cx).vm.palette_vividness_slider.clone();
    subscriptions.push(cx.subscribe(&palette_vividness_slider, |app, _, event: &SliderEvent, cx| {
        let SliderEvent::Change { value } = event;
        app.set_palette_vividness_amount(clamp_palette_vividness_amount(*value), cx);
    }));

    let palette_temperature_slider = sidebar.read(cx).vm.palette_temperature_slider.clone();
    subscriptions.push(cx.subscribe(&palette_temperature_slider, |app, _, event: &SliderEvent, cx| {
        let SliderEvent::Change { value } = event;
        app.set_palette_temperature_amount(clamp_palette_temperature_amount(*value), cx);
    }));

    let radius_field = sidebar.read(cx).vm.radius_field.clone();
    subscriptions.push(cx.subscribe(&radius_field, |app, _, event: &TextFieldEvent, cx| {
        if let TextFieldEvent::Change { value } = event {
            let Some(rem) = parse_metric_rem(value, RADIUS_REM_MIN, RADIUS_REM_MAX).map(clamp_radius_rem) else {
                return;
            };
            app.set_radius_rem(rem, cx);
        }
    }));

    let spacing_field = sidebar.read(cx).vm.spacing_field.clone();
    subscriptions.push(cx.subscribe(&spacing_field, |app, _, event: &TextFieldEvent, cx| {
        if let TextFieldEvent::Change { value } = event {
            let Some(rem) = parse_metric_rem(value, SPACING_REM_MIN, SPACING_REM_MAX).map(clamp_spacing_rem) else {
                return;
            };
            app.set_spacing_rem(rem, cx);
        }
    }));

    let radius_slider = sidebar.read(cx).vm.radius_slider.clone();
    subscriptions.push(cx.subscribe(&radius_slider, |app, _, event: &SliderEvent, cx| {
        let SliderEvent::Change { value } = event;
        app.set_radius_rem(*value, cx);
    }));

    let spacing_slider = sidebar.read(cx).vm.spacing_slider.clone();
    subscriptions.push(cx.subscribe(&spacing_slider, |app, _, event: &SliderEvent, cx| {
        let SliderEvent::Change { value } = event;
        app.set_spacing_rem(*value, cx);
    }));

    let shadow_color_field = sidebar.read(cx).vm.shadow_color_field.clone();
    subscriptions.push(cx.subscribe(&shadow_color_field, |app, _, event: &TextFieldEvent, cx| {
        if let TextFieldEvent::Change { value } = event {
            let Some(color) = parse_shadow_color_input(value) else {
                return;
            };
            app.set_shadow_color(color, cx);
        }
    }));

    let shadow_opacity_field = sidebar.read(cx).vm.shadow_opacity_field.clone();
    subscriptions.push(cx.subscribe(&shadow_opacity_field, |app, _, event: &TextFieldEvent, cx| {
        if let TextFieldEvent::Change { value } = event {
            let Some(opacity) =
                parse_metric_rem(value, SHADOW_OPACITY_MIN, SHADOW_OPACITY_MAX).map(clamp_shadow_opacity)
            else {
                return;
            };
            app.set_shadow_opacity(opacity, cx);
        }
    }));

    let shadow_blur_field = sidebar.read(cx).vm.shadow_blur_field.clone();
    subscriptions.push(cx.subscribe(&shadow_blur_field, |app, _, event: &TextFieldEvent, cx| {
        if let TextFieldEvent::Change { value } = event {
            let Some(blur) = parse_metric_rem(value, SHADOW_BLUR_MIN, SHADOW_BLUR_MAX).map(clamp_shadow_blur) else {
                return;
            };
            app.set_shadow_blur(blur, cx);
        }
    }));

    let shadow_spread_field = sidebar.read(cx).vm.shadow_spread_field.clone();
    subscriptions.push(cx.subscribe(&shadow_spread_field, |app, _, event: &TextFieldEvent, cx| {
        if let TextFieldEvent::Change { value } = event {
            let Some(spread) = parse_metric_rem(value, SHADOW_SPREAD_MIN, SHADOW_SPREAD_MAX).map(clamp_shadow_spread)
            else {
                return;
            };
            app.set_shadow_spread(spread, cx);
        }
    }));

    let shadow_offset_x_field = sidebar.read(cx).vm.shadow_offset_x_field.clone();
    subscriptions.push(cx.subscribe(&shadow_offset_x_field, |app, _, event: &TextFieldEvent, cx| {
        if let TextFieldEvent::Change { value } = event {
            let Some(offset_x) =
                parse_metric_rem(value, SHADOW_OFFSET_X_MIN, SHADOW_OFFSET_X_MAX).map(clamp_shadow_offset_x)
            else {
                return;
            };
            app.set_shadow_offset_x(offset_x, cx);
        }
    }));

    let shadow_offset_y_field = sidebar.read(cx).vm.shadow_offset_y_field.clone();
    subscriptions.push(cx.subscribe(&shadow_offset_y_field, |app, _, event: &TextFieldEvent, cx| {
        if let TextFieldEvent::Change { value } = event {
            let Some(offset_y) =
                parse_metric_rem(value, SHADOW_OFFSET_Y_MIN, SHADOW_OFFSET_Y_MAX).map(clamp_shadow_offset_y)
            else {
                return;
            };
            app.set_shadow_offset_y(offset_y, cx);
        }
    }));

    let shadow_opacity_slider = sidebar.read(cx).vm.shadow_opacity_slider.clone();
    subscriptions.push(cx.subscribe(&shadow_opacity_slider, |app, _, event: &SliderEvent, cx| {
        let SliderEvent::Change { value } = event;
        app.set_shadow_opacity(*value, cx);
    }));

    let shadow_blur_slider = sidebar.read(cx).vm.shadow_blur_slider.clone();
    subscriptions.push(cx.subscribe(&shadow_blur_slider, |app, _, event: &SliderEvent, cx| {
        let SliderEvent::Change { value } = event;
        app.set_shadow_blur(*value, cx);
    }));

    let shadow_spread_slider = sidebar.read(cx).vm.shadow_spread_slider.clone();
    subscriptions.push(cx.subscribe(&shadow_spread_slider, |app, _, event: &SliderEvent, cx| {
        let SliderEvent::Change { value } = event;
        app.set_shadow_spread(*value, cx);
    }));

    let shadow_offset_x_slider = sidebar.read(cx).vm.shadow_offset_x_slider.clone();
    subscriptions.push(cx.subscribe(&shadow_offset_x_slider, |app, _, event: &SliderEvent, cx| {
        let SliderEvent::Change { value } = event;
        app.set_shadow_offset_x(*value, cx);
    }));

    let shadow_offset_y_slider = sidebar.read(cx).vm.shadow_offset_y_slider.clone();
    subscriptions.push(cx.subscribe(&shadow_offset_y_slider, |app, _, event: &SliderEvent, cx| {
        let SliderEvent::Change { value } = event;
        app.set_shadow_offset_y(*value, cx);
    }));
}
