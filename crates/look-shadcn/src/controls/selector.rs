//! Selector property mappings — ghost trigger (foreground-only hover) + accent item panel.

use gpui_luma::controls::selector::SelectorPalette;
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::LookResolver;

use super::floating_menu::{resolve_ghost_trigger_colors};
use super::selector_items_panel::selector_items_panel_look;

pub fn selector_palette(mode: &ShadcnModeTokens, theme_mode: ThemeMode, state: InteractionState) -> SelectorPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let state = ctx.state;
    let typography = ctx.typography();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "selector_trigger");
    let trigger_colors = resolve_ghost_trigger_colors(&resolver, state.layer(), state.disabled)
        .unwrap_or_else(|_| super::floating_menu::GhostTriggerColorTable::fallback());

    SelectorPalette {
        trigger_background: trigger_colors.background.hsla(),
        trigger_foreground: trigger_colors.foreground.hsla(),
        trigger_border: resolver
            .resolve_decl("border")
            .unwrap_or_else(|err| panic!("selector properties: {err}"))
            .hsla(),
        focus_ring: state
            .focused
            .then(|| resolver.resolve_decl("ring"))
            .transpose()
            .unwrap_or_else(|err| panic!("selector properties: {err}"))
            .map(|color| color.hsla()),
        trigger_typography: typography.text.label,
        items_panel: selector_items_panel_look(ctx.tokens, ctx.theme_mode, ControlSize::Md),
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
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
        ]))
    }

    #[test]
    fn selector_catalog_uses_ghost_trigger_table() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let palette = selector_palette(
            &mode,
            ThemeMode::Light,
            gpui_luma::theme::InteractionState { hovered: true, ..Default::default() },
        );
        assert_eq!(palette.trigger_foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }
}
