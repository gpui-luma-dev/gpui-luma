//! Popup menu property mappings:
//!
//! | Part    | Token                              |
//! |---------|------------------------------------|
//! | Trigger | ghost (accent-foreground on hover) |
//! | Menu    | floating menu surface              |

use gpui_luma::controls::popup_menu::PopupMenuPalette;
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use super::floating_menu::floating_menu_look;
use crate::focus::focus_ring_color;
use crate::resolve::{resolve_color, resolve_ghost_trigger_background, resolve_ghost_trigger_foreground};
use crate::mode::ShadcnModeTokens;

pub fn popup_menu_palette(mode: &ShadcnModeTokens, theme_mode: ThemeMode, state: InteractionState) -> PopupMenuPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let state = ctx.state;
    let catalog = ctx.catalog();
    let typography = ctx.typography();
    let layer = state.layer();

    PopupMenuPalette {
        trigger_background: resolve_ghost_trigger_background(catalog, layer)
            .unwrap_or_else(|err| panic!("popup menu properties: {err}")),
        trigger_foreground: resolve_ghost_trigger_foreground(catalog, layer, state.disabled)
            .unwrap_or_else(|err| panic!("popup menu properties: {err}")),
        trigger_border: resolve_color(catalog, "border").unwrap_or_else(|err| panic!("popup menu properties: {err}")),
        focus_ring: state
            .focused
            .then(|| focus_ring_color(catalog))
            .transpose()
            .unwrap_or_else(|err| panic!("popup menu properties: {err}")),
        trigger_typography: typography.text.label,
        floating_menu: floating_menu_look(ctx.tokens, ctx.theme_mode, ControlSize::Md),
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use gpui_luma::theme::ThemeMode;

    use gpui_luma::theme::InteractionState;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::popup_menu_palette;

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
    fn popup_menu_hovered_trigger_uses_accent_foreground_without_background_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let default = popup_menu_palette(&mode, gpui_luma::theme::ThemeMode::Light, InteractionState::default());
        let hovered = popup_menu_palette(
            &mode,
            gpui_luma::theme::ThemeMode::Light,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert_eq!(hovered.trigger_background, default.trigger_background);
        assert_eq!(hovered.trigger_foreground, catalog.color("accent-foreground").expect("accent-foreground"));
        assert_eq!(hovered.floating_menu.item_hover_background, catalog.color("accent").expect("accent"));
    }
}
