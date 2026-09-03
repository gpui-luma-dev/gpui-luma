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
    use crate::metrics::{derived_metric, pill_radius_metric};

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let look = luma_look_shadcn::paint::scrollbar_look(
        mode,
        InteractionState::default(),
        orientation,
        luma::theme::ControlSize::Md,
        style,
    );
    let catalog = ctx.catalog();

    ScrollbarInspectMetrics {
        length: derived_metric("gallery scrollbar demo length", look.length),
        thickness: derived_metric("scrollbar chrome thickness", look.thickness),
        track_thickness: derived_metric("track hit target", look.track_thickness),
        thumb_thickness: derived_metric("thumb visual size", look.thumb_thickness),
        min_thumb_length: derived_metric("minimum draggable thumb", look.min_thumb_length),
        radius: pill_radius_metric(catalog, look.radius),
    }
}
