//! Inspect metadata for `context_menu`.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{
    AppearanceContext, ShadcnModeTokens,
};


pub struct ContextMenuInspectPalette {
    pub target_background: gpui_luma_look_shadcn::ResolvedColor,
    pub target_foreground: gpui_luma_look_shadcn::ResolvedColor,
    pub target_border: gpui_luma_look_shadcn::ResolvedColor,
    pub focus_ring: Option<gpui_luma_look_shadcn::ResolvedColor>,
    pub menu: crate::controls::floating_menu::FloatingMenuInspectPalette,
}

#[derive(Clone, Debug)]
pub struct ContextMenuInspectMetrics {
    pub target_padding_x: gpui_luma_look_shadcn::ResolvedMetric,
    pub target_padding_y: gpui_luma_look_shadcn::ResolvedMetric,
    pub target_radius: gpui_luma_look_shadcn::ResolvedMetric,
    pub target_min_width: gpui_luma_look_shadcn::ResolvedMetric,
    pub menu: crate::controls::floating_menu::FloatingMenuInspectMetrics,
}

pub fn inspect_context_menu_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
    size: ControlSize,
) -> ContextMenuInspectPalette {
    use gpui_luma_look_shadcn::{ColorSource, LookResolver};

    let ctx = AppearanceContext::new(mode, theme_mode, state);
    let appearance = if ctx.catalog().tokens.is_empty() {
        gpui_luma_look_shadcn::paint::context_menu_appearance_from_palette(&ctx)
    } else {
        gpui_luma_look_shadcn::paint::context_menu_appearance_from_catalog(&ctx).unwrap_or_else(|_| gpui_luma_look_shadcn::paint::context_menu_appearance_from_palette(&ctx))
    };

    let menu = crate::controls::floating_menu::inspect_floating_menu_color_palette(mode, theme_mode, size);

    if ctx.catalog().tokens.is_empty() {
        return ContextMenuInspectPalette {
            target_background: resolved_from_hsla(
                appearance.target_background,
                if state.disabled {
                    ColorSource::CssVar { token: "muted".into() }
                } else {
                    ColorSource::Transparent
                },
            ),
            target_foreground: resolved_from_hsla(
                appearance.target_foreground,
                if state.disabled {
                    ColorSource::CssVar { token: "muted-foreground".into() }
                } else if state.hovered || state.pressed {
                    ColorSource::CssVar { token: "accent-foreground".into() }
                } else {
                    ColorSource::CssVar { token: "foreground".into() }
                },
            ),
            target_border: resolved_from_hsla(appearance.target_border, ColorSource::CssVar { token: "border".into() }),
            focus_ring: state
                .focused
                .then(|| appearance.focus_ring)
                .flatten()
                .map(|color| resolved_from_hsla(color, ColorSource::CssVar { token: "ring".into() })),
            menu,
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "context_menu_inspect");
    let trigger_colors = gpui_luma_look_shadcn::tables::resolve_ghost_trigger_colors(&resolver, state.layer(), state.disabled)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::GhostTriggerColorTable::fallback());
    let target_background = trigger_colors.background;
    let target_foreground = trigger_colors.foreground;
    let target_border = resolver.resolve_decl("border").unwrap_or_else(|_| {
        resolved_from_hsla(appearance.target_border, ColorSource::CssVar { token: "border".into() })
    });
    let focus_ring = state.focused.then(|| resolver.resolve_decl("ring")).transpose().ok().flatten();

    ContextMenuInspectPalette { target_background, target_foreground, target_border, focus_ring, menu }
}

pub fn inspect_context_menu_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ContextMenuInspectMetrics {
    use crate::metrics::{derived_metric, radius_metric, spacing_control_metric};
    use gpui_luma_look_shadcn::catalog::SpacingField;

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let appearance = if ctx.catalog().tokens.is_empty() {
        gpui_luma_look_shadcn::paint::context_menu_appearance_from_palette(&ctx)
    } else {
        gpui_luma_look_shadcn::paint::context_menu_appearance_from_catalog(&ctx).unwrap_or_else(|_| gpui_luma_look_shadcn::paint::context_menu_appearance_from_palette(&ctx))
    };
    let catalog = ctx.catalog();

    ContextMenuInspectMetrics {
        target_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, appearance.target_padding_x),
        target_padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, appearance.target_padding_y),
        target_radius: radius_metric(catalog, size, appearance.target_radius),
        target_min_width: derived_metric("context menu target min width", appearance.target_min_width),
        menu: crate::controls::floating_menu::inspect_floating_menu_metrics(mode, theme_mode, size),
    }
}

fn resolved_from_hsla(value: gpui::Hsla, source: gpui_luma_look_shadcn::ColorSource) -> gpui_luma_look_shadcn::ResolvedColor {
    gpui_luma_look_shadcn::ResolvedColor { value, source }
}
