//! Slider property mappings (primary action styling):
//!
//! | Part  | Token |
//! |-------|-------|
//! | Track | `border` (neutral track rail) |
//! | Fill  | `@primary_default` / `@primary_layer` |
//! | Thumb | `first(background,card)` |
//! | Ring  | `@primary_default` |
//!
//! Track uses `border` rather than `muted` because many tweakcn light themes
//! set `--muted` near white (e.g. 98% lightness), which disappears on card panels.

const SLIDER_STYLE_KEY: &str = "primary";

use gpui_luma::controls::slider::{SliderLook, SliderThumbSize};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use crate::controls::button::ButtonRadiusPreset;
use crate::look_context::LookContext;
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

/// Shared corner-radius preset resolution for slider parts.
///
/// Medium/large/full resolve to a pill/circle for the given cross-axis size.
fn resolve_slider_part_radius_preset(
    preset: ButtonRadiusPreset,
    metrics: &gpui_luma::theme::MetricTokens,
    cross_axis_size: f32,
) -> f32 {
    match preset {
        ButtonRadiusPreset::None => metrics.radius.none,
        ButtonRadiusPreset::Small => metrics.radius.sm,
        ButtonRadiusPreset::Medium | ButtonRadiusPreset::Large | ButtonRadiusPreset::Full => cross_axis_size / 2.0,
    }
}

/// Thumb corner radius presets for slider controls.
pub fn resolve_slider_thumb_radius_preset(
    preset: ButtonRadiusPreset,
    metrics: &gpui_luma::theme::MetricTokens,
    thumb_size: f32,
) -> f32 {
    resolve_slider_part_radius_preset(preset, metrics, thumb_size)
}

/// Track end-cap radius presets for slider controls.
pub fn resolve_slider_track_radius_preset(
    preset: ButtonRadiusPreset,
    metrics: &gpui_luma::theme::MetricTokens,
    track_height: f32,
) -> f32 {
    resolve_slider_part_radius_preset(preset, metrics, track_height)
}

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
    let rule = find_slider_color_rule(stylesheet, SLIDER_STYLE_KEY, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching slider color rule"))?;
    let colors = resolve_slider_color_rule(resolver, rule, layer)?;
    Ok(SliderColorTable {
        track_background: colors.track_background,
        fill_background: colors.fill_background,
        thumb_background: colors.thumb_background,
        thumb_border: colors.thumb_border,
    })
}

pub fn slider_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
    thumb_size: Option<SliderThumbSize>,
    state: InteractionState,
) -> SliderLook {
    let ctx = LookContext::new(mode, theme_mode, state);
    let state = ctx.state;
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let layer = state.layer();
    let thumb_shadow = thumb_shadow(ctx.theme_mode);
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "slider");
    let colors = resolve_slider_colors(&resolver, layer).unwrap_or_else(|_| SliderColorTable::fallback());
    let stylesheet = embedded_stylesheet();
    let (width, height, track_height, resolved_thumb_size_px, radius) = stylesheet
        .slider
        .metrics_for_size(size)
        .map(|rule| {
            let m = resolve_slider_metrics(rule, metrics, size);
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

    let thumb_size = thumb_size.map(slider_thumb_size).unwrap_or(resolved_thumb_size_px);

    SliderLook {
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

fn slider_thumb_size(size: SliderThumbSize) -> f32 {
    match size {
        SliderThumbSize::Sm => 12.0,
        SliderThumbSize::Md => 16.0,
        SliderThumbSize::Lg => 20.0,
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

    use super::slider_look;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use gpui_luma::controls::slider::SliderThumbSize;

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
    fn primary_slider_uses_border_track_and_primary_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = slider_look(&mode, ThemeMode::Light, ControlSize::Md, None, InteractionState::default());

        assert_eq!(look.track_background, catalog.color("border").expect("border"));
        assert_eq!(look.fill_background, catalog.color("primary").expect("primary"));
        assert_eq!(look.thumb_background, catalog.color("background").expect("background"));
        assert_ne!(look.track_background, catalog.color("secondary").expect("secondary"));
    }

    #[test]
    fn disabled_slider_track_background_matches_enabled() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let enabled = slider_look(&mode, ThemeMode::Light, ControlSize::Md, None, InteractionState::default());
        let disabled = slider_look(
            &mode,
            ThemeMode::Light,
            ControlSize::Md,
            None,
            InteractionState { disabled: true, ..InteractionState::default() },
        );

        assert_eq!(disabled.track_background, enabled.track_background);
        assert_eq!(disabled.track_background, catalog.color("border").expect("border"));
    }

    #[test]
    fn astrovista_light_track_is_border_not_white_muted() {
        let catalog = astrovista_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = slider_look(&mode, ThemeMode::Light, ControlSize::Md, None, InteractionState::default());

        let border = catalog.color("border").expect("border");
        let muted = catalog.color("muted").expect("muted");
        let card = catalog.color("card").expect("card");

        assert_eq!(look.track_background, border);
        assert_eq!(look.fill_background, catalog.color("primary").expect("primary"));
        assert_ne!(look.track_background, catalog.color("secondary").expect("secondary"));
        assert!(border.l > muted.l || (border.l - muted.l).abs() > 0.05);
        assert!(border.l < card.l - 0.05);
    }

    #[test]
    fn slider_size_changes_metrics() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");

        let sm = slider_look(&mode, ThemeMode::Light, ControlSize::Sm, None, InteractionState::default());
        let md = slider_look(&mode, ThemeMode::Light, ControlSize::Md, None, InteractionState::default());
        let lg = slider_look(&mode, ThemeMode::Light, ControlSize::Lg, None, InteractionState::default());

        assert!(sm.height < md.height);
        assert!(md.height < lg.height);
        assert!(sm.track_height < md.track_height);
        assert!(md.track_height < lg.track_height);
        assert!(sm.thumb_size < md.thumb_size);
        assert!(md.thumb_size < lg.thumb_size);
    }

    #[test]
    fn slider_radius_presets_use_square_and_pill_shapes() {
        use crate::controls::button::ButtonRadiusPreset;

        use super::{resolve_slider_thumb_radius_preset, resolve_slider_track_radius_preset};
        use gpui_luma::theme::MetricTokens;

        let metrics = MetricTokens::default();
        let thumb_size = 16.0;
        let track_height = 6.0;
        let thumb_circle = thumb_size / 2.0;
        let track_pill = track_height / 2.0;

        assert_eq!(resolve_slider_thumb_radius_preset(ButtonRadiusPreset::None, &metrics, thumb_size), 0.0);
        assert_eq!(resolve_slider_track_radius_preset(ButtonRadiusPreset::None, &metrics, track_height), 0.0);
        assert!(resolve_slider_thumb_radius_preset(ButtonRadiusPreset::Small, &metrics, thumb_size) < thumb_circle);
        assert!(resolve_slider_track_radius_preset(ButtonRadiusPreset::Small, &metrics, track_height) <= track_pill);
        assert!(
            resolve_slider_track_radius_preset(ButtonRadiusPreset::Small, &metrics, track_height * 2.0) < track_height
        );
        assert_eq!(resolve_slider_thumb_radius_preset(ButtonRadiusPreset::Medium, &metrics, thumb_size), thumb_circle);
        assert_eq!(resolve_slider_track_radius_preset(ButtonRadiusPreset::Medium, &metrics, track_height), track_pill);
    }

    #[test]
    fn slider_thumb_override_changes_thumb_metrics_only() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");

        let md_default = slider_look(&mode, ThemeMode::Light, ControlSize::Md, None, InteractionState::default());
        let md_large = slider_look(
            &mode,
            ThemeMode::Light,
            ControlSize::Md,
            Some(SliderThumbSize::Lg),
            InteractionState::default(),
        );

        assert_eq!(md_default.height, md_large.height);
        assert_eq!(md_default.track_height, md_large.track_height);
        assert!(md_large.thumb_size > md_default.thumb_size);
    }
}
