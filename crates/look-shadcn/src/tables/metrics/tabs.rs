//! Shared tabs metric resolution.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct TabsMetricTable {
    pub list_radius: crate::ResolvedMetric,
    pub list_gap: crate::ResolvedMetric,
    pub list_padding: crate::ResolvedMetric,
    pub item_padding_x: crate::ResolvedMetric,
    pub item_height: crate::ResolvedMetric,
    pub item_radius: crate::ResolvedMetric,
    pub indicator_height: crate::ResolvedMetric,
}

pub(crate) fn resolve_common_tabs_geometry(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    size: ControlSize,
) -> gpui_luma::theme::stylesheet::ResolvedTabsGeometry {
    use gpui_luma::theme::stylesheet::ResolvedTabsGeometry;
    use super::helpers::spacing_control_metric;
    let defaults = gpui_luma::controls::tabs::default_tabs_theme();
    let default_list = defaults.resolve_list(true, size);
    let default_item = defaults.resolve_item(true, InteractionState::default(), size);
    stylesheet.common.tabs.resolve_geometry(
        "default",
        ResolvedTabsGeometry {
            list_gap: {
                let value = spacing_control_metric(
                    &mode.catalog,
                    size,
                    crate::catalog::SpacingField::Gap,
                    mode.metrics.gap(size),
                );
                let source = match value.source {
                    crate::MetricSource::Derived { note } => {
                        gpui_luma::theme::provenance::MetricSource::Derived { note }
                    }
                    crate::MetricSource::Scaffold { path } => {
                        gpui_luma::theme::provenance::MetricSource::Scaffold { path }
                    }
                    crate::MetricSource::CssVar { token } => {
                        gpui_luma::theme::provenance::MetricSource::Authored { key: token }
                    }
                    crate::MetricSource::Constant { label } => {
                        gpui_luma::theme::provenance::MetricSource::Constant { label }
                    }
                };
                gpui_luma::theme::provenance::ResolvedMetric::new(value.value_px, source)
            },
            list_padding: gpui_luma::theme::provenance::ResolvedMetric::constant(
                default_list.padding,
                "SDK tabs list padding",
            ),
            indicator_height: gpui_luma::theme::provenance::ResolvedMetric::constant(
                default_item.indicator_height,
                "SDK tabs indicator height",
            ),
        },
    )
}

pub fn resolve_tabs_metrics(mode: &ShadcnModeTokens, theme_mode: ThemeMode, size: ControlSize) -> TabsMetricTable {
    resolve_tabs_metrics_with_stylesheet(mode, mode.stylesheet(), theme_mode, size)
}

/// Inspect the same selected stylesheet that the tabs theme uses for painting.
pub fn resolve_tabs_metrics_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> TabsMetricTable {
    use super::helpers::{derived_metric, radius_metric, spacing_control_metric};
    use crate::catalog::SpacingField;

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let list = crate::controls::tabs::tabs_list_look_with_stylesheet(mode, stylesheet, true, size);
    let item = crate::controls::tabs::tabs_item_look_with_stylesheet(
        mode,
        stylesheet,
        true,
        InteractionState::default(),
        size,
    );
    let geometry = resolve_common_tabs_geometry(mode, stylesheet, size);

    TabsMetricTable {
        list_radius: radius_metric(catalog, size, list.radius),
        list_gap: super::helpers::inspect_shared_metric(geometry.list_gap),
        list_padding: super::helpers::inspect_shared_metric(geometry.list_padding),
        item_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, item.padding_x),
        item_height: derived_metric("label line_height + spacing.s2", item.height),
        item_radius: radius_metric(catalog, size, item.radius),
        indicator_height: super::helpers::inspect_shared_metric(geometry.indicator_height),
    }
}
