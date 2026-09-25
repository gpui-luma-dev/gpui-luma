//! Inspect metadata for `scrollbar`.

use luma::theme::{InteractionState, ThemeMode};
use luma_look_shadcn::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

use luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};

pub struct ScrollbarInspectPalette {
    pub track_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ScrollbarInspectMetrics {
    pub length: luma_look_shadcn::ResolvedMetric,
    pub thickness: luma_look_shadcn::ResolvedMetric,
    pub track_thickness: luma_look_shadcn::ResolvedMetric,
    pub thumb_thickness: luma_look_shadcn::ResolvedMetric,
    pub min_thumb_length: luma_look_shadcn::ResolvedMetric,
    pub radius: luma_look_shadcn::ResolvedMetric,
}

pub fn inspect_scrollbar_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ScrollbarStyle,
    state: InteractionState,
) -> ScrollbarInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "scrollbar_inspect");
    let colors = luma_look_shadcn::tables::resolve_scrollbar_colors(&resolver, style, state.disabled, state.layer())
        .unwrap_or_else(|_| luma_look_shadcn::tables::ScrollbarColorTable::fallback());

    ScrollbarInspectPalette { track_background: colors.track_background, thumb_background: colors.thumb_background }
}

pub fn inspect_scrollbar_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    orientation: ScrollbarOrientation,
    style: ScrollbarStyle,
) -> ScrollbarInspectMetrics {
    let table = luma_look_shadcn::tables::metrics::resolve_scrollbar_metrics(mode, theme_mode, orientation, style);
    table.into()
}

impl From<luma_look_shadcn::tables::metrics::ScrollbarMetricTable> for ScrollbarInspectMetrics {
    fn from(table: luma_look_shadcn::tables::metrics::ScrollbarMetricTable) -> Self {
        Self {
            length: table.length,
            thickness: table.thickness,
            track_thickness: table.track_thickness,
            thumb_thickness: table.thumb_thickness,
            min_thumb_length: table.min_thumb_length,
            radius: table.radius,
        }
    }
}
