//! Context menu property mappings:
//!
//! | Part   | Token                              |
//! |--------|------------------------------------|
//! | Target | ghost (accent-foreground on hover) |
//! | Menu   | floating menu surface              |

use gpui_luma::controls::context_menu::ContextMenuLook;
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use super::floating_menu::floating_menu_look;
use crate::provenance::ResolvedColor;
use crate::mode::ShadcnModeTokens;

/// Effective context-menu trigger colors, including per-field missing-token fallbacks.
#[derive(Clone, Debug)]
pub struct ContextMenuColorTable {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
}

pub fn resolve_context_menu_colors(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> ContextMenuColorTable {
    let resolver = crate::LookResolver::new(&mode.catalog, theme_mode, "context_menu");
    let background = if state.disabled {
        resolver.resolve_decl("muted").unwrap_or_else(|_| ResolvedColor::transparent())
    } else {
        ResolvedColor::transparent()
    };
    let foreground = if state.disabled {
        resolver.resolve_decl("muted-foreground")
    } else if state.hovered || state.pressed {
        resolver.resolve_first_decl(&["accent-foreground", "foreground"])
    } else {
        resolver.resolve_decl("foreground")
    }
    .unwrap_or_else(|_| ResolvedColor::fallback_foreground());
    let border = resolver.resolve_decl("border").unwrap_or_else(|_| ResolvedColor::fallback_foreground());
    ContextMenuColorTable { background, foreground, border }
}

pub fn context_menu_look(mode: &ShadcnModeTokens, theme_mode: ThemeMode, state: InteractionState) -> ContextMenuLook {
    let ctx = LookContext::new(mode, theme_mode, state);
    let state = ctx.state;
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let size = ControlSize::Md;
    let colors = resolve_context_menu_colors(mode, theme_mode, state);

    ContextMenuLook {
        target_background: colors.background.hsla(),
        target_foreground: colors.foreground.hsla(),
        target_border: colors.border.hsla(),
        target_typography: typography.text.label,
        target_radius: metrics.radius(size),
        target_padding_x: metrics.padding_x(size),
        target_padding_y: metrics.padding_y(size),
        target_min_width: 200.0,
        floating_menu: floating_menu_look(ctx.tokens, ctx.theme_mode, size),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui_luma::theme::{InteractionState, ThemeMode};

    use super::context_menu_look;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::provenance::ResolvedColor;

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
        ]))
    }

    #[test]
    fn incomplete_catalog_does_not_panic_on_border_or_disabled_ghost() {
        let mut mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        mode.catalog.tokens.remove("border");
        mode.catalog.tokens.remove("muted");
        mode.catalog.tokens.remove("muted-foreground");
        let fallback = ResolvedColor::fallback_foreground().hsla();

        let look = context_menu_look(&mode, ThemeMode::Light, InteractionState::default());
        let disabled = context_menu_look(
            &mode,
            ThemeMode::Light,
            InteractionState { disabled: true, ..InteractionState::default() },
        );

        assert_eq!(look.target_border, fallback);
        assert_eq!(disabled.target_background, gpui::hsla(0.0, 0.0, 0.0, 0.0));
        assert_eq!(disabled.target_foreground, fallback);
    }
}
