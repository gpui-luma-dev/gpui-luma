//! Slider property mappings (Radix Themes surface / shadcn):
//!
//! | Part  | Token                          |
//! |-------|--------------------------------|
//! | Track | `border` (neutral track rail)  |
//! | Fill  | `primary` (+ interaction)      |
//! | Thumb | `background` / `card`          |
//! | Ring  | `primary` on thumb border      |
//!
//! Track uses `border` rather than `muted` because many tweakcn light themes
//! set `--muted` near white (e.g. 98% lightness), which disappears on card panels.

use gpui_luma::controls::slider::SliderAppearance;
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::elevation::thumb_shadow;
use crate::focus::focus_ring_color;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_slider_color_rule, resolve_slider_color_rule, resolve_slider_metrics,
};

const DEFAULT_SLIDER_WIDTH: f32 = 260.0;
const DEFAULT_SLIDER_HEIGHT: f32 = 32.0;
const DEFAULT_SLIDER_TRACK_HEIGHT: f32 = 8.0;
const DEFAULT_SLIDER_THUMB_SIZE: f32 = 18.0;

#[derive(Clone, Debug)]
pub struct SliderColorTable {
    pub track_background: ResolvedColor,
    pub fill_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
}

impl SliderColorTable {
    pub fn fallback() -> Self {
        Self {
            track_background: ResolvedColor::transparent(),
            fill_background: ResolvedColor::fallback_foreground(),
            thumb_background: ResolvedColor::fallback_foreground(),
            thumb_border: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_slider_colors(resolver: &LookResolver<'_>, layer: InteractionLayer) -> anyhow::Result<SliderColorTable> {
    resolve_slider_colors_with_stylesheet(resolver, embedded_stylesheet(), layer)
}

pub fn resolve_slider_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    layer: InteractionLayer,
) -> anyhow::Result<SliderColorTable> {
    let rule =
        find_slider_color_rule(stylesheet, layer).ok_or_else(|| anyhow::anyhow!("no matching slider color rule"))?;
    let colors = resolve_slider_color_rule(resolver, rule, layer)?;
    Ok(SliderColorTable {
        track_background: colors.track_background,
        fill_background: colors.fill_background,
        thumb_background: colors.thumb_background,
        thumb_border: colors.thumb_border,
    })
}

pub fn slider_appearance(mode: &ShadcnModeTokens, theme_mode: ThemeMode, state: InteractionState) -> SliderAppearance {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    let state = ctx.state;
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let layer = state.layer();
    let thumb_shadow = thumb_shadow(ctx.theme_mode);
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "slider");
    let colors = resolve_slider_colors(&resolver, layer).unwrap_or_else(|_| SliderColorTable::fallback());
    let stylesheet = embedded_stylesheet();
    let (width, height, track_height, thumb_size, radius) = stylesheet
        .slider
        .metrics_for_size(ControlSize::Md)
        .map(|rule| {
            let m = resolve_slider_metrics(rule, metrics);
            (m.width, m.height, m.track_height, m.thumb_size, m.radius)
        })
        .unwrap_or((
            DEFAULT_SLIDER_WIDTH,
            DEFAULT_SLIDER_HEIGHT,
            DEFAULT_SLIDER_TRACK_HEIGHT,
            DEFAULT_SLIDER_THUMB_SIZE,
            metrics.radius.pill,
        ));
    let focus_ring = state
        .focused
        .then(|| focus_ring_color(catalog))
        .transpose()
        .unwrap_or_else(|err| panic!("slider properties: {err}"));

    SliderAppearance {
        track_background: colors.track_background.hsla(),
        fill_background: colors.fill_background.hsla(),
        thumb_background: colors.thumb_background.hsla(),
        thumb_border: colors.thumb_border.hsla(),
        thumb_shadow,
        focus_ring,
        width,
        height,
        track_height,
        thumb_size,
        radius,
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use gpui_luma::theme::ThemeMode;

    use gpui_luma::theme::InteractionState;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::slider_appearance;

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
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.8 0.02 200)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    fn astrovista_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "hsl(15.1765 72.6496% 54.1176%)".into()),
            ("primary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("secondary".into(), "hsl(217.2973 44.0476% 32.9412%)".into()),
            ("secondary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("background".into(), "hsl(204.0000 12.1951% 91.9608%)".into()),
            ("foreground".into(), "hsl(0 0% 20%)".into()),
            ("muted".into(), "hsl(210 20.0000% 98.0392%)".into()),
            ("muted-foreground".into(), "hsl(220 8.9362% 46.0784%)".into()),
            ("accent".into(), "hsl(207.6923 46.4286% 89.0196%)".into()),
            ("border".into(), "hsl(0 0% 80%)".into()),
            ("input".into(), "hsl(220 15.7895% 96.2745%)".into()),
            ("ring".into(), "hsl(13.2143 73.0435% 54.9020%)".into()),
            ("card".into(), "hsl(0 0% 100%)".into()),
        ]))
    }

    #[test]
    fn default_slider_uses_border_track_and_primary_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let appearance = slider_appearance(&mode, gpui_luma::theme::ThemeMode::Light, InteractionState::default());

        assert_eq!(appearance.track_background, catalog.color("border").expect("border"));
        assert_eq!(appearance.fill_background, catalog.color("primary").expect("primary"));
        assert_eq!(appearance.thumb_background, catalog.color("background").expect("background"));
        assert_ne!(appearance.track_background, catalog.color("secondary").expect("secondary"));
    }

    #[test]
    fn astrovista_light_track_is_border_not_white_muted() {
        let catalog = astrovista_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let appearance = slider_appearance(&mode, gpui_luma::theme::ThemeMode::Light, InteractionState::default());

        let border = catalog.color("border").expect("border");
        let muted = catalog.color("muted").expect("muted");
        let card = catalog.color("card").expect("card");

        assert_eq!(appearance.track_background, border);
        assert_eq!(appearance.fill_background, catalog.color("primary").expect("primary"));
        assert_ne!(appearance.track_background, catalog.color("secondary").expect("secondary"));
        assert!(border.l > muted.l || (border.l - muted.l).abs() > 0.05);
        assert!(border.l < card.l - 0.05);
    }
}
