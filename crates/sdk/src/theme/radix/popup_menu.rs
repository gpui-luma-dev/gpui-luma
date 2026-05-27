//! Popup menu property mappings:
//!
//! | Part    | Token                    |
//! |---------|--------------------------|
//! | Trigger | ghost (`accent` on hover) |
//! | Menu    | floating menu surface    |

use crate::controls::popup_menu::PopupMenuAppearance;
use crate::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use super::floating_menu::floating_menu_appearance;
use super::focus::focus_ring_color;
use super::resolve::{resolve_color, resolve_ghost_background, resolve_label_color};
use super::catalog::CssTokenMap;
use super::mode::RadixModeTokens;
use super::palette::RadixPalette;

pub(crate) fn popup_menu_appearance(
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> PopupMenuAppearance {
    if mode.catalog.tokens.is_empty() {
        popup_menu_appearance_from_palette(&mode.palette, mode, theme_mode, state)
    } else {
        popup_menu_appearance_from_catalog(&mode.catalog, mode, theme_mode, state)
            .unwrap_or_else(|err| panic!("popup menu properties: {err}"))
    }
}

fn popup_menu_appearance_from_palette(
    palette: &RadixPalette,
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> PopupMenuAppearance {
    let metrics = &mode.metrics;
    let typography = &mode.typography;
    let size = ControlSize::Md;
    let layer = state.layer();
    let ghost = palette.ghost;

    let trigger_background = match layer {
        InteractionLayer::Disabled => palette.disabled_background,
        InteractionLayer::Pressed => ghost.pressed_background,
        InteractionLayer::Hovered => ghost.hover_background,
        InteractionLayer::Default => ghost.background,
    };

    PopupMenuAppearance {
        trigger_background,
        trigger_foreground: if state.disabled {
            palette.disabled_foreground
        } else {
            ghost.foreground
        },
        trigger_border: palette.border_default,
        focus_ring: state.focused.then_some(palette.focus_ring),
        trigger_typography: typography.text.label,
        trigger_radius: metrics.radius(size),
        trigger_padding_x: metrics.padding_x(size),
        trigger_padding_y: metrics.padding_y(size),
        trigger_gap: metrics.gap(size),
        trigger_height: metrics.control_height(size),
        trigger_icon_size: metrics.control_height(size) * 0.44,
        menu_offset_y: metrics.gap(size) * 0.5,
        floating_menu: floating_menu_appearance(mode, theme_mode, size),
    }
}

pub(crate) fn popup_menu_appearance_from_catalog(
    catalog: &CssTokenMap,
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> anyhow::Result<PopupMenuAppearance> {
    let metrics = &mode.metrics;
    let typography = &mode.typography;
    let size = ControlSize::Md;
    let layer = state.layer();

    let trigger_background = resolve_ghost_background(catalog, layer)?;
    let trigger_foreground = resolve_label_color(catalog, state.disabled)?;

    Ok(PopupMenuAppearance {
        trigger_background,
        trigger_foreground,
        trigger_border: resolve_color(catalog, "border")?,
        focus_ring: state.focused.then(|| focus_ring_color(catalog)).transpose()?,
        trigger_typography: typography.text.label,
        trigger_radius: metrics.radius(size),
        trigger_padding_x: metrics.padding_x(size),
        trigger_padding_y: metrics.padding_y(size),
        trigger_gap: metrics.gap(size),
        trigger_height: metrics.control_height(size),
        trigger_icon_size: metrics.control_height(size) * 0.44,
        menu_offset_y: metrics.gap(size) * 0.5,
        floating_menu: floating_menu_appearance(mode, theme_mode, size),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::theme::InteractionState;

    use super::super::catalog::CssTokenMap;
    use super::super::mode::RadixModeTokens;
    use super::popup_menu_appearance_from_catalog;

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
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
        ]))
    }

    #[test]
    fn popup_menu_hovered_trigger_uses_accent_background() {
        let catalog = sample_catalog();
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let appearance = popup_menu_appearance_from_catalog(
            &catalog,
            &mode,
            crate::theme::ThemeMode::Light,
            InteractionState { hovered: true, ..InteractionState::default() },
        )
        .expect("popup menu");

        assert_eq!(appearance.trigger_background, catalog.color("accent").expect("accent"));
        assert_eq!(appearance.floating_menu.background, catalog.color("popover").expect("popover"));
    }
}
