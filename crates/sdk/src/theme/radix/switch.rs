//! Switch property mappings (Radix Themes surface / shadcn):
//!
//! | State    | Track token | Thumb token                    |
//! |----------|-------------|--------------------------------|
//! | Off      | `input`     | `background` (fallback `card`) |
//! | On       | `{style}`   | `{style}-foreground`           |
//! | Disabled | `muted`     | `muted-foreground`             |

use crate::controls::switch::SwitchAppearance;
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTheme, ThemeMode};

use super::focus::focus_adorner;
use super::resolve::{
    resolve_action_foreground, resolve_action_layer, resolve_color, resolve_color_layer, resolve_label_color,
};
use super::RadixButtonStyle;
use super::catalog::CssTokenMap;
use super::mode::RadixModeTokens;
use super::palette::RadixPalette;

pub(crate) fn switch_appearance(
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    style: RadixButtonStyle,
    on: bool,
    state: InteractionState,
) -> SwitchAppearance {
    if mode.catalog.tokens.is_empty() {
        return switch_appearance_from_palette(
            &mode.palette,
            &mode.metrics,
            &mode.typography,
            theme_mode,
            style,
            on,
            state,
        );
    }

    switch_appearance_from_catalog(&mode.catalog, &mode.metrics, &mode.typography, theme_mode, style, on, state)
        .unwrap_or_else(|err| panic!("switch properties: {err}"))
}

fn switch_appearance_from_palette(
    palette: &RadixPalette,
    metrics: &crate::theme::MetricTokens,
    typography: &crate::theme::LumaTypography,
    theme_mode: ThemeMode,
    style: RadixButtonStyle,
    on: bool,
    state: InteractionState,
) -> SwitchAppearance {
    let layer = state.layer();
    let size = ControlSize::Md;
    let thumb_shadow = {
        let native = LumaTheme::native();
        native.mode(theme_mode).elevation.thumb.to_box_shadows()
    };
    let on_action = palette.action(style);

    let track_background = match (on, layer) {
        (_, InteractionLayer::Disabled) => palette.disabled_background,
        (true, InteractionLayer::Pressed) => on_action.pressed_background,
        (true, InteractionLayer::Hovered) => on_action.hover_background,
        (true, InteractionLayer::Default) => on_action.background,
        (false, InteractionLayer::Pressed) => palette.muted_background,
        (false, InteractionLayer::Hovered) => palette.muted_background,
        (false, InteractionLayer::Default) => palette.input_background,
    };

    let (thumb_background, thumb_border) = if state.disabled {
        (palette.disabled_foreground, palette.disabled_background)
    } else if on {
        (on_action.foreground, on_action.foreground)
    } else {
        (palette.panel_background, palette.border_default)
    };

    SwitchAppearance {
        track_background,
        track_border: if on && !state.disabled {
            track_background
        } else {
            palette.border_default
        },
        thumb_background,
        thumb_border,
        thumb_shadow,
        label_color: if state.disabled {
            palette.disabled_foreground
        } else {
            palette.app_foreground
        },
        adorner: super::focus::focus_adorner_from_palette(palette, metrics, state.focused),
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
        width: 42.0,
        height: 22.0,
        thumb_size: metrics.control_height(size) * 0.5,
        padding: 2.0,
        gap: metrics.gap(size),
        radius: metrics.radius.pill,
    }
}

pub(crate) fn switch_appearance_from_catalog(
    catalog: &CssTokenMap,
    metrics: &crate::theme::MetricTokens,
    typography: &crate::theme::LumaTypography,
    theme_mode: ThemeMode,
    style: RadixButtonStyle,
    on: bool,
    state: InteractionState,
) -> anyhow::Result<SwitchAppearance> {
    let layer = state.layer();
    let size = ControlSize::Md;
    let thumb_shadow = {
        let native = LumaTheme::native();
        native.mode(theme_mode).elevation.thumb.to_box_shadows()
    };

    let track_background = match (on, layer) {
        (_, InteractionLayer::Disabled) => resolve_color(catalog, "muted")?,
        (true, _) => resolve_action_layer(catalog, style, layer)?,
        (false, _) => resolve_color_layer(catalog, "input", layer, false)?,
    };

    let track_border = if on && !state.disabled {
        track_background
    } else {
        resolve_color(catalog, "border")?
    };

    let (thumb_background, thumb_border) = switch_thumb_colors(catalog, style, on, state.disabled)?;

    Ok(SwitchAppearance {
        track_background,
        track_border,
        thumb_background,
        thumb_border,
        thumb_shadow,
        label_color: resolve_label_color(catalog, state.disabled)?,
        adorner: focus_adorner(catalog, metrics, state.focused)?,
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
        width: 42.0,
        height: 22.0,
        thumb_size: metrics.control_height(size) * 0.5,
        padding: 2.0,
        gap: metrics.gap(size),
        radius: metrics.radius.pill,
    })
}

fn switch_thumb_colors(
    catalog: &CssTokenMap,
    style: RadixButtonStyle,
    on: bool,
    disabled: bool,
) -> anyhow::Result<(gpui::Hsla, gpui::Hsla)> {
    if disabled {
        let thumb = resolve_color(catalog, "muted-foreground")?;
        let border = resolve_color(catalog, "muted")?;
        return Ok((thumb, border));
    }

    if on {
        let thumb = resolve_action_foreground(catalog, style)?;
        return Ok((thumb, thumb));
    }

    let thumb = catalog.color_first(&["background", "card"])?;
    let border = resolve_color(catalog, "border")?;
    Ok((thumb, border))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::theme::{InteractionState, ThemeMode};

    use super::super::RadixButtonStyle;
    use super::super::catalog::CssTokenMap;
    use super::super::mode::RadixModeTokens;
    use super::switch_appearance_from_catalog;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.1 0 0)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.8 0.02 200)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    #[test]
    fn off_switch_uses_input_track_and_background_thumb() {
        let catalog = sample_catalog();
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let appearance = switch_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            &mode.typography,
            ThemeMode::Light,
            RadixButtonStyle::Primary,
            false,
            InteractionState::default(),
        )
        .expect("switch");

        let input = catalog.color("input").expect("input");
        let background = catalog.color("background").expect("background");
        let border = catalog.color("border").expect("border");
        assert_eq!(appearance.track_background, input);
        assert_eq!(appearance.track_border, border);
        assert_eq!(appearance.thumb_background, background);
        assert_eq!(appearance.thumb_border, border);
    }

    #[test]
    fn primary_and_secondary_share_off_appearance() {
        let catalog = sample_catalog();
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let primary = switch_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            &mode.typography,
            ThemeMode::Light,
            RadixButtonStyle::Primary,
            false,
            InteractionState::default(),
        )
        .expect("primary");
        let secondary = switch_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            &mode.typography,
            ThemeMode::Light,
            RadixButtonStyle::Secondary,
            false,
            InteractionState::default(),
        )
        .expect("secondary");

        assert_eq!(primary.track_background, secondary.track_background);
        assert_eq!(primary.thumb_background, secondary.thumb_background);
    }
}
