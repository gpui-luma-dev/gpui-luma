//! Twelve-step Radix scales (named or generated; color + gray).
//!
//! Radix Themes always carry at least:
//! - a chromatic **color** scale (12 steps)
//! - a neutral **gray** scale (12 steps)
//! - optional **destructive** (and other) scales
//!
//! Page chrome often mixes families (e.g. color step 3 → gray step 1).

use gpui::Hsla;
use luma::theme::ThemeMode;
use luma::theme::provenance::ResolvedColor;

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
///
/// `palette` is the Radix palette the steps came from (`"indigo"`, `"slate"`, …) and rides
/// along as color provenance. Hand-seeded scales carry [`CUSTOM_PALETTE`].
#[derive(Clone, Copy, Debug)]
pub struct ColorScale {
    steps: [Hsla; SCALE_LEN],
    palette: &'static str,
}

/// Provenance for a scale generated from a seed rather than a named Radix palette.
pub const CUSTOM_PALETTE: &str = "custom";

impl ColorScale {
    pub const fn new(steps: [Hsla; SCALE_LEN]) -> Self {
        Self { steps, palette: CUSTOM_PALETTE }
    }

    pub const fn named(palette: &'static str, steps: [Hsla; SCALE_LEN]) -> Self {
        Self { steps, palette }
    }

    pub fn palette(self) -> &'static str {
        self.palette
    }

    /// Returns the color for a 1-based step, clamping into `1..=12`.
    pub fn step(self, step: ScaleStep) -> Hsla {
        let idx = step.clamp(1, SCALE_LEN as u8) as usize - 1;
        self.steps[idx]
    }

    pub fn resolved(self, step: ScaleStep) -> ResolvedColor {
        ResolvedColor::scale_step(self.step(step), self.palette, step.clamp(1, SCALE_LEN as u8))
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
        self.family(family).resolved(step)
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

#[cfg(test)]
mod tests {
    use gpui::hsla;

    use super::*;
    use crate::palette::{Accent, Gray, scale_pair};

    fn color_scale_from_seed(seed: Hsla, mode: ThemeMode) -> ColorScale {
        crate::generate_colors(
            crate::CustomColors {
                accent: seed,
                gray: gpui::rgb(0x8b8d98).into(),
                background: if mode == ThemeMode::Light {
                    gpui::white()
                } else {
                    gpui::black()
                },
            },
            mode,
        )
        .accent
    }

    fn indigo_slate() -> ScalePair {
        scale_pair(Accent::Indigo, Gray::Auto)
    }

    #[test]
    fn step_clamps_and_is_one_based() {
        let gray = indigo_slate().light.gray;
        assert_eq!(gray.step(1).l, gray.step(0).l);
        assert_eq!(gray.step(12).l, gray.step(99).l);
        assert!(gray.step(1).l > gray.step(12).l);
    }

    #[test]
    fn dark_gray_inverts_lightness_direction() {
        let scales = indigo_slate();
        assert!(scales.light.gray.step(1).l > scales.light.gray.step(12).l);
        assert!(scales.dark.gray.step(1).l < scales.dark.gray.step(12).l);
    }

    /// Seeded scales belong to no palette. Named provenance is covered by
    /// `palette::tests::indigo_slate_carries_real_radix_steps_and_provenance`.
    #[test]
    fn seeded_scales_report_custom_provenance() {
        let seeded = color_scale_from_seed(hsla(0.58, 0.72, 0.58, 1.0), ThemeMode::Light);
        match seeded.resolved(9).source {
            luma::theme::provenance::ColorSource::ScaleStep { family, step } => {
                assert_eq!(family, CUSTOM_PALETTE);
                assert_eq!(step, 9);
            }
            other => panic!("unexpected source: {other:?}"),
        }
    }

    #[test]
    fn color_and_gray_are_distinct_families() {
        let scales = indigo_slate().for_mode(ThemeMode::Light);
        let color_3 = scales.resolved(ScaleFamily::Color, 3).hsla();
        let gray_3 = scales.resolved(ScaleFamily::Gray, 3).hsla();
        assert_ne!(color_3.s, gray_3.s);
    }

    #[test]
    fn generated_color_scale_preserves_step_nine_seed() {
        let seed = hsla(0.58, 0.72, 0.58, 1.0);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let actual: gpui::Rgba = color_scale_from_seed(seed, mode).step(9).into();
            let expected: gpui::Rgba = seed.into();
            assert!((actual.r - expected.r).abs() <= 1.0 / 255.0);
            assert!((actual.g - expected.g).abs() <= 1.0 / 255.0);
            assert!((actual.b - expected.b).abs() <= 1.0 / 255.0);
        }
    }
}
