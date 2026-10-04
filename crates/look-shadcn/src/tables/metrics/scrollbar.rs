//! Shared scrollbar metric resolution.

use gpui_luma::theme::{InteractionState, ThemeMode};
use crate::{LookContext, ShadcnModeTokens};
use gpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};

#[derive(Clone, Debug)]
pub struct ScrollbarMetricTable {
    pub length: crate::ResolvedMetric,
    pub thickness: crate::ResolvedMetric,
    pub track_thickness: crate::ResolvedMetric,
    pub thumb_thickness: crate::ResolvedMetric,
    pub min_thumb_length: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
}

pub fn resolve_scrollbar_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    orientation: ScrollbarOrientation,
    style: ScrollbarStyle,
) -> ScrollbarMetricTable {
    resolve_scrollbar_metrics_for_size(mode, theme_mode, orientation, style, gpui_luma::theme::ControlSize::Md)
}

/// Resolve the selected look's Scrollbar dimensions and their sources for a size.
pub fn resolve_scrollbar_metrics_for_size(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    orientation: ScrollbarOrientation,
    style: ScrollbarStyle,
    size: gpui_luma::theme::ControlSize,
) -> ScrollbarMetricTable {
    use super::helpers::{derived_metric, pill_radius_metric};

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let look = crate::paint::scrollbar_look(mode, InteractionState::default(), orientation, size, style);
    let catalog = ctx.catalog();

    let mut table = ScrollbarMetricTable {
        length: derived_metric("gallery scrollbar demo length", look.length),
        thickness: derived_metric("scrollbar chrome thickness", look.thickness),
        track_thickness: derived_metric("track hit target", look.track_thickness),
        thumb_thickness: derived_metric("thumb visual size", look.thumb_thickness),
        min_thumb_length: derived_metric("minimum draggable thumb", look.min_thumb_length),
        radius: pill_radius_metric(catalog, look.radius),
    };
    let geometry = mode.stylesheet().common.scrollbar.resolve_geometry(
        super::helpers::control_size_key(size),
        gpui_luma::theme::stylesheet::ScrollbarGeometry {
            thickness: table.thickness.value_px,
            track_thickness: table.track_thickness.value_px,
            thumb_thickness: table.thumb_thickness.value_px,
            min_thumb_length: table.min_thumb_length.value_px,
            length_h: table.length.value_px,
            length_v: table.length.value_px,
        },
    );
    table.thickness = super::helpers::prefer_shared_metric(geometry.thickness, table.thickness);
    table.track_thickness = super::helpers::prefer_shared_metric(geometry.track_thickness, table.track_thickness);
    table.thumb_thickness = super::helpers::prefer_shared_metric(geometry.thumb_thickness, table.thumb_thickness);
    table.min_thumb_length = super::helpers::prefer_shared_metric(geometry.min_thumb_length, table.min_thumb_length);
    table.length = super::helpers::prefer_shared_metric(
        match orientation {
            ScrollbarOrientation::Horizontal => geometry.length_h,
            ScrollbarOrientation::Vertical => geometry.length_v,
        },
        table.length,
    );
    table
}
