//! Inspect metadata for `popup_menu`.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{AppearanceContext, ShadcnModeTokens};

pub struct PopupMenuInspectPalette {
    pub trigger_background: gpui_luma_look_shadcn::ResolvedColor,
    pub trigger_foreground: gpui_luma_look_shadcn::ResolvedColor,
    pub trigger_border: gpui_luma_look_shadcn::ResolvedColor,
    pub focus_ring: Option<gpui_luma_look_shadcn::ResolvedColor>,
    pub menu: crate::controls::floating_menu::FloatingMenuInspectPalette,
}

#[derive(Clone, Debug)]
pub struct PopupMenuInspectMetrics {
    pub trigger: crate::controls::button::ButtonInspectMetrics,
    pub menu: crate::controls::floating_menu::FloatingMenuInspectMetrics,
}

pub fn inspect_popup_menu_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
    size: ControlSize,
) -> PopupMenuInspectPalette {
    use gpui_luma_look_shadcn::{ColorSource, LookResolver};

    let ctx = AppearanceContext::new(mode, theme_mode, state);
    let appearance = if ctx.catalog().tokens.is_empty() {
        gpui_luma_look_shadcn::paint::popup_menu_palette_from_palette(&ctx)
    } else {
        gpui_luma_look_shadcn::paint::popup_menu_palette_from_catalog(&ctx)
            .unwrap_or_else(|_| gpui_luma_look_shadcn::paint::popup_menu_palette_from_palette(&ctx))
    };

    let menu = crate::controls::floating_menu::inspect_floating_menu_color_palette(mode, theme_mode, size);

    if ctx.catalog().tokens.is_empty() {
        return PopupMenuInspectPalette {
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
            focus_ring: state
                .focused
                .then(|| appearance.focus_ring)
                .flatten()
                .map(|color| resolved_from_hsla(color, ColorSource::CssVar { token: "ring".into() })),
            menu,
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "popup_menu_inspect");
    let trigger_colors =
        gpui_luma_look_shadcn::tables::resolve_ghost_trigger_colors(&resolver, state.layer(), state.disabled)
            .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::GhostTriggerColorTable::fallback());
    let trigger_background = trigger_colors.background;
    let trigger_foreground = trigger_colors.foreground;
    let trigger_border = resolver.resolve_decl("border").unwrap_or_else(|_| {
        resolved_from_hsla(appearance.trigger_border, ColorSource::CssVar { token: "border".into() })
    });
    let focus_ring = state.focused.then(|| resolver.resolve_decl("ring")).transpose().ok().flatten();

    PopupMenuInspectPalette { trigger_background, trigger_foreground, trigger_border, focus_ring, menu }
}

pub fn inspect_popup_menu_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> PopupMenuInspectMetrics {
    use gpui_luma::controls::button_family::ButtonFamilyRole;

    PopupMenuInspectMetrics {
        trigger: crate::controls::button::inspect_button_metrics(
            mode,
            theme_mode,
            gpui_luma_look_shadcn::ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            size,
            InteractionState::default(),
        ),
        menu: crate::controls::floating_menu::inspect_floating_menu_metrics(mode, theme_mode, size),
    }
}

fn resolved_from_hsla(
    value: gpui::Hsla,
    source: gpui_luma_look_shadcn::ColorSource,
) -> gpui_luma_look_shadcn::ResolvedColor {
    gpui_luma_look_shadcn::ResolvedColor { value, source }
}
