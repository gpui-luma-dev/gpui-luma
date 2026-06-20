//! Inspect metadata for `floating_menu`.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

pub struct FloatingMenuInspectPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub item_hover_background: ResolvedColor,
    pub item_hover_foreground: ResolvedColor,
    pub item_disabled_foreground: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct FloatingMenuInspectMetrics {
    pub radius: gpui_luma_look_shadcn::ResolvedMetric,
    pub padding: gpui_luma_look_shadcn::ResolvedMetric,
    pub min_width: gpui_luma_look_shadcn::ResolvedMetric,
    pub item_height: gpui_luma_look_shadcn::ResolvedMetric,
    pub item_padding_x: gpui_luma_look_shadcn::ResolvedMetric,
    pub item_gap: gpui_luma_look_shadcn::ResolvedMetric,
    pub item_icon_size: gpui_luma_look_shadcn::ResolvedMetric,
    pub item_radius: gpui_luma_look_shadcn::ResolvedMetric,
    pub submenu_offset_x: gpui_luma_look_shadcn::ResolvedMetric,
}

pub fn inspect_floating_menu_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    _size: ControlSize,
) -> FloatingMenuInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "floating_menu_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_floating_menu_colors(&resolver, true)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::FloatingMenuColorTable::fallback());
    FloatingMenuInspectPalette {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
        item_hover_background: colors.item_hover_background,
        item_hover_foreground: colors.item_hover_foreground,
        item_disabled_foreground: colors.item_disabled_foreground,
    }
}

pub fn inspect_floating_menu_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> FloatingMenuInspectMetrics {
    use crate::metrics::{derived_metric, scaffold_control_metric, spacing_control_metric};
    use gpui_luma_look_shadcn::catalog::SpacingField;

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let look = gpui_luma_look_shadcn::paint::floating_menu_look(mode, theme_mode, size);
    let catalog = ctx.catalog();

    FloatingMenuInspectMetrics {
        radius: derived_metric("lg = --radius", look.radius),
        padding: derived_metric("padding_y × 0.5", look.padding),
        min_width: derived_metric("floating menu min width", look.min_width),
        item_height: derived_metric("control_height × 0.9", look.item_height),
        item_padding_x: derived_metric("padding_x × 0.75", look.item_padding_x),
        item_gap: spacing_control_metric(catalog, size, SpacingField::Gap, look.item_gap),
        item_icon_size: derived_metric("control_height × 0.44", look.item_icon_size),
        item_radius: scaffold_control_metric("sm", "radius", look.item_radius),
        submenu_offset_x: derived_metric("gap × 0.5", look.submenu_offset_x),
    }
}
