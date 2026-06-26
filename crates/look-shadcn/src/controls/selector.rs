//! Selector property mappings — input trigger fill + accent item panel.

use gpui_luma::controls::selector::SelectorPalette;
use gpui_luma::controls::textfield::TextFieldState;
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use crate::mode::ShadcnModeTokens;

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
        trigger_border: trigger.border,
        focus_ring: trigger.focus_ring,
        trigger_typography: typography.text.label,
        items_panel: selector_items_panel_look(mode, theme_mode, ControlSize::Md),
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use gpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::selector_palette;

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
    fn selector_trigger_uses_background_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let palette = selector_palette(&mode, ThemeMode::Light, gpui_luma::theme::InteractionState::default());

        assert_eq!(palette.trigger_background, catalog.color("background").expect("background"));
        assert_eq!(palette.trigger_foreground, catalog.color("foreground").expect("foreground"));
        assert_eq!(palette.trigger_border, catalog.color("border").expect("border"));
    }

    #[test]
    fn selector_trigger_hover_keeps_background_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let default = selector_palette(&mode, ThemeMode::Light, gpui_luma::theme::InteractionState::default());
        let hovered = selector_palette(
            &mode,
            ThemeMode::Light,
            gpui_luma::theme::InteractionState { hovered: true, ..Default::default() },
        );

        assert_eq!(hovered.trigger_background, default.trigger_background);
        assert_eq!(hovered.trigger_foreground, default.trigger_foreground);
    }
}
