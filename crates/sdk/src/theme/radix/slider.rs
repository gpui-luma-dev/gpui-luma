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

use crate::controls::slider::SliderAppearance;
use crate::theme::{InteractionLayer, InteractionState, LumaTheme, ThemeMode};

use super::focus::focus_ring_color;
use super::resolve::{resolve_action_layer, resolve_color};
use super::RadixButtonStyle;
use super::catalog::CssTokenMap;
use super::mode::RadixModeTokens;

pub(crate) fn slider_appearance(
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> SliderAppearance {
    if mode.catalog.tokens.is_empty() {
        slider_appearance_from_palette(&mode.palette, &mode.metrics, theme_mode, state)
    } else {
        slider_appearance_from_catalog(&mode.catalog, &mode.metrics, theme_mode, state)
            .unwrap_or_else(|err| panic!("slider properties: {err}"))
    }
}

fn slider_appearance_from_palette(
    palette: &super::palette::RadixPalette,
    metrics: &crate::theme::MetricTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> SliderAppearance {
    let layer = state.layer();
    let thumb_shadow = {
        let native = LumaTheme::native();
        native.mode(theme_mode).elevation.thumb.to_box_shadows()
    };

    let fill_background = match layer {
        InteractionLayer::Disabled => palette.disabled_foreground,
        InteractionLayer::Pressed => palette.primary.pressed_background,
        InteractionLayer::Hovered => palette.primary.hover_background,
        InteractionLayer::Default => palette.primary.background,
    };

    let track_background = if state.disabled {
        palette.disabled_background
    } else {
        palette.border_default
    };

    let (thumb_background, thumb_border) = if state.disabled {
        (palette.disabled_foreground, palette.disabled_background)
    } else {
        (palette.panel_background, palette.primary.background)
    };

    SliderAppearance {
        track_background,
        fill_background,
        thumb_background,
        thumb_border,
        thumb_shadow,
        focus_ring: state.focused.then_some(palette.focus_ring),
        width: 260.0,
        height: 32.0,
        track_height: 8.0,
        thumb_size: 18.0,
        radius: metrics.radius.pill,
    }
}

pub(crate) fn slider_appearance_from_catalog(
    catalog: &CssTokenMap,
    metrics: &crate::theme::MetricTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> anyhow::Result<SliderAppearance> {
    let layer = state.layer();
    let thumb_shadow = {
        let native = LumaTheme::native();
        native.mode(theme_mode).elevation.thumb.to_box_shadows()
    };

    let fill_background = match layer {
        InteractionLayer::Disabled => resolve_color(catalog, "muted-foreground")?,
        _ => resolve_action_layer(catalog, RadixButtonStyle::Primary, layer)?,
    };

    let track_background = if state.disabled {
        resolve_color(catalog, "muted")?
    } else {
        resolve_color(catalog, "border")?
    };

    let (thumb_background, thumb_border) = if state.disabled {
        let thumb = resolve_color(catalog, "muted-foreground")?;
        let border = resolve_color(catalog, "muted")?;
        (thumb, border)
    } else {
        let thumb = catalog.color_first(&["background", "card"])?;
        let border = resolve_action_layer(catalog, RadixButtonStyle::Primary, InteractionLayer::Default)?;
        (thumb, border)
    };

    Ok(SliderAppearance {
        track_background,
        fill_background,
        thumb_background,
        thumb_border,
        thumb_shadow,
        focus_ring: state.focused.then(|| focus_ring_color(catalog)).transpose()?,
        width: 260.0,
        height: 32.0,
        track_height: 8.0,
        thumb_size: 18.0,
        radius: metrics.radius.pill,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::theme::InteractionState;

    use super::super::catalog::CssTokenMap;
    use super::super::mode::RadixModeTokens;
    use super::slider_appearance_from_catalog;

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
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let appearance = slider_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            crate::theme::ThemeMode::Light,
            InteractionState::default(),
        )
        .expect("slider");

        assert_eq!(appearance.track_background, catalog.color("border").expect("border"));
        assert_eq!(appearance.fill_background, catalog.color("primary").expect("primary"));
        assert_eq!(appearance.thumb_background, catalog.color("background").expect("background"));
        assert_ne!(appearance.track_background, catalog.color("secondary").expect("secondary"));
    }

    #[test]
    fn astrovista_light_track_is_border_not_white_muted() {
        let catalog = astrovista_catalog();
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let appearance = slider_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            crate::theme::ThemeMode::Light,
            InteractionState::default(),
        )
        .expect("slider");

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
