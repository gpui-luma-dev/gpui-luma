//! Static Radix Colors catalog generated from the pinned local v3.0.0 source.

#[path = "colors_data.rs"]
mod colors_data;

use gpui::{Hsla, hsla};
use luma::theme::ThemeMode;

pub use colors_data::{BLACK_ALPHA_STEPS, DARK_FAMILIES, LIGHT_FAMILIES, COLORS_VERSION, WHITE_ALPHA_STEPS};
pub use colors_data::{RawColorScale, ColorValueKind};

pub fn families(mode: ThemeMode) -> &'static [RawColorScale] {
    match mode {
        ThemeMode::Light => LIGHT_FAMILIES,
        ThemeMode::Dark => DARK_FAMILIES,
    }
}

/// Twelve parsed steps for one named family, or `None` when the catalog has no such family.
pub fn family_steps(mode: ThemeMode, family: &str) -> Option<[Hsla; 12]> {
    let scale = families(mode).iter().find(|candidate| candidate.family == family)?;
    Some(scale.steps.map(parse_color))
}

pub fn parse_color(value: &str) -> Hsla {
    if let Some(hex) = value.strip_prefix('#') {
        let red = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let green = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let blue = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        return rgb_to_hsla(red, green, blue, 1.0);
    }

    let values: Vec<f32> = value
        .trim_start_matches("rgba(")
        .trim_end_matches(')')
        .split(',')
        .filter_map(|component| component.trim().parse().ok())
        .collect();
    if values.len() == 4 {
        return rgb_to_hsla(values[0] as u8, values[1] as u8, values[2] as u8, values[3]);
    }
    hsla(0.0, 0.0, 0.0, 0.0)
}

fn rgb_to_hsla(red: u8, green: u8, blue: u8, alpha: f32) -> Hsla {
    let red = f32::from(red) / 255.0;
    let green = f32::from(green) / 255.0;
    let blue = f32::from(blue) / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let lightness = (max + min) / 2.0;
    let delta = max - min;
    if delta == 0.0 {
        return hsla(0.0, 0.0, lightness, alpha);
    }
    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue = if max == red {
        ((green - blue) / delta).rem_euclid(6.0) / 6.0
    } else if max == green {
        ((blue - red) / delta + 2.0) / 6.0
    } else {
        ((red - green) / delta + 4.0) / 6.0
    };
    hsla(hue, saturation, lightness, alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_contains_all_v3_families_in_order() {
        assert_eq!(families(ThemeMode::Light).len(), 31);
        assert_eq!(families(ThemeMode::Light).first().unwrap().family, "gray");
        assert_eq!(families(ThemeMode::Light).last().unwrap().family, "gold");
        assert_eq!(COLORS_VERSION, "3.0.0");
    }

    #[test]
    fn alpha_values_keep_transparency() {
        assert!(parse_color(BLACK_ALPHA_STEPS[0]).a < 1.0);
        assert!(parse_color(WHITE_ALPHA_STEPS[11]).a > parse_color(WHITE_ALPHA_STEPS[0]).a);
    }
}
