//! Inspect metadata for `scrollbar`.

use gpui_luma::theme::{InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{AppearanceContext, ColorSource, LookResolver, ResolvedColor, ShadcnModeTokens};

use gpui_luma::controls::scrollbar::ScrollbarOrientation;

pub struct ScrollbarInspectPalette {
    pub track_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub focus_ring: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ScrollbarInspectMetrics {
    pub length: gpui_luma_look_shadcn::ResolvedMetric,
    pub thickness: gpui_luma_look_shadcn::ResolvedMetric,
    pub track_thickness: gpui_luma_look_shadcn::ResolvedMetric,
    pub thumb_thickness: gpui_luma_look_shadcn::ResolvedMetric,
    pub min_thumb_length: gpui_luma_look_shadcn::ResolvedMetric,
    pub radius: gpui_luma_look_shadcn::ResolvedMetric,
}

pub fn inspect_scrollbar_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    orientation: ScrollbarOrientation,
    state: InteractionState,
) -> ScrollbarInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if ctx.catalog().tokens.is_empty() {
        let appearance = gpui_luma_look_shadcn::paint::scrollbar_appearance(mode, state, orientation);
        return ScrollbarInspectPalette {
            track_background: resolved_from_hsla(
                appearance.track_background,
                if state.disabled {
                    ColorSource::CssVar { token: "muted".into() }
                } else {
                    ColorSource::Transparent
                },
            ),
            thumb_background: resolved_from_hsla(
                appearance.thumb_background,
                if state.disabled {
                    ColorSource::CssVar { token: "muted-foreground".into() }
                } else if state.pressed {
                    ColorSource::Derived { note: "darken(border, 8%)".into() }
                } else {
                    ColorSource::CssVar { token: "border".into() }
                },
            ),
            focus_ring: state
                .focused
                .then(|| appearance.focus_ring)
                .flatten()
                .map(|color| resolved_from_hsla(color, ColorSource::CssVar { token: "ring".into() })),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "scrollbar_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_scrollbar_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::ScrollbarColorTable::fallback());
    let focus_ring = state.focused.then(|| resolver.resolve_decl("ring")).transpose().ok().flatten();

    ScrollbarInspectPalette {
        track_background: colors.track_background,
        thumb_background: colors.thumb_background,
        focus_ring,
    }
}

pub fn inspect_scrollbar_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    orientation: ScrollbarOrientation,
) -> ScrollbarInspectMetrics {
    use crate::metrics::{derived_metric, pill_radius_metric};

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let appearance =
        gpui_luma_look_shadcn::paint::scrollbar_appearance_from_catalog(&ctx, orientation).unwrap_or_else(|_| {
            gpui_luma_look_shadcn::paint::scrollbar_appearance(mode, InteractionState::default(), orientation)
        });
    let catalog = ctx.catalog();

    ScrollbarInspectMetrics {
        length: derived_metric("gallery scrollbar demo length", appearance.length),
        thickness: derived_metric("scrollbar chrome thickness", appearance.thickness),
        track_thickness: derived_metric("track hit target", appearance.track_thickness),
        thumb_thickness: derived_metric("thumb visual size", appearance.thumb_thickness),
        min_thumb_length: derived_metric("minimum draggable thumb", appearance.min_thumb_length),
        radius: pill_radius_metric(catalog, appearance.radius),
    }
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}
