//! Inspect metadata for `context_menu`.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{AppearanceContext, ShadcnModeTokens};

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
    use gpui_luma_look_shadcn::{ColorSource, LookResolver, ResolvedColor};

    let ctx = AppearanceContext::new(mode, theme_mode, state);
    let menu = crate::controls::floating_menu::inspect_floating_menu_color_palette(mode, theme_mode, size);

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "context_menu_inspect");
    let trigger_colors =
        gpui_luma_look_shadcn::tables::resolve_ghost_trigger_colors(&resolver, state.layer(), state.disabled)
            .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::GhostTriggerColorTable::fallback());
    let target_background = trigger_colors.background;
    let target_foreground = trigger_colors.foreground;
    let target_border = resolver.resolve_decl("border").unwrap_or_else(|_| ResolvedColor {
        value: ctx.catalog().color("border").expect("border"),
        source: ColorSource::CssVar { token: "border".into() },
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
    let appearance =
        gpui_luma_look_shadcn::paint::context_menu_appearance(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();

    ContextMenuInspectMetrics {
        target_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, appearance.target_padding_x),
        target_padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, appearance.target_padding_y),
        target_radius: radius_metric(catalog, size, appearance.target_radius),
        target_min_width: derived_metric("context menu target min width", appearance.target_min_width),
        menu: crate::controls::floating_menu::inspect_floating_menu_metrics(mode, theme_mode, size),
    }
}
