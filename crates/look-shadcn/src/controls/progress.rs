//! Progress — muted track + primary fill.

use gpui_luma::controls::progress::ProgressAppearance;
use gpui_luma::theme::{InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{ColorSource, LookResolver, ResolvedColor};

use gpui_luma_look_shadcn_macros::declare_look_table;

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

declare_look_table! {
    name: resolve_progress_colors,
    inputs: {
        enabled: bool,
    },
    output: ProgressColorTable { track_color, progress_color },
    matrix: [
        [true]  => "muted" | "primary",
        [false] => "muted-foreground" | "muted-foreground",
    ]
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
        size: 64.0,
        stroke_width: 6.0,
    }
}

pub fn progress_from_catalog(ctx: &AppearanceContext, enabled: bool) -> anyhow::Result<ProgressAppearance> {
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "progress");
    let colors = resolve_progress_colors(&resolver, enabled).unwrap_or_else(|_| ProgressColorTable::fallback());

    Ok(ProgressAppearance {
        track_color: colors.track_color.hsla(),
        progress_color: colors.progress_color.hsla(),
        size: 64.0,
        stroke_width: 6.0,
    })
}

#[cfg(test)]
mod tests {


    use std::collections::BTreeMap;
    use gpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::provenance::ColorSource;
    use super::{progress_appearance, resolve_progress_colors_metadata};

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
