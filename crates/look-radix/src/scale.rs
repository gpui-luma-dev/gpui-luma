//! Hand-authored 12-step scales (Radix-shaped dual space: color + gray).
//!
//! Radix Themes always carry at least:
//! - a chromatic **color** scale (12 steps)
//! - a neutral **gray** scale (12 steps)
//! - optional **destructive** (and other) scales
//!
//! Page chrome often mixes families (e.g. color step 3 → gray step 1).

use gpui::{Hsla, hsla};
use luma::theme::ThemeMode;
use luma_look_core::ResolvedColor;

/// 1-based Radix-style scale step (`1`…`12`).
pub type ScaleStep = u8;

pub const SCALE_LEN: usize = 12;

/// Named scale family used by semantic mappings and provenance.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ScaleFamily {
    /// Neutral 12-step scale.
    Gray,
    /// Chromatic 12-step scale (Radix "color" / accent family).
    Color,
    Destructive,
}

impl ScaleFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gray => "gray",
            Self::Color => "color",
            Self::Destructive => "destructive",
        }
    }
}

/// Twelve HSL steps for one family in one mode.
#[derive(Clone, Copy, Debug)]
pub struct ColorScale {
    steps: [Hsla; SCALE_LEN],
}

impl ColorScale {
    pub const fn new(steps: [Hsla; SCALE_LEN]) -> Self {
        Self { steps }
    }

    /// Returns the color for a 1-based step, clamping into `1..=12`.
    pub fn step(self, step: ScaleStep) -> Hsla {
        let idx = step.clamp(1, SCALE_LEN as u8) as usize - 1;
        self.steps[idx]
    }

    pub fn resolved(self, family: ScaleFamily, step: ScaleStep) -> ResolvedColor {
        ResolvedColor::scale_step(self.step(step), family.as_str(), step.clamp(1, SCALE_LEN as u8))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ModeScales {
    pub gray: ColorScale,
    pub color: ColorScale,
    pub destructive: ColorScale,
}

impl ModeScales {
    pub fn family(self, family: ScaleFamily) -> ColorScale {
        match family {
            ScaleFamily::Gray => self.gray,
            ScaleFamily::Color => self.color,
            ScaleFamily::Destructive => self.destructive,
        }
    }

    pub fn resolved(self, family: ScaleFamily, step: ScaleStep) -> ResolvedColor {
        self.family(family).resolved(family, step)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ScalePair {
    pub light: ModeScales,
    pub dark: ModeScales,
}

impl ScalePair {
    pub fn for_mode(self, mode: ThemeMode) -> ModeScales {
        match mode {
            ThemeMode::Light => self.light,
            ThemeMode::Dark => self.dark,
        }
    }
}

/// Built-in stub palettes approximating slate / indigo / red Radix scales.
pub fn built_in_scales() -> ScalePair {
    ScalePair {
        light: ModeScales { gray: light_gray(), color: light_color(), destructive: light_destructive() },
        dark: ModeScales { gray: dark_gray(), color: dark_color(), destructive: dark_destructive() },
    }
}

/// Generates a Radix-shaped chromatic scale anchored at color step 9.
///
/// The curves intentionally vary by mode: light themes spend more of the scale
/// above the anchor, while dark themes spend more of it below the anchor.
pub fn color_scale_from_seed(seed: Hsla, mode: ThemeMode) -> ColorScale {
    const LIGHTNESS_OFFSETS: [f32; SCALE_LEN] =
        [0.54, 0.50, 0.45, 0.38, 0.28, 0.18, 0.08, 0.03, 0.0, -0.07, -0.15, -0.25];
    const SATURATION_FACTORS: [f32; SCALE_LEN] = [1.0, 0.98, 0.95, 0.90, 0.85, 0.80, 0.75, 0.72, 1.0, 1.03, 1.07, 1.10];

    let offsets = match mode {
        ThemeMode::Light => LIGHTNESS_OFFSETS,
        ThemeMode::Dark => LIGHTNESS_OFFSETS.map(|offset| -offset),
    };
    let steps = std::array::from_fn(|index| {
        if index == 8 {
            return seed;
        }
        gpui::hsla(
            seed.h,
            (seed.s * SATURATION_FACTORS[index]).clamp(0.0, 1.0),
            (seed.l + offsets[index]).clamp(0.0, 1.0),
            seed.a,
        )
    });
    ColorScale::new(steps)
}

fn hsl(h_deg: f32, s_pct: f32, l_pct: f32) -> Hsla {
    hsla(h_deg / 360.0, s_pct / 100.0, l_pct / 100.0, 1.0)
}

fn light_gray() -> ColorScale {
    ColorScale::new([
        hsl(210.0, 20.0, 99.0),
        hsl(210.0, 20.0, 98.0),
        hsl(210.0, 16.0, 96.0),
        hsl(210.0, 14.0, 93.0),
        hsl(210.0, 12.0, 88.0),
        hsl(210.0, 11.0, 82.0),
        hsl(210.0, 10.0, 70.0),
        hsl(210.0, 10.0, 55.0),
        hsl(210.0, 12.0, 45.0),
        hsl(210.0, 14.0, 35.0),
        hsl(210.0, 16.0, 25.0),
        hsl(210.0, 20.0, 12.0),
    ])
}

fn dark_gray() -> ColorScale {
    ColorScale::new([
        hsl(210.0, 20.0, 8.0),
        hsl(210.0, 18.0, 10.0),
        hsl(210.0, 16.0, 14.0),
        hsl(210.0, 14.0, 18.0),
        hsl(210.0, 12.0, 24.0),
        hsl(210.0, 11.0, 32.0),
        hsl(210.0, 10.0, 42.0),
        hsl(210.0, 10.0, 55.0),
        hsl(210.0, 12.0, 68.0),
        hsl(210.0, 14.0, 78.0),
        hsl(210.0, 16.0, 88.0),
        hsl(210.0, 20.0, 96.0),
    ])
}

fn light_color() -> ColorScale {
    ColorScale::new([
        hsl(226.0, 100.0, 99.0),
        hsl(226.0, 100.0, 97.0),
        hsl(226.0, 95.0, 94.0),
        hsl(226.0, 90.0, 88.0),
        hsl(226.0, 85.0, 80.0),
        hsl(226.0, 80.0, 70.0),
        hsl(226.0, 75.0, 60.0),
        hsl(226.0, 70.0, 52.0),
        hsl(226.0, 70.0, 45.0),
        hsl(226.0, 72.0, 38.0),
        hsl(226.0, 75.0, 30.0),
        hsl(226.0, 80.0, 20.0),
    ])
}

fn dark_color() -> ColorScale {
    ColorScale::new([
        hsl(226.0, 50.0, 10.0),
        hsl(226.0, 45.0, 14.0),
        hsl(226.0, 45.0, 18.0),
        hsl(226.0, 50.0, 24.0),
        hsl(226.0, 55.0, 32.0),
        hsl(226.0, 60.0, 40.0),
        hsl(226.0, 65.0, 50.0),
        hsl(226.0, 70.0, 58.0),
        hsl(226.0, 75.0, 62.0),
        hsl(226.0, 80.0, 72.0),
        hsl(226.0, 85.0, 82.0),
        hsl(226.0, 90.0, 92.0),
    ])
}

fn light_destructive() -> ColorScale {
    ColorScale::new([
        hsl(0.0, 100.0, 99.0),
        hsl(0.0, 100.0, 97.0),
        hsl(0.0, 95.0, 94.0),
        hsl(0.0, 90.0, 88.0),
        hsl(0.0, 85.0, 80.0),
        hsl(0.0, 80.0, 72.0),
        hsl(0.0, 75.0, 62.0),
        hsl(0.0, 72.0, 55.0),
        hsl(0.0, 72.0, 48.0),
        hsl(0.0, 74.0, 40.0),
        hsl(0.0, 76.0, 32.0),
        hsl(0.0, 80.0, 22.0),
    ])
}

fn dark_destructive() -> ColorScale {
    ColorScale::new([
        hsl(0.0, 40.0, 10.0),
        hsl(0.0, 42.0, 14.0),
        hsl(0.0, 45.0, 18.0),
        hsl(0.0, 48.0, 24.0),
        hsl(0.0, 52.0, 32.0),
        hsl(0.0, 55.0, 40.0),
        hsl(0.0, 60.0, 48.0),
        hsl(0.0, 65.0, 55.0),
        hsl(0.0, 70.0, 58.0),
        hsl(0.0, 75.0, 68.0),
        hsl(0.0, 80.0, 78.0),
        hsl(0.0, 85.0, 90.0),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_clamps_and_is_one_based() {
        let gray = light_gray();
        assert_eq!(gray.step(1).l, gray.step(0).l);
        assert_eq!(gray.step(12).l, gray.step(99).l);
        assert!(gray.step(1).l > gray.step(12).l);
    }

    #[test]
    fn dark_gray_inverts_lightness_direction() {
        let light = light_gray();
        let dark = dark_gray();
        assert!(light.step(1).l > light.step(12).l);
        assert!(dark.step(1).l < dark.step(12).l);
    }

    #[test]
    fn resolved_carries_scale_provenance() {
        let scales = built_in_scales().for_mode(ThemeMode::Light);
        let color = scales.resolved(ScaleFamily::Color, 9);
        match color.source {
            luma_look_core::ColorSource::ScaleStep { family, step } => {
                assert_eq!(family, "color");
                assert_eq!(step, 9);
            }
            other => panic!("unexpected source: {other:?}"),
        }
    }

    #[test]
    fn color_and_gray_are_distinct_families() {
        let scales = built_in_scales().for_mode(ThemeMode::Light);
        let color_3 = scales.resolved(ScaleFamily::Color, 3).hsla();
        let gray_3 = scales.resolved(ScaleFamily::Gray, 3).hsla();
        assert_ne!(color_3.s, gray_3.s);
    }

    #[test]
    fn generated_color_scale_preserves_step_nine_seed() {
        let seed = hsla(0.58, 0.72, 0.58, 1.0);
        assert_eq!(color_scale_from_seed(seed, ThemeMode::Light).step(9), seed);
        assert_eq!(color_scale_from_seed(seed, ThemeMode::Dark).step(9), seed);
    }
}
