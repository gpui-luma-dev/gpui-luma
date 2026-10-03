//! Popup menu property mappings:
//!
//! | Part    | Variant                          |
//! |---------|----------------------------------|
//! | Trigger | outline or ghost command button  |
//! | Menu    | floating menu surface            |

use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::controls::popup_menu::{PopupMenuLook, PopupMenuPalette, PopupMenuTriggerStyle, compose_popup_menu_look};
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use super::button::{ShadcnButtonStyle, button_box_scale, button_elevation_shadow, button_palette};
use super::floating_menu::floating_menu_look;
use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{embedded_stylesheet, resolve_button_metrics_rule};

fn shadcn_button_style(trigger_style: PopupMenuTriggerStyle) -> ShadcnButtonStyle {
    match trigger_style {
        PopupMenuTriggerStyle::Primary => ShadcnButtonStyle::Primary,
        PopupMenuTriggerStyle::Secondary => ShadcnButtonStyle::Secondary,
        PopupMenuTriggerStyle::Outline => ShadcnButtonStyle::Outline,
        PopupMenuTriggerStyle::Ghost => ShadcnButtonStyle::Ghost,
    }
}

pub fn popup_menu_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: PopupMenuTriggerStyle,
    metrics: gpui_luma::controls::popup_menu::PopupMenuTriggerMetrics,
    state: InteractionState,
) -> PopupMenuPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let stylesheet = embedded_stylesheet();
    let button_style = shadcn_button_style(trigger_style);
    let role = if metrics.icon_only {
        ButtonFamilyRole::Icon
    } else {
        ButtonFamilyRole::Text
    };
    let button = button_palette(&ctx, stylesheet, button_style, role, metrics.size);
    let trigger_shadow = if metrics.without_elevation {
        None
    } else {
        button_elevation_shadow(&ctx, stylesheet, button_style)
    };

    PopupMenuPalette {
        trigger_background: button.background,
        trigger_foreground: button.foreground,
        trigger_border: button.border,
        trigger_shadow,
        trigger_typography: button.typography,
        floating_menu: floating_menu_look(mode, theme_mode, metrics.menu_size),
    }
}

pub fn popup_menu_trigger_scale(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
    state: InteractionState,
    scale_factor: f32,
) -> gpui_luma::theme::StandardBoxScale {
    let ctx = LookContext::new(mode, theme_mode, state);
    button_box_scale(&ctx, embedded_stylesheet(), size, scale_factor)
}

pub fn popup_menu_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: PopupMenuTriggerStyle,
    metrics: gpui_luma::controls::popup_menu::PopupMenuTriggerMetrics,
    state: InteractionState,
    scale_factor: f32,
    _cx: &mut gpui::App,
) -> PopupMenuLook {
    let scale = popup_menu_trigger_scale(mode, theme_mode, metrics.size, state, scale_factor);
    let mut look =
        compose_popup_menu_look(&popup_menu_palette(mode, theme_mode, trigger_style, metrics, state), &scale);
    if let Some(rule) = embedded_stylesheet().button.metrics_for_size(metrics.size) {
        look.trigger_icon_size = resolve_button_metrics_rule(rule, &mode.metrics, metrics.size).icon_size;
    }
    if let Some(radius) = metrics.trigger_radius_override {
        look.trigger_radius = radius;
    }
    look
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use gpui_luma::theme::ThemeMode;

    use gpui_luma::controls::popup_menu::{PopupMenuTriggerMetrics, PopupMenuTriggerStyle};
    use gpui_luma::theme::InteractionState;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::controls::button::{ShadcnButtonStyle, button_palette};
    use super::{popup_menu_palette, shadcn_button_style};
    use crate::controls::floating_menu::floating_menu_look;
    use gpui_luma::controls::button_family::ButtonFamilyRole;
    use gpui_luma::theme::ControlSize;
    use crate::look_context::LookContext;
    use crate::stylesheet::embedded_stylesheet;

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
            ("shadow-xs".into(), "0 1px 2px 0px hsl(0 0% 0% / 0.05)".into()),
        ]))
    }

    #[test]
    fn outline_trigger_matches_outline_command_button() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let state = InteractionState::default();
        let metrics = PopupMenuTriggerMetrics::default();
        let popup = popup_menu_palette(&mode, ThemeMode::Light, PopupMenuTriggerStyle::Outline, metrics, state);
        let ctx = LookContext::new(&mode, ThemeMode::Light, state);
        let button = button_palette(
            &ctx,
            embedded_stylesheet(),
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            ControlSize::Md,
        );

        assert_eq!(popup.trigger_background, button.background);
        assert_eq!(popup.trigger_foreground, button.foreground);
        assert_eq!(popup.trigger_border, button.border);
        assert!(popup.trigger_shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn ghost_trigger_matches_ghost_command_button() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let state = InteractionState::default();
        let metrics = PopupMenuTriggerMetrics::default();
        let popup = popup_menu_palette(&mode, ThemeMode::Light, PopupMenuTriggerStyle::Ghost, metrics, state);
        let ctx = LookContext::new(&mode, ThemeMode::Light, state);
        let button = button_palette(
            &ctx,
            embedded_stylesheet(),
            ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            ControlSize::Md,
        );

        assert_eq!(popup.trigger_background, button.background);
        assert_eq!(popup.trigger_foreground, button.foreground);
        assert_eq!(popup.trigger_border, button.border);
        assert!(popup.trigger_shadow.is_none());
    }

    #[test]
    fn ghost_hovered_trigger_uses_accent_fill_like_command_button() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let metrics = PopupMenuTriggerMetrics::default();
        let hovered = InteractionState { hovered: true, ..InteractionState::default() };
        let popup = popup_menu_palette(&mode, ThemeMode::Light, PopupMenuTriggerStyle::Ghost, metrics, hovered);
        let ctx = LookContext::new(&mode, ThemeMode::Light, hovered);
        let button = button_palette(
            &ctx,
            embedded_stylesheet(),
            shadcn_button_style(PopupMenuTriggerStyle::Ghost),
            ButtonFamilyRole::Text,
            ControlSize::Md,
        );

        assert_eq!(popup.trigger_background, button.background);
        assert_eq!(popup.trigger_foreground, button.foreground);
        assert_eq!(popup.trigger_background, catalog.color("accent").expect("accent"));
    }

    #[test]
    fn menu_size_is_independent_from_trigger_size() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let metrics = PopupMenuTriggerMetrics {
            size: ControlSize::Sm,
            menu_size: ControlSize::Lg,
            ..PopupMenuTriggerMetrics::default()
        };
        let state = InteractionState::default();
        let popup = popup_menu_palette(&mode, ThemeMode::Light, PopupMenuTriggerStyle::Outline, metrics, state);
        let ctx = LookContext::new(&mode, ThemeMode::Light, state);
        let small_trigger = button_palette(
            &ctx,
            embedded_stylesheet(),
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            ControlSize::Sm,
        );
        let large_menu = floating_menu_look(&mode, ThemeMode::Light, ControlSize::Lg);

        assert_eq!(popup.trigger_typography.size, small_trigger.typography.size);
        assert_eq!(popup.floating_menu.item_height, large_menu.item_height);
        assert_eq!(popup.floating_menu.item_typography.size, large_menu.item_typography.size);
    }
}
