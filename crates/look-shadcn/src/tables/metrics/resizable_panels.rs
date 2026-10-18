//! Shared resizable panels metric resolution.

use crate::ResolvedMetric;
use gpui_luma::controls::resizable_panels::ResizeHandleSize;

#[derive(Clone, Debug)]
pub struct ResizablePanelsMetricTable {
    pub lane_px: ResolvedMetric,
    pub hit_target_px: ResolvedMetric,
    pub grip_cross_axis_px: ResolvedMetric,
    pub grip_main_axis_px: ResolvedMetric,
}

pub fn resolve_resizable_panels_metrics(
    handle_size: ResizeHandleSize,
    stylesheet: &gpui_luma::theme::stylesheet::CommonStylesheet,
) -> ResizablePanelsMetricTable {
    use super::helpers::inspect_shared_metric;

    let metrics = handle_size.metrics_with_stylesheet(stylesheet);
    let size_label = match handle_size {
        ResizeHandleSize::Sm => "sm",
        ResizeHandleSize::Md => "md",
        ResizeHandleSize::Lg => "lg",
    };

    let geometry = stylesheet.resizable_panels.resolve_geometry(
        size_label,
        gpui_luma::theme::stylesheet::ResizablePanelsGeometry {
            lane_width: metrics.lane_px,
            hit_target: metrics.hit_target_px,
            grip_cross_axis: metrics.grip_cross_axis_px,
            grip_main_axis: metrics.grip_main_axis_px,
            ..Default::default()
        },
    );
    ResizablePanelsMetricTable {
        lane_px: inspect_shared_metric(geometry.lane_width),
        hit_target_px: inspect_shared_metric(geometry.hit_target),
        grip_cross_axis_px: inspect_shared_metric(geometry.grip_cross_axis),
        grip_main_axis_px: inspect_shared_metric(geometry.grip_main_axis),
    }
}
