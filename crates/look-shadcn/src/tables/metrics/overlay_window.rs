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

    OverlayWindowMetricTable {
        radius: derived_metric("overlay radius = radius.lg", shell.radius),
        padding: derived_metric(format!("{size_key} overlay padding"), shell.padding),
        min_width: derived_metric(format!("{size_key} overlay min width"), shell.min_width),
        max_width: derived_metric(format!("{size_key} overlay max width"), shell.max_width),
        estimated_height: derived_metric(format!("{size_key} overlay estimated height"), shell.estimated_height),
    }
}
