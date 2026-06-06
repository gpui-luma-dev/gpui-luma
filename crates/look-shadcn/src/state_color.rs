use gpui::Hsla;

use gpui_luma::theme::{InteractionLayer, ThemeMode};

use crate::catalog::CssTokenMap;
use crate::color::adjust_lightness;
use crate::palette::ShadcnPalette;
use crate::tokens::ShadcnToken;

const TOKEN_COUNT: usize = 19;
const LAYER_COUNT: usize = 4;

/// Pre-indexed `(ShadcnToken, InteractionLayer)` colors for O(1) runtime lookup.
#[derive(Clone, Debug)]
pub struct StateColorTable {
    colors: [Hsla; TOKEN_COUNT * LAYER_COUNT],
}

impl StateColorTable {
    pub fn from_catalog(catalog: &CssTokenMap, palette: &ShadcnPalette, theme_mode: ThemeMode) -> Self {
        let mut colors = [Hsla::default(); TOKEN_COUNT * LAYER_COUNT];
        for token in ShadcnToken::ALL {
            let base = token_base_from_palette(palette, token);
            for layer in all_layers() {
                colors[slot(token, layer)] =
                    resolve_state_color(catalog, palette, token, layer, base, theme_mode, None);
            }
        }
        Self { colors }
    }

    pub fn from_luma_palette(palette: &ShadcnPalette, theme_mode: ThemeMode) -> Self {
        let catalog = CssTokenMap::default();
        let mut colors = [Hsla::default(); TOKEN_COUNT * LAYER_COUNT];
        for token in ShadcnToken::ALL {
            let base = token_base_from_palette(palette, token);
            for layer in all_layers() {
                colors[slot(token, layer)] = resolve_state_color(
                    &catalog,
                    palette,
                    token,
                    layer,
                    base,
                    theme_mode,
                    luma_state_override(token, layer, palette),
                );
            }
        }
        Self { colors }
    }

    pub fn get(&self, token: ShadcnToken, layer: InteractionLayer) -> Hsla {
        self.colors[slot(token, layer)]
    }
}

pub(crate) fn resolve_state_color(
    catalog: &CssTokenMap,
    palette: &ShadcnPalette,
    token: ShadcnToken,
    layer: InteractionLayer,
    base: Hsla,
    theme_mode: ThemeMode,
    luma_override: Option<Hsla>,
) -> Hsla {
    if let Some(color) = luma_override {
        return color;
    }
    if let Some(color) = catalog_state_color(catalog, token.css_name(), layer) {
        return color;
    }
    match layer {
        InteractionLayer::Default => base,
        InteractionLayer::Disabled => disabled_color(token, palette),
        InteractionLayer::Hovered => algorithmic_state_color(base, InteractionLayer::Hovered, theme_mode, true),
        InteractionLayer::Pressed => algorithmic_state_color(base, InteractionLayer::Pressed, theme_mode, true),
    }
}

pub(crate) fn catalog_state_color(catalog: &CssTokenMap, css_name: &str, layer: InteractionLayer) -> Option<Hsla> {
    let key = state_key(css_name, layer);
    catalog.color(&key).ok()
}

pub(crate) fn algorithmic_state_color(
    base: Hsla,
    layer: InteractionLayer,
    theme_mode: ThemeMode,
    filled: bool,
) -> Hsla {
    let delta = match (theme_mode, layer, filled) {
        (_, InteractionLayer::Default | InteractionLayer::Disabled, _) => 0.0,
        // Subtle shifts — GPUI renders sRGB/Hsla; large Oklch steps clip harshly after round-trip.
        (ThemeMode::Light, InteractionLayer::Hovered, true) => -0.03,
        (ThemeMode::Light, InteractionLayer::Pressed, true) => -0.06,
        (ThemeMode::Light, InteractionLayer::Hovered, false) => -0.02,
        (ThemeMode::Light, InteractionLayer::Pressed, false) => -0.04,
        (ThemeMode::Dark, InteractionLayer::Hovered, true) => 0.04,
        (ThemeMode::Dark, InteractionLayer::Pressed, true) => 0.08,
        (ThemeMode::Dark, InteractionLayer::Hovered, false) => 0.03,
        (ThemeMode::Dark, InteractionLayer::Pressed, false) => 0.06,
    };
    if delta == 0.0 {
        base
    } else {
        adjust_lightness(base, delta)
    }
}

pub(crate) fn token_base_from_palette(palette: &ShadcnPalette, token: ShadcnToken) -> Hsla {
    match token {
        ShadcnToken::Background => palette.app_background,
        ShadcnToken::Foreground => palette.app_foreground,
        ShadcnToken::Card => palette.panel_background,
        ShadcnToken::CardForeground => palette.body_text,
        ShadcnToken::Popover => palette.panel_background,
        ShadcnToken::PopoverForeground => palette.body_text,
        ShadcnToken::Primary => palette.primary.background,
        ShadcnToken::PrimaryForeground => palette.primary.foreground,
        ShadcnToken::Secondary => palette.secondary.background,
        ShadcnToken::SecondaryForeground => palette.secondary.foreground,
        ShadcnToken::Muted => palette.muted_background,
        ShadcnToken::MutedForeground => palette.app_muted_foreground,
        ShadcnToken::Accent => palette.accent_background,
        ShadcnToken::AccentForeground => palette.accent_foreground,
        ShadcnToken::Destructive => palette.destructive_background,
        ShadcnToken::DestructiveForeground => palette.destructive_foreground,
        ShadcnToken::Border => palette.border_default,
        ShadcnToken::Input => palette.input_background,
        ShadcnToken::Ring => palette.focus_ring,
    }
}

fn disabled_color(token: ShadcnToken, palette: &ShadcnPalette) -> Hsla {
    if token.is_foreground() {
        palette.disabled_foreground
    } else {
        palette.disabled_background
    }
}

fn luma_state_override(token: ShadcnToken, layer: InteractionLayer, palette: &ShadcnPalette) -> Option<Hsla> {
    match (token, layer) {
        (ShadcnToken::Primary, InteractionLayer::Hovered) => Some(palette.primary.hover_background),
        (ShadcnToken::Primary, InteractionLayer::Pressed) => Some(palette.primary.pressed_background),
        (ShadcnToken::Secondary, InteractionLayer::Hovered) => Some(palette.secondary.hover_background),
        (ShadcnToken::Secondary, InteractionLayer::Pressed) => Some(palette.secondary.pressed_background),
        (ShadcnToken::Accent, InteractionLayer::Hovered) => Some(palette.ghost.hover_background),
        (ShadcnToken::Accent, InteractionLayer::Pressed) => Some(palette.ghost.pressed_background),
        _ => None,
    }
}

fn state_key(css_name: &str, layer: InteractionLayer) -> String {
    match layer {
        InteractionLayer::Default => css_name.to_string(),
        InteractionLayer::Hovered => format!("{css_name}-hover"),
        InteractionLayer::Pressed => format!("{css_name}-pressed"),
        InteractionLayer::Disabled => format!("{css_name}-disabled"),
    }
}

fn slot(token: ShadcnToken, layer: InteractionLayer) -> usize {
    token.index() * LAYER_COUNT + layer_index(layer)
}

fn layer_index(layer: InteractionLayer) -> usize {
    match layer {
        InteractionLayer::Default => 0,
        InteractionLayer::Hovered => 1,
        InteractionLayer::Pressed => 2,
        InteractionLayer::Disabled => 3,
    }
}

fn all_layers() -> [InteractionLayer; 4] {
    [
        InteractionLayer::Default,
        InteractionLayer::Hovered,
        InteractionLayer::Pressed,
        InteractionLayer::Disabled,
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::palette::ShadcnPalette;

    use super::{algorithmic_state_color, catalog_state_color, resolve_state_color};
    use crate::tokens::ShadcnToken;

    use gpui_luma::theme::InteractionLayer;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("primary-hover".into(), "oklch(0.45 0.20 355.8943)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    #[test]
    fn explicit_catalog_override_wins() {
        let catalog = sample_catalog();
        let hover = catalog_state_color(&catalog, "primary", InteractionLayer::Hovered).expect("override");
        let palette = ShadcnPalette::from_catalog(&catalog, ThemeMode::Light).expect("palette");
        let base = palette.primary.background;
        assert_ne!(hover, base);
    }

    #[test]
    fn light_primary_hover_darkens() {
        let catalog = CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.75 0.15 250)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.65 0.10 250)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.98 0.01 250)".into()),
            ("foreground".into(), "oklch(0.20 0.05 250)".into()),
            ("muted".into(), "oklch(0.90 0.01 250)".into()),
            ("muted-foreground".into(), "oklch(0.45 0.02 250)".into()),
            ("accent".into(), "oklch(0.90 0.05 250)".into()),
            ("accent-foreground".into(), "oklch(0.20 0.05 250)".into()),
            ("border".into(), "oklch(0.85 0.01 250)".into()),
            ("input".into(), "oklch(0.85 0.01 250)".into()),
            ("ring".into(), "oklch(0.55 0.15 250)".into()),
            ("card".into(), "oklch(0.96 0.01 250)".into()),
        ]));
        let palette = ShadcnPalette::from_catalog(&catalog, ThemeMode::Light).expect("palette");
        let base = palette.primary.background;
        let hover = resolve_state_color(
            &catalog,
            &palette,
            ShadcnToken::Primary,
            InteractionLayer::Hovered,
            base,
            ThemeMode::Light,
            None,
        );
        assert!(hover.l < base.l, "light hover should darken");
    }

    #[test]
    fn dark_primary_hover_lightens() {
        let catalog = CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.35 0.15 250)".into()),
            ("primary-foreground".into(), "oklch(0.98 0 0)".into()),
            ("secondary".into(), "oklch(0.40 0.10 250)".into()),
            ("secondary-foreground".into(), "oklch(0.98 0 0)".into()),
            ("background".into(), "oklch(0.15 0.02 250)".into()),
            ("foreground".into(), "oklch(0.95 0.01 250)".into()),
            ("muted".into(), "oklch(0.25 0.02 250)".into()),
            ("muted-foreground".into(), "oklch(0.70 0.01 250)".into()),
            ("accent".into(), "oklch(0.30 0.05 250)".into()),
            ("accent-foreground".into(), "oklch(0.95 0.01 250)".into()),
            ("border".into(), "oklch(0.30 0.02 250)".into()),
            ("input".into(), "oklch(0.30 0.02 250)".into()),
            ("ring".into(), "oklch(0.55 0.15 250)".into()),
            ("card".into(), "oklch(0.18 0.02 250)".into()),
        ]));
        let palette = ShadcnPalette::from_catalog(&catalog, ThemeMode::Dark).expect("palette");
        let base = palette.primary.background;
        let hover = resolve_state_color(
            &catalog,
            &palette,
            ShadcnToken::Primary,
            InteractionLayer::Hovered,
            base,
            ThemeMode::Dark,
            None,
        );
        assert!(hover.l > base.l, "dark hover should lighten");
    }

    #[test]
    fn algorithmic_light_and_dark_move_in_opposite_directions() {
        let base = gpui::hsla(0.6, 0.5, 0.5, 1.0);
        let light_hover = algorithmic_state_color(base, InteractionLayer::Hovered, ThemeMode::Light, true);
        let dark_hover = algorithmic_state_color(base, InteractionLayer::Hovered, ThemeMode::Dark, true);
        assert!(light_hover.l < base.l);
        assert!(dark_hover.l > base.l);
    }
}
