use std::sync::Arc;

use gpui::{Context, SharedString};
use gpui::Hsla;
use gpui_luma::controls::slider::Slider;
use gpui_luma::controls::textfield::{TextField, TextFieldLook, TextFieldLookOverride, TextFieldBuilder};
use gpui_luma::controls::selector::SelectorItem;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};

use super::model::METRIC_STEP_REM;
use super::parsing::{
    format_metric_rem, format_palette_hsl_multiplier, format_palette_hue_deg, format_shadow_color_input,
    format_shadow_number,
};
use super::ThemeSidebar;
use crate::studio::overrides::ThemeShadowOverride;
use crate::theme::available_themes;

/// System monospace face for hex values. Theme CSS `font-mono` families (e.g. Fira Code) are not
/// registered with GPUI unless explicitly loaded, so token fields use a native face per platform.
fn token_field_mono_font() -> SharedString {
    #[cfg(target_os = "macos")]
    {
        "Menlo".into()
    }
    #[cfg(target_os = "windows")]
    {
        return "Consolas".into();
    }
    #[cfg(target_os = "linux")]
    {
        return "DejaVu Sans Mono".into();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "monospace".into()
    }
}

const TOKEN_FIELD_FONT_SIZE: f32 = 12.0;
const TOKEN_FIELD_LINE_HEIGHT: f32 = 16.0;

fn apply_token_field_look(mut look: TextFieldLook) -> TextFieldLook {
    look.font_family = token_field_mono_font();
    look.typography.size = TOKEN_FIELD_FONT_SIZE;
    look.typography.line_height = TOKEN_FIELD_LINE_HEIGHT;
    look.padding_y = 2.0;
    look.min_height = 22.0;
    look
}

pub(super) fn token_field_look_override_arc() -> TextFieldLookOverride {
    Arc::new(apply_token_field_look)
}

pub(super) trait TokenFieldBuilderExt {
    fn token_style(self) -> Self;
}

impl TokenFieldBuilderExt for TextFieldBuilder {
    fn token_style(self) -> Self {
        self.look_override(apply_token_field_look)
    }
}

fn build_number_field(
    look: &Arc<ShadcnLook>,
    id: &str,
    value: impl Into<SharedString>,
    cx: &mut Context<ThemeSidebar>,
) -> TextField {
    look.textfield(format!("theme-studio-{id}-field"))
        .value(value)
        .full_width(true)
        .token_style()
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

pub(super) fn build_metric_field(
    look: &Arc<ShadcnLook>,
    id: &str,
    value_rem: f32,
    cx: &mut Context<ThemeSidebar>,
) -> TextField {
    build_number_field(look, id, format_metric_rem(value_rem), cx)
}

pub(super) fn build_metric_slider(
    look: &Arc<ShadcnLook>,
    id: &str,
    min: f32,
    max: f32,
    value: f32,
    cx: &mut Context<ThemeSidebar>,
) -> Slider {
    build_slider(look, id, min, max, METRIC_STEP_REM, value, cx)
}

pub(super) fn build_palette_hue_field(
    look: &Arc<ShadcnLook>,
    value_deg: f32,
    cx: &mut Context<ThemeSidebar>,
) -> TextField {
    build_number_field(look, "palette-hue", format_palette_hue_deg(value_deg), cx)
}

pub(super) fn build_palette_hsl_field(
    look: &Arc<ShadcnLook>,
    id: &str,
    value: f32,
    cx: &mut Context<ThemeSidebar>,
) -> TextField {
    build_number_field(look, id, format_palette_hsl_multiplier(value), cx)
}

pub(super) fn build_palette_slider(
    look: &Arc<ShadcnLook>,
    id: &str,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    cx: &mut Context<ThemeSidebar>,
) -> Slider {
    build_slider(look, id, min, max, step, value, cx)
}

pub(super) fn build_shadow_color_field(
    look: &Arc<ShadcnLook>,
    shadow: &ThemeShadowOverride,
    cx: &mut Context<ThemeSidebar>,
) -> TextField {
    build_number_field(look, "shadow-color", format_shadow_color_input(shadow.color), cx)
}

pub(super) fn build_shadow_number_field(
    look: &Arc<ShadcnLook>,
    id: &str,
    value: f32,
    cx: &mut Context<ThemeSidebar>,
) -> TextField {
    build_number_field(look, id, format_shadow_number(value), cx)
}

pub(super) fn build_shadow_slider(
    look: &Arc<ShadcnLook>,
    id: &str,
    min: f32,
    max: f32,
    value: f32,
    cx: &mut Context<ThemeSidebar>,
) -> Slider {
    build_slider(look, id, min, max, METRIC_STEP_REM, value, cx)
}

pub(super) fn theme_selector_items() -> Vec<SelectorItem> {
    let mut items = vec![SelectorItem::new("default").label("Default")];
    for theme in available_themes() {
        items.push(SelectorItem::new(theme.id).label(theme.display_name()));
    }
    items
}

#[allow(dead_code)]
fn _keep_hsla_used(_: Hsla) {}
