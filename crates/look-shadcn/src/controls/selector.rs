//! Selector property mappings — input trigger fill + accent item panel.

use gpui_luma::controls::selector::{SelectorLook, SelectorPalette};
use gpui_luma::controls::textfield::TextFieldState;
use gpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale, ThemeMode};

use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{embedded_stylesheet, resolve_button_metrics_rule};

use super::selector_items_panel::selector_items_panel_look;
use super::textfield::{ShadcnTextFieldStyle, textfield_palette};

fn selector_textfield_state(state: InteractionState) -> TextFieldState {
    TextFieldState {
        hovered: state.hovered,
        focused: state.focused,
        focus_visible: state.focused,
        ..TextFieldState::default()
    }
}

pub fn selector_palette(mode: &ShadcnModeTokens, theme_mode: ThemeMode, state: InteractionState) -> SelectorPalette {
    let trigger = textfield_palette(
        mode,
        theme_mode,
        ShadcnTextFieldStyle::Input,
        selector_textfield_state(state),
        !state.disabled,
    );
    let typography = &mode.typography;

    SelectorPalette {
        trigger_background: trigger.background,
        trigger_foreground: trigger.foreground,
        trigger_icon: trigger.icon,
        trigger_border: trigger.border,
        focus_ring: trigger.focus_ring,
        trigger_typography: typography.text.label,
        items_panel: selector_items_panel_look(mode, theme_mode, ControlSize::Md),
    }
}

pub fn selector_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
    size: ControlSize,
    scale: &StandardBoxScale,
) -> SelectorLook {
    let palette = selector_palette(mode, theme_mode, state);
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
        focus_ring: palette.focus_ring,
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

    use gpui_luma::theme::{ControlSize, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::color::with_alpha;
    use crate::mode::ShadcnModeTokens;
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
        ]))
    }

    #[test]
    fn selector_trigger_light_uses_transparent_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let palette = selector_palette(&mode, ThemeMode::Light, gpui_luma::theme::InteractionState::default());

        assert_eq!(palette.trigger_background, gpui::hsla(0.0, 0.0, 0.0, 0.0));
        assert_eq!(palette.trigger_foreground, catalog.color("foreground").expect("foreground"));
        assert_eq!(palette.trigger_border, catalog.color("border").expect("border"));
    }

    #[test]
    fn selector_trigger_dark_uses_input_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let palette = selector_palette(&mode, ThemeMode::Dark, gpui_luma::theme::InteractionState::default());
        let input = catalog.color("input").expect("input");

        assert_eq!(palette.trigger_background, with_alpha(input, 0.30));
        assert_eq!(palette.trigger_border, catalog.color("border").expect("border"));
    }

    #[test]
    fn selector_trigger_dark_hover_uses_input_half_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let default = selector_palette(&mode, ThemeMode::Dark, gpui_luma::theme::InteractionState::default());
        let hovered = selector_palette(
            &mode,
            ThemeMode::Dark,
            gpui_luma::theme::InteractionState { hovered: true, ..Default::default() },
        );
        let input = catalog.color("input").expect("input");

        assert_eq!(default.trigger_background, with_alpha(input, 0.30));
        assert_eq!(hovered.trigger_background, with_alpha(input, 0.50));
        assert_eq!(hovered.trigger_foreground, default.trigger_foreground);
    }

    #[test]
    fn selector_trigger_chevron_uses_muted_icon_color() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let palette = selector_palette(&mode, ThemeMode::Dark, gpui_luma::theme::InteractionState::default());

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
            gpui_luma::theme::InteractionState::default(),
            ControlSize::Sm,
            &sm_scale,
        );
        let lg = selector_look(
            &mode,
            ThemeMode::Light,
            gpui_luma::theme::InteractionState::default(),
            ControlSize::Lg,
            &lg_scale,
        );

        assert_eq!(sm.trigger_typography.size, 12.0);
        assert_eq!(lg.trigger_typography.size, 16.0);
    }
}
