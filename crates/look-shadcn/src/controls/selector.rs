//! Selector property mappings — outline/ghost command-button trigger + accent item panel.

use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::controls::selector::{SelectorLook, SelectorPalette, SelectorTriggerStyle};
use gpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{embedded_stylesheet, resolve_button_metrics_rule};

use super::button::{ShadcnButtonStyle, button_elevation_shadow, button_palette};
use super::selector_items_panel::selector_items_panel_look;

fn shadcn_button_style(trigger_style: SelectorTriggerStyle) -> ShadcnButtonStyle {
    match trigger_style {
        SelectorTriggerStyle::Outline => ShadcnButtonStyle::Outline,
        SelectorTriggerStyle::Ghost => ShadcnButtonStyle::Ghost,
    }
}

pub fn selector_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: SelectorTriggerStyle,
    state: InteractionState,
    without_elevation: bool,
) -> SelectorPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let stylesheet = embedded_stylesheet();
    let button_style = shadcn_button_style(trigger_style);
    let button = button_palette(&ctx, stylesheet, button_style, ButtonFamilyRole::Text, ControlSize::Md);
    let trigger_shadow = if without_elevation {
        None
    } else {
        button_elevation_shadow(&ctx, stylesheet, button_style)
    };
    let icon = ctx.catalog().color("muted-foreground").unwrap_or(button.foreground);

    SelectorPalette {
        trigger_background: button.background,
        trigger_foreground: button.foreground,
        trigger_icon: icon,
        trigger_border: button.border,
        trigger_shadow,
        adorner: button.adorner,
        trigger_typography: button.typography,
        items_panel: selector_items_panel_look(mode, theme_mode, ControlSize::Md),
    }
}

pub fn selector_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: SelectorTriggerStyle,
    state: InteractionState,
    size: ControlSize,
    scale: &StandardBoxScale,
    without_elevation: bool,
) -> SelectorLook {
    let palette = selector_palette(mode, theme_mode, trigger_style, state, without_elevation);
    let mut typography = palette.trigger_typography;
    if let Some(rule) = embedded_stylesheet().button.metrics_for_size(size) {
        let metrics = resolve_button_metrics_rule(rule, &mode.metrics, size);
        let base_size = typography.size;
        typography.size = metrics.font_size;
        if base_size > 0.0 {
            typography.line_height = metrics.font_size * (typography.line_height / base_size);
        }
    }

    SelectorLook {
        trigger_background: palette.trigger_background,
        trigger_foreground: palette.trigger_foreground,
        trigger_icon: palette.trigger_icon,
        trigger_border: palette.trigger_border,
        trigger_shadow: palette.trigger_shadow,
        adorner: palette.adorner,
        trigger_typography: typography,
        trigger_radius: scale.radius,
        trigger_padding_x: scale.padding_x,
        trigger_padding_y: scale.padding_y,
        trigger_gap: scale.gap,
        trigger_height: scale.height,
        trigger_icon_size: scale.height / 3.0,
        menu_offset_y: scale.gap * 0.5,
        items_panel: selector_items_panel_look(mode, theme_mode, size),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui_luma::controls::button_family::ButtonFamilyRole;
    use gpui_luma::controls::selector::SelectorTriggerStyle;
    use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::controls::button::{ShadcnButtonStyle, button_palette};
    use crate::look_context::LookContext;
    use crate::mode::ShadcnModeTokens;
    use crate::stylesheet::embedded_stylesheet;
    use super::{selector_look, selector_palette};

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
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.7200 0.0120 205.0000)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("shadow-xs".into(), "0 1px 2px 0px hsl(0 0% 0% / 0.05)".into()),
        ]))
    }

    #[test]
    fn outline_trigger_matches_outline_command_button() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let state = InteractionState::default();
        let selector = selector_palette(&mode, ThemeMode::Light, SelectorTriggerStyle::Outline, state, false);
        let ctx = LookContext::new(&mode, ThemeMode::Light, state);
        let button = button_palette(
            &ctx,
            embedded_stylesheet(),
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            ControlSize::Md,
        );

        assert_eq!(selector.trigger_background, button.background);
        assert_eq!(selector.trigger_foreground, button.foreground);
        assert_eq!(selector.trigger_border, button.border);
        assert!(selector.trigger_shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn ghost_trigger_matches_ghost_command_button() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let state = InteractionState::default();
        let selector = selector_palette(&mode, ThemeMode::Light, SelectorTriggerStyle::Ghost, state, false);
        let ctx = LookContext::new(&mode, ThemeMode::Light, state);
        let button = button_palette(
            &ctx,
            embedded_stylesheet(),
            ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            ControlSize::Md,
        );

        assert_eq!(selector.trigger_background, button.background);
        assert_eq!(selector.trigger_foreground, button.foreground);
        assert_eq!(selector.trigger_border, button.border);
        assert!(selector.trigger_shadow.is_none());
    }

    #[test]
    fn outline_without_elevation_clears_shadow() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let selector =
            selector_palette(&mode, ThemeMode::Light, SelectorTriggerStyle::Outline, InteractionState::default(), true);
        assert!(selector.trigger_shadow.is_none());
    }

    #[test]
    fn selector_trigger_chevron_uses_muted_icon_color() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let palette =
            selector_palette(&mode, ThemeMode::Dark, SelectorTriggerStyle::Outline, InteractionState::default(), false);

        assert_eq!(palette.trigger_icon, catalog.color("muted-foreground").expect("muted-foreground"));
    }

    #[test]
    fn selector_trigger_typography_uses_stylesheet_font_size() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let sm_scale = gpui_luma::theme::StandardBoxScale::compute(ControlSize::Sm, &mode.metrics, 1.0);
        let lg_scale = gpui_luma::theme::StandardBoxScale::compute(ControlSize::Lg, &mode.metrics, 1.0);

        let sm = selector_look(
            &mode,
            ThemeMode::Light,
            SelectorTriggerStyle::Outline,
            InteractionState::default(),
            ControlSize::Sm,
            &sm_scale,
            false,
        );
        let lg = selector_look(
            &mode,
            ThemeMode::Light,
            SelectorTriggerStyle::Outline,
            InteractionState::default(),
            ControlSize::Lg,
            &lg_scale,
            false,
        );

        assert_eq!(sm.trigger_typography.size, 12.0);
        assert_eq!(lg.trigger_typography.size, 16.0);
    }
}
