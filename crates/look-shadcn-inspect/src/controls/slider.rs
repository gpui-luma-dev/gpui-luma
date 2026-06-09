//! Inspect metadata for `slider`.

use gpui_luma::theme::{InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{AppearanceContext, ColorSource, LookResolver, ResolvedColor, ShadcnModeTokens};

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
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if ctx.catalog().tokens.is_empty() {
        let appearance = gpui_luma_look_shadcn::paint::slider_appearance_from_palette(&ctx);
        return SliderInspectPalette {
            track_background: resolved_from_hsla(
                appearance.track_background,
                if state.disabled {
                    ColorSource::CssVar { token: "muted".into() }
                } else {
                    ColorSource::CssVar { token: "border".into() }
                },
            ),
            fill_background: resolved_from_hsla(
                appearance.fill_background,
                if state.disabled {
                    ColorSource::CssVar { token: "muted-foreground".into() }
                } else {
                    ColorSource::CssVar { token: "primary".into() }
                },
            ),
            thumb_background: resolved_from_hsla(
                appearance.thumb_background,
                if state.disabled {
                    ColorSource::CssVar { token: "muted-foreground".into() }
                } else {
                    ColorSource::CssVar { token: "background".into() }
                },
            ),
            thumb_border: resolved_from_hsla(
                appearance.thumb_border,
                if state.disabled {
                    ColorSource::CssVar { token: "muted".into() }
                } else {
                    ColorSource::CssVar { token: "primary".into() }
                },
            ),
            focus_ring: state
                .focused
                .then(|| appearance.focus_ring)
                .flatten()
                .map(|color| resolved_from_hsla(color, ColorSource::CssVar { token: "ring".into() })),
        };
    }

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

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let appearance = if ctx.catalog().tokens.is_empty() {
        gpui_luma_look_shadcn::paint::slider_appearance_from_palette(&ctx)
    } else {
        gpui_luma_look_shadcn::paint::slider_appearance_from_catalog(&ctx)
            .unwrap_or_else(|_| gpui_luma_look_shadcn::paint::slider_appearance_from_palette(&ctx))
    };

    SliderInspectMetrics {
        width: derived_metric("slider demo width", appearance.width),
        height: derived_metric("slider control height", appearance.height),
        track_height: derived_metric("track rail height", appearance.track_height),
        thumb_size: derived_metric("thumb diameter", appearance.thumb_size),
        radius: pill_radius_metric(ctx.catalog(), appearance.radius),
    }
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}
