//! Stepper — accent track + accent-foreground fill (same color elements as progress).

use gpui_luma::controls::stepper::StepperLook;
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use crate::controls::progress::ProgressColorTable;
use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{ColorSource, LookResolver, ResolvedColor};
use crate::stylesheet::{StylesheetConfig, embedded_stylesheet, resolve_color_ref, resolve_stepper_metrics, ResolvedFields};
use crate::controls::progress::resolve_progress_colors_with_stylesheet;

const DEFAULT_STEP_BADGE_SIZE: f32 = 32.0;
const DEFAULT_TRACK_THICKNESS: f32 = 2.0;

#[derive(Clone, Debug)]
pub struct StepperColorTable {
    pub complete_bg: ResolvedColor,
    pub complete_fg: ResolvedColor,
    pub in_progress_bg: ResolvedColor,
    pub in_progress_fg: ResolvedColor,
    pub incomplete_bg: ResolvedColor,
    pub incomplete_border: ResolvedColor,
    pub incomplete_fg: ResolvedColor,
    pub track_active: ResolvedColor,
    pub track_muted: ResolvedColor,
}

impl StepperColorTable {
    pub fn fallback() -> Self {
        Self {
            complete_bg: ResolvedColor::fallback_foreground(),
            complete_fg: ResolvedColor::transparent(),
            in_progress_bg: ResolvedColor::fallback_foreground(),
            in_progress_fg: ResolvedColor::transparent(),
            incomplete_bg: ResolvedColor::transparent(),
            incomplete_border: ResolvedColor {
                value: gpui::hsla(0.0, 0.0, 0.0, 0.0),
                source: ColorSource::Transparent,
            },
            incomplete_fg: ResolvedColor::fallback_foreground(),
            track_active: ResolvedColor::fallback_foreground(),
            track_muted: ResolvedColor { value: gpui::hsla(0.0, 0.0, 0.0, 0.0), source: ColorSource::Transparent },
        }
    }
}

pub fn resolve_stepper_colors(resolver: &LookResolver<'_>, enabled: bool) -> anyhow::Result<StepperColorTable> {
    resolve_stepper_colors_with_stylesheet(resolver, embedded_stylesheet(), enabled)
}

pub fn resolve_stepper_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> anyhow::Result<StepperColorTable> {
    let progress = resolve_progress_colors_with_stylesheet(resolver, stylesheet, enabled)?;
    stepper_colors_from_progress(resolver, &progress, enabled)
}

fn stepper_colors_from_progress(
    resolver: &LookResolver<'_>,
    progress: &ProgressColorTable,
    enabled: bool,
) -> anyhow::Result<StepperColorTable> {
    let fields = ResolvedFields::default();
    let incomplete_bg = resolve_color_ref(resolver, "background", &fields)?;
    let incomplete_border = resolve_color_ref(resolver, "border", &fields)?;
    let incomplete_fg = resolve_color_ref(resolver, "muted-foreground", &fields)?;

    let (complete_bg, complete_fg, in_progress_bg, in_progress_fg, track_active, track_muted) = if enabled {
        (
            progress.progress_color.clone(),
            progress.track_color.clone(),
            progress.progress_color.clone(),
            progress.track_color.clone(),
            progress.progress_color.clone(),
            progress.track_color.clone(),
        )
    } else {
        // Disabled progress collapses track/progress to the same token; badges need
        // checkbox-like contrast (muted fill + muted-foreground glyph).
        let muted = resolve_color_ref(resolver, "muted", &fields)?;
        (
            muted.clone(),
            progress.progress_color.clone(),
            muted.clone(),
            progress.progress_color.clone(),
            progress.progress_color.clone(),
            muted,
        )
    };

    Ok(StepperColorTable {
        complete_bg,
        complete_fg,
        in_progress_bg,
        in_progress_fg,
        incomplete_bg,
        incomplete_border,
        incomplete_fg,
        track_active,
        track_muted,
    })
}

pub fn stepper_look(mode: &ShadcnModeTokens, enabled: bool, size: ControlSize) -> StepperLook {
    let ctx = LookContext::new(mode, ThemeMode::Light, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "stepper");
    let colors = resolve_stepper_colors(&resolver, enabled).unwrap_or_else(|_| StepperColorTable::fallback());
    let stylesheet = embedded_stylesheet();
    let (step_badge_size, track_thickness) = stylesheet
        .stepper
        .metrics_for_size(size)
        .map(resolve_stepper_metrics)
        .map(|metrics| (metrics.step_badge_size, metrics.track_thickness))
        .unwrap_or((DEFAULT_STEP_BADGE_SIZE, DEFAULT_TRACK_THICKNESS));

    StepperLook {
        complete_bg: colors.complete_bg.hsla(),
        complete_fg: colors.complete_fg.hsla(),
        in_progress_bg: colors.in_progress_bg.hsla(),
        in_progress_fg: colors.in_progress_fg.hsla(),
        incomplete_bg: colors.incomplete_bg.hsla(),
        incomplete_border: colors.incomplete_border.hsla(),
        incomplete_fg: colors.incomplete_fg.hsla(),
        track_active_color: colors.track_active.hsla(),
        track_muted_color: colors.track_muted.hsla(),
        step_badge_size,
        track_thickness,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;

    use super::stepper_look;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn stepper_uses_accent_track_and_accent_foreground_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = stepper_look(&mode, true, gpui_luma::theme::ControlSize::Md);

        assert_eq!(look.complete_bg, catalog.color("accent-foreground").expect("accent-foreground"));
        assert_eq!(look.complete_fg, catalog.color("accent").expect("accent"));
        assert_eq!(look.track_active_color, catalog.color("accent-foreground").expect("accent-foreground"));
        assert_eq!(look.track_muted_color, catalog.color("accent").expect("accent"));
    }

    #[test]
    fn disabled_stepper_complete_badge_has_checkmark_contrast() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = stepper_look(&mode, false, gpui_luma::theme::ControlSize::Md);

        assert_eq!(look.complete_bg, catalog.color("muted").expect("muted"));
        assert_eq!(look.complete_fg, catalog.color("muted-foreground").expect("muted-foreground"));
        assert_ne!(look.complete_bg, look.complete_fg);
    }

    #[test]
    fn stepper_size_changes_badge_metrics() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let small = stepper_look(&mode, true, gpui_luma::theme::ControlSize::Sm);
        let large = stepper_look(&mode, true, gpui_luma::theme::ControlSize::Lg);

        assert!(small.step_badge_size < large.step_badge_size);
    }
}
