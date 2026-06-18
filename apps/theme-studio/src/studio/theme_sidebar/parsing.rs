use std::collections::HashMap;

use gpui::Hsla;
use gpui_luma_look_shadcn::ShadcnLook;

use super::model::{DEFAULT_RADIUS_REM, DEFAULT_SPACING_REM, REM_IN_PX};
use crate::studio::export::{catalog_color_for_token, token_css_name};
use crate::studio::overrides::StudioOverrides;
use crate::studio::panels::{format_hex_color, parse_hex_color};

pub(super) fn parse_metric_rem(value: &str, min: f32, max: f32) -> Option<f32> {
    value
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(min, max))
}

pub(super) fn format_metric_rem(value: f32) -> String {
    let rounded = (value * 1000.0).round() / 1000.0;
    let mut text = format!("{rounded:.3}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

pub(super) fn effective_radius_rem(look: &ShadcnLook, overrides: &StudioOverrides) -> f32 {
    overrides.radius_rem().unwrap_or_else(|| {
        look.parse_pixel_token("radius").map(|value| value / REM_IN_PX).unwrap_or(DEFAULT_RADIUS_REM)
    })
}

pub(super) fn effective_spacing_rem(look: &ShadcnLook, overrides: &StudioOverrides) -> f32 {
    overrides.spacing_rem().unwrap_or_else(|| {
        look.parse_pixel_token("spacing").map(|value| value / REM_IN_PX).unwrap_or(DEFAULT_SPACING_REM)
    })
}

pub(super) fn format_shadow_color_input(color: Hsla) -> String {
    format!("hsl({} {}% {}%)", (color.h * 360.0).round(), (color.s * 100.0).round(), (color.l * 100.0).round(),)
}

pub(super) fn format_shadow_number(value: f32) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    let mut text = format!("{rounded:.2}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

pub(super) fn parse_shadow_color_input(raw: &str) -> Option<Hsla> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(color) = parse_hex_color(trimmed) {
        return Some(color);
    }

    let inner = if trimmed.len() >= 5 && trimmed[..4].eq_ignore_ascii_case("hsl(") && trimmed.ends_with(')') {
        &trimmed[4..trimmed.len() - 1]
    } else if trimmed.len() >= 6 && trimmed[..5].eq_ignore_ascii_case("hsla(") && trimmed.ends_with(')') {
        &trimmed[5..trimmed.len() - 1]
    } else {
        return None;
    };

    let normalized = inner.replace(',', " ");
    let (channels, alpha) = match normalized.split_once('/') {
        Some((channels, alpha)) => (channels.trim(), Some(alpha.trim())),
        None => (normalized.trim(), None),
    };
    let mut parts = channels.split_whitespace();
    let hue = parts.next()?.parse::<f32>().ok()? / 360.0;
    let saturation = parts.next()?.trim_end_matches('%').parse::<f32>().ok()? / 100.0;
    let lightness = parts.next()?.trim_end_matches('%').parse::<f32>().ok()? / 100.0;
    let alpha = alpha.and_then(|value| value.parse::<f32>().ok()).unwrap_or(1.0);

    Some(Hsla {
        h: hue.rem_euclid(1.0),
        s: saturation.clamp(0.0, 1.0),
        l: lightness.clamp(0.0, 1.0),
        a: alpha.clamp(0.0, 1.0),
    })
}

pub(super) fn token_hex_value(look: &ShadcnLook, global_overrides: &HashMap<String, Hsla>, token: &str) -> String {
    format_hex_color(effective_token_color(look, global_overrides, token))
}

pub(super) fn effective_token_color(look: &ShadcnLook, global_overrides: &HashMap<String, Hsla>, token: &str) -> Hsla {
    token_color_with_fallback(look, global_overrides, token, gpui::hsla(0.0, 0.0, 0.5, 1.0))
}

/// Resolves a theme token when present; otherwise uses `fallback` (e.g. chrome defaults).
pub(super) fn token_color_with_fallback(
    look: &ShadcnLook,
    global_overrides: &HashMap<String, Hsla>,
    token: &str,
    fallback: Hsla,
) -> Hsla {
    let css_name = token_css_name(token);
    global_overrides
        .get(&css_name)
        .copied()
        .or_else(|| catalog_color_for_token(look, token))
        .unwrap_or(fallback)
}
