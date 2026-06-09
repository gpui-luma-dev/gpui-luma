//! Progress — muted track + primary fill.

use gpui_luma::controls::progress::ProgressAppearance;
use gpui_luma::theme::{InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{ColorSource, LookResolver, ResolvedColor};
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_progress_color_rule, resolve_progress_color_rule,
    resolve_progress_metrics,
};

const DEFAULT_PROGRESS_SIZE: f32 = 64.0;
const DEFAULT_PROGRESS_STROKE_WIDTH: f32 = 6.0;

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

pub fn progress_appearance(mode: &ShadcnModeTokens, enabled: bool) -> ProgressAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        progress_from_palette(&ctx, enabled)
    } else {
        progress_from_catalog(&ctx, enabled).unwrap_or_else(|err| panic!("progress properties: {err}"))
    }
}

pub fn progress_from_palette(ctx: &AppearanceContext, enabled: bool) -> ProgressAppearance {
    let palette = ctx.palette();

    ProgressAppearance {
        track_color: if enabled {
            palette.muted_background
        } else {
            palette.disabled_background
        },
        progress_color: if enabled {
            palette.primary.background
        } else {
            palette.disabled_foreground
        },
        size: DEFAULT_PROGRESS_SIZE,
        stroke_width: DEFAULT_PROGRESS_STROKE_WIDTH,
    }
}

pub fn progress_from_catalog(ctx: &AppearanceContext, enabled: bool) -> anyhow::Result<ProgressAppearance> {
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "progress");
    let colors = resolve_progress_colors(&resolver, enabled).unwrap_or_else(|_| ProgressColorTable::fallback());
    let stylesheet = embedded_stylesheet();
    let (size, stroke_width) = stylesheet
        .progress
        .metrics
        .as_ref()
        .map(resolve_progress_metrics)
        .map(|metrics| (metrics.size, metrics.stroke_width))
        .unwrap_or((DEFAULT_PROGRESS_SIZE, DEFAULT_PROGRESS_STROKE_WIDTH));

    Ok(ProgressAppearance {
        track_color: colors.track_color.hsla(),
        progress_color: colors.progress_color.hsla(),
        size,
        stroke_width,
    })
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use gpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::progress_appearance;

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
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn progress_uses_muted_track_and_primary_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let appearance = progress_appearance(&mode, true);

        assert_eq!(appearance.track_color, catalog.color("muted").expect("muted"));
        assert_eq!(appearance.progress_color, catalog.color("primary").expect("primary"));
    }
}
