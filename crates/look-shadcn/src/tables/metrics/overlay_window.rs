//! Shared overlay window metric resolution.

use gpui_luma::theme::ControlSize;
use crate::ResolvedMetric;
use gpui_luma::controls::overlay_window::OverlayWindowMode;
use crate::ShadcnLook;

#[derive(Clone, Debug)]
pub struct OverlayWindowMetricTable {
    pub radius: ResolvedMetric,
    pub padding: ResolvedMetric,
    pub min_width: ResolvedMetric,
    pub max_width: ResolvedMetric,
    pub estimated_height: ResolvedMetric,
}

pub fn resolve_overlay_window_metrics(
    look: &ShadcnLook,
    size: ControlSize,
    mode: OverlayWindowMode,
) -> OverlayWindowMetricTable {
    use super::helpers::derived_metric;

    let shell = crate::paint::overlay_window_look(look, size, mode);
    let size_key = match size {
        ControlSize::Sm => "sm",
        ControlSize::Md => "md",
        ControlSize::Lg => "lg",
    };

    let mut table = OverlayWindowMetricTable {
        radius: derived_metric("overlay radius = radius.lg", shell.radius),
        padding: derived_metric(format!("{size_key} overlay padding"), shell.padding),
        min_width: derived_metric(format!("{size_key} overlay min width"), shell.min_width),
        max_width: derived_metric(format!("{size_key} overlay max width"), shell.max_width),
        estimated_height: derived_metric(format!("{size_key} overlay estimated height"), shell.estimated_height),
    };
    let geometry = look.mode_tokens().stylesheet().common.overlay_window.resolve_geometry(
        size_key,
        gpui_luma::theme::stylesheet::OverlayWindowGeometry {
            padding: table.padding.value_px,
            min_width: table.min_width.value_px,
            max_width: table.max_width.value_px,
            estimated_height: table.estimated_height.value_px,
            ..Default::default()
        },
    );
    table.padding = super::helpers::prefer_shared_metric(geometry.padding, table.padding);
    table.min_width = super::helpers::prefer_shared_metric(geometry.min_width, table.min_width);
    table.max_width = super::helpers::prefer_shared_metric(geometry.max_width, table.max_width);
    table.estimated_height = super::helpers::prefer_shared_metric(geometry.estimated_height, table.estimated_height);

    table
}
