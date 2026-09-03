//! Inspect metadata for `resizable_panels`.

use luma::theme::{InteractionState, ThemeMode};
use luma_look_shadcn::{LookContext, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

use luma::controls::resizable_panels::ResizeHandleSize;

pub struct ResizablePanelsInspectPalette {
    pub border: ResolvedColor,
    pub divider: ResolvedColor,
    pub grip: ResolvedColor,
    pub grip_emphasis: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResizablePanelsInspectMetrics {
    pub lane_px: ResolvedMetric,
    pub hit_target_px: ResolvedMetric,
    pub grip_cross_axis_px: ResolvedMetric,
    pub grip_main_axis_px: ResolvedMetric,
}

pub fn inspect_resizable_panels_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> ResizablePanelsInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "resizable_panels_inspect");
    let colors = luma_look_shadcn::tables::resolve_resizable_panels_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| luma_look_shadcn::tables::ResizablePanelsColorTable::fallback());
    ResizablePanelsInspectPalette {
        border: colors.border,
        divider: colors.divider,
        grip: colors.grip,
        grip_emphasis: colors.grip_emphasis,
    }
}

pub fn inspect_resizable_panels_metrics(handle_size: ResizeHandleSize) -> ResizablePanelsInspectMetrics {
    use crate::metrics::derived_metric;

    let metrics = handle_size.metrics();
    let size_label = match handle_size {
        ResizeHandleSize::Sm => "sm",
        ResizeHandleSize::Md => "md",
        ResizeHandleSize::Lg => "lg",
    };

    ResizablePanelsInspectMetrics {
        lane_px: derived_metric(format!("{size_label} handle lane"), metrics.lane_px),
        hit_target_px: derived_metric(format!("{size_label} handle hit target"), metrics.hit_target_px),
        grip_cross_axis_px: derived_metric(format!("{size_label} grip cross axis"), metrics.grip_cross_axis_px),
        grip_main_axis_px: derived_metric(format!("{size_label} grip main axis"), metrics.grip_main_axis_px),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::sample_catalog;
    use luma::theme::InteractionLayer;

    #[test]
    fn resizable_panels_metadata_covers_enabled_and_disabled_paths() {
        assert_eq!(
            luma_look_shadcn::stylesheet::resolve_resizable_panels_colors_metadata(
                luma_look_shadcn::embedded_stylesheet()
            )
            .len(),
            5
        );
    }

    #[test]
    fn inspect_hovered_grip_uses_accent_layer() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_resizable_panels_color_palette(
            &mode,
            ThemeMode::Light,
            luma::theme::InteractionState { hovered: true, ..Default::default() },
        );
        let resolver = LookResolver::new(&mode.catalog, ThemeMode::Light, "test");
        let expected =
            luma_look_shadcn::tables::resolve_resizable_panels_colors(&resolver, false, InteractionLayer::Hovered)
                .expect("colors")
                .grip_emphasis;
        assert_eq!(palette.grip_emphasis.value, expected.hsla());
    }

    #[test]
    fn inspect_handle_metrics_follow_preset_sizes() {
        let metrics = inspect_resizable_panels_metrics(ResizeHandleSize::Md);
        assert!((metrics.lane_px.value_px - 12.0).abs() < f32::EPSILON);
    }
}
