//! Inspect metadata for `slider`.

use gpui_luma::theme::{InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

pub struct SliderInspectPalette {
    pub track_background: ResolvedColor,
    pub fill_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
    pub focus_ring: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct SliderInspectMetrics {
    pub width: gpui_luma_look_shadcn::ResolvedMetric,
    pub height: gpui_luma_look_shadcn::ResolvedMetric,
    pub track_height: gpui_luma_look_shadcn::ResolvedMetric,
    pub thumb_size: gpui_luma_look_shadcn::ResolvedMetric,
    pub radius: gpui_luma_look_shadcn::ResolvedMetric,
}

pub fn inspect_slider_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> SliderInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let layer = state.layer();
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "slider_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_slider_colors(&resolver, layer)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::SliderColorTable::fallback());
    let focus_ring = state.focused.then(|| resolver.resolve_decl("ring")).transpose().ok().flatten();

    SliderInspectPalette {
        track_background: colors.track_background,
        fill_background: colors.fill_background,
        thumb_background: colors.thumb_background,
        thumb_border: colors.thumb_border,
        focus_ring,
    }
}

pub fn inspect_slider_metrics(mode: &ShadcnModeTokens, theme_mode: ThemeMode) -> SliderInspectMetrics {
    use crate::metrics::{derived_metric, pill_radius_metric};

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let look = gpui_luma_look_shadcn::paint::slider_look(mode, theme_mode, InteractionState::default());

    SliderInspectMetrics {
        width: derived_metric("slider demo width", look.width),
        height: derived_metric("slider control height", look.height),
        track_height: derived_metric("track rail height", look.track_height),
        thumb_size: derived_metric("thumb diameter", look.thumb_size),
        radius: pill_radius_metric(ctx.catalog(), look.radius),
    }
}
