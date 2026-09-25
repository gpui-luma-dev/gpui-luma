//! Shared resizable panels metric resolution.

use crate::ResolvedMetric;
use luma::controls::resizable_panels::ResizeHandleSize;

#[derive(Clone, Debug)]
pub struct ResizablePanelsMetricTable {
    pub lane_px: ResolvedMetric,
    pub hit_target_px: ResolvedMetric,
    pub grip_cross_axis_px: ResolvedMetric,
    pub grip_main_axis_px: ResolvedMetric,
}

pub fn resolve_resizable_panels_metrics(handle_size: ResizeHandleSize) -> ResizablePanelsMetricTable {
    use super::helpers::derived_metric;

    let metrics = handle_size.metrics();
    let size_label = match handle_size {
        ResizeHandleSize::Sm => "sm",
        ResizeHandleSize::Md => "md",
        ResizeHandleSize::Lg => "lg",
    };

    ResizablePanelsMetricTable {
        lane_px: derived_metric(format!("{size_label} handle lane"), metrics.lane_px),
        hit_target_px: derived_metric(format!("{size_label} handle hit target"), metrics.hit_target_px),
        grip_cross_axis_px: derived_metric(format!("{size_label} grip cross axis"), metrics.grip_cross_axis_px),
        grip_main_axis_px: derived_metric(format!("{size_label} grip main axis"), metrics.grip_main_axis_px),
    }
}
