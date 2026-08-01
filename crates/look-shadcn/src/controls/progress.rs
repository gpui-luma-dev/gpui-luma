//! Progress — accent track + accent-foreground fill.

use gpui_luma::controls::progress::ProgressLook;
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{ColorSource, LookResolver, ResolvedColor};
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_progress_color_rule, resolve_progress_color_rule,
    resolve_progress_metrics,
};

const DEFAULT_PROGRESS_SIZE: f32 = 64.0;
const DEFAULT_PROGRESS_STROKE_WIDTH: f32 = 6.0;
const DEFAULT_PROGRESS_TRACK_HEIGHT: f32 = 6.0;
const DEFAULT_PROGRESS_THUMB_SIZE: f32 = 16.0;

#[derive(Clone, Debug)]
pub struct ProgressColorTable {
    pub track_color: ResolvedColor,
    pub progress_color: ResolvedColor,
}

impl ProgressColorTable {
    pub fn fallback() -> Self {
        Self {
            track_color: ResolvedColor { value: gpui::hsla(0.0, 0.0, 0.0, 0.0), source: ColorSource::Transparent },
            progress_color: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_progress_colors(resolver: &LookResolver<'_>, enabled: bool) -> anyhow::Result<ProgressColorTable> {
    resolve_progress_colors_with_stylesheet(resolver, embedded_stylesheet(), enabled)
}

pub fn resolve_progress_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> anyhow::Result<ProgressColorTable> {
    let rule = find_progress_color_rule(stylesheet, enabled)
        .ok_or_else(|| anyhow::anyhow!("no matching progress color rule"))?;
    let colors = resolve_progress_color_rule(resolver, rule)?;
    Ok(ProgressColorTable { track_color: colors.track_color, progress_color: colors.progress_color })
}

pub fn progress_look(mode: &ShadcnModeTokens, enabled: bool, size: ControlSize) -> ProgressLook {
    let ctx = LookContext::new(mode, ThemeMode::Light, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "progress");
    let colors = resolve_progress_colors(&resolver, enabled).unwrap_or_else(|_| ProgressColorTable::fallback());
    let stylesheet = embedded_stylesheet();
    let (size, stroke_width, track_height, thumb_size) = stylesheet
        .progress
        .metrics_for_size(size)
        .map(resolve_progress_metrics)
        .map(|metrics| (metrics.size, metrics.stroke_width, metrics.track_height, metrics.thumb_size))
        .unwrap_or((
            DEFAULT_PROGRESS_SIZE,
            DEFAULT_PROGRESS_STROKE_WIDTH,
            DEFAULT_PROGRESS_TRACK_HEIGHT,
            DEFAULT_PROGRESS_THUMB_SIZE,
        ));

    ProgressLook {
        track_color: colors.track_color.hsla(),
        progress_color: colors.progress_color.hsla(),
        thumb_color: colors.progress_color.hsla(),
        track_height,
        thumb_size,
        size,
        stroke_width,
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use gpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::progress_look;

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
    fn progress_uses_accent_track_and_accent_foreground_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = progress_look(&mode, true, gpui_luma::theme::ControlSize::Md);

        assert_eq!(look.track_color, catalog.color("accent").expect("accent"));
        assert_eq!(look.progress_color, catalog.color("accent-foreground").expect("accent-foreground"));
    }

    #[test]
    fn progress_size_changes_metrics() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let small = progress_look(&mode, true, gpui_luma::theme::ControlSize::Sm);
        let medium = progress_look(&mode, true, gpui_luma::theme::ControlSize::Md);
        let large = progress_look(&mode, true, gpui_luma::theme::ControlSize::Lg);

        assert!(small.size < medium.size);
        assert!(medium.size < large.size);
        assert!(small.stroke_width < medium.stroke_width);
        assert!(medium.stroke_width < large.stroke_width);
    }
}
