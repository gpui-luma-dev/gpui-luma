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
    let appearance = gpui_luma_look_shadcn::paint::selector_palette(mode, theme_mode, state);
    let menu = crate::controls::floating_menu::inspect_floating_menu_color_palette(mode, theme_mode, size);

    if ctx.catalog().tokens.is_empty() {
        return SelectorInspectPalette {
            trigger_background: resolved_from_hsla(
                appearance.trigger_background,
                if state.disabled {
                    ColorSource::CssVar { token: "muted".into() }
                } else {
                    ColorSource::Transparent
                },
            ),
            trigger_foreground: resolved_from_hsla(
                appearance.trigger_foreground,
                if state.disabled {
                    ColorSource::CssVar { token: "muted-foreground".into() }
                } else if state.hovered || state.pressed {
                    ColorSource::CssVar { token: "accent-foreground".into() }
                } else {
                    ColorSource::CssVar { token: "foreground".into() }
                },
            ),
            trigger_border: resolved_from_hsla(
                appearance.trigger_border,
                ColorSource::CssVar { token: "border".into() },
            ),
            focus_ring: appearance
                .focus_ring
                .map(|color| resolved_from_hsla(color, ColorSource::CssVar { token: "ring".into() })),
            items_panel: menu,
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "selector_inspect");
    let trigger_colors =
        gpui_luma_look_shadcn::tables::resolve_ghost_trigger_colors(&resolver, state.layer(), state.disabled)
            .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::GhostTriggerColorTable::fallback());
    let trigger_border = resolver.resolve_decl("border").unwrap_or_else(|_| {
        resolved_from_hsla(appearance.trigger_border, ColorSource::CssVar { token: "border".into() })
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

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}
