//! Inspect metadata for `context_menu`.

use luma::theme::{ControlSize, InteractionState, ThemeMode};
use luma_look_shadcn::ShadcnModeTokens;

pub struct ContextMenuInspectPalette {
    pub target_background: luma_look_shadcn::ResolvedColor,
    pub target_foreground: luma_look_shadcn::ResolvedColor,
    pub target_border: luma_look_shadcn::ResolvedColor,
    pub menu: crate::controls::floating_menu::FloatingMenuInspectPalette,
}

#[derive(Clone, Debug)]
pub struct ContextMenuInspectMetrics {
    pub target_padding_x: luma_look_shadcn::ResolvedMetric,
    pub target_padding_y: luma_look_shadcn::ResolvedMetric,
    pub target_radius: luma_look_shadcn::ResolvedMetric,
    pub target_min_width: luma_look_shadcn::ResolvedMetric,
    pub menu: crate::controls::floating_menu::FloatingMenuInspectMetrics,
}

pub fn inspect_context_menu_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
    size: ControlSize,
) -> ContextMenuInspectPalette {
    let colors = luma_look_shadcn::tables::resolve_context_menu_colors(mode, theme_mode, state);
    let menu = crate::controls::floating_menu::inspect_floating_menu_color_palette(mode, theme_mode, size);
    ContextMenuInspectPalette {
        target_background: colors.background,
        target_foreground: colors.foreground,
        target_border: colors.border,
        menu,
    }
}

pub fn inspect_context_menu_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ContextMenuInspectMetrics {
    let table = luma_look_shadcn::tables::metrics::resolve_context_menu_metrics(mode, theme_mode, size);
    table.into()
}

impl From<luma_look_shadcn::tables::metrics::ContextMenuMetricTable> for ContextMenuInspectMetrics {
    fn from(table: luma_look_shadcn::tables::metrics::ContextMenuMetricTable) -> Self {
        Self {
            target_padding_x: table.target_padding_x,
            target_padding_y: table.target_padding_y,
            target_radius: table.target_radius,
            target_min_width: table.target_min_width,
            menu: table.menu.into(),
        }
    }
}
