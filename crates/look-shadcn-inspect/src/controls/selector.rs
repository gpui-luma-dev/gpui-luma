//! Inspect metadata for `selector`.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{AppearanceContext, ColorSource, LookResolver, ResolvedColor, ShadcnModeTokens};

use super::floating_menu::{FloatingMenuInspectMetrics, FloatingMenuInspectPalette};

pub struct SelectorInspectPalette {
    pub trigger_background: ResolvedColor,
    pub trigger_foreground: ResolvedColor,
    pub trigger_border: ResolvedColor,
    pub focus_ring: Option<ResolvedColor>,
    pub items_panel: FloatingMenuInspectPalette,
}

#[derive(Clone, Debug)]
pub struct SelectorInspectMetrics {
    pub trigger: crate::controls::button::ButtonInspectMetrics,
    pub items_panel: FloatingMenuInspectMetrics,
}

pub fn inspect_selector_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
    size: ControlSize,
) -> SelectorInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    let menu = crate::controls::floating_menu::inspect_floating_menu_color_palette(mode, theme_mode, size);

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "selector_inspect");
    let trigger_colors =
        gpui_luma_look_shadcn::tables::resolve_ghost_trigger_colors(&resolver, state.layer(), state.disabled)
            .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::GhostTriggerColorTable::fallback());
    let trigger_border = resolver.resolve_decl("border").unwrap_or_else(|_| ResolvedColor {
        value: ctx.catalog().color("border").expect("border"),
        source: ColorSource::CssVar { token: "border".into() },
    });
    let focus_ring = state.focused.then(|| resolver.resolve_decl("ring")).transpose().ok().flatten();

    SelectorInspectPalette {
        trigger_background: trigger_colors.background,
        trigger_foreground: trigger_colors.foreground,
        trigger_border,
        focus_ring,
        items_panel: menu,
    }
}

pub fn inspect_selector_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> SelectorInspectMetrics {
    use gpui_luma::controls::button_family::ButtonFamilyRole;

    SelectorInspectMetrics {
        trigger: crate::controls::button::inspect_button_metrics(
            mode,
            theme_mode,
            gpui_luma_look_shadcn::ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            size,
            InteractionState::default(),
        ),
        items_panel: crate::controls::floating_menu::inspect_floating_menu_metrics(mode, theme_mode, size),
    }
}
