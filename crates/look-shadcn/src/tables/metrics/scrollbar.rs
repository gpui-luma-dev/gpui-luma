//! Shared scrollbar metric resolution.

use luma::theme::{InteractionState, ThemeMode};
use crate::{LookContext, ShadcnModeTokens};
use luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};

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
    use super::helpers::{derived_metric, pill_radius_metric};

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let look = crate::paint::scrollbar_look(
        mode,
        InteractionState::default(),
        orientation,
        luma::theme::ControlSize::Md,
        style,
    );
    let catalog = ctx.catalog();

    ScrollbarMetricTable {
        length: derived_metric("gallery scrollbar demo length", look.length),
        thickness: derived_metric("scrollbar chrome thickness", look.thickness),
        track_thickness: derived_metric("track hit target", look.track_thickness),
        thumb_thickness: derived_metric("thumb visual size", look.thumb_thickness),
        min_thumb_length: derived_metric("minimum draggable thumb", look.min_thumb_length),
        radius: pill_radius_metric(catalog, look.radius),
    }
}
