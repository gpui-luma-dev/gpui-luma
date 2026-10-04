//! Shared button metric resolution.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};
use crate::catalog::SpacingField;
use crate::{MetricSource, ShadcnButtonStyle};
use gpui_luma::controls::button_family::ButtonFamilyRole;
use super::helpers::{control_size_key, spacing_control_metric};

#[derive(Clone, Debug)]
pub struct ButtonMetricTable {
    pub height: ResolvedMetric,
    pub icon_size: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn resolve_button_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
) -> ButtonMetricTable {
    resolve_button_metrics_with_stylesheet(mode, mode.stylesheet(), theme_mode, style, role, size, state)
}

pub fn resolve_button_metrics_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
) -> ButtonMetricTable {
    let look = crate::controls::button::button_look_with_stylesheet(
        mode, stylesheet, theme_mode, style, role, size, None, state,
    );
    let ctx = LookContext::new(mode, theme_mode, state);
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();

    let size_key = control_size_key(size);
    let family = if matches!(role, ButtonFamilyRole::Toggle { .. }) {
        "toggle"
    } else {
        "button"
    };
    let legacy = if matches!(role, ButtonFamilyRole::Toggle { .. }) {
        stylesheet.toggle.metrics_for_size(size)
    } else {
        stylesheet.button.metrics_for_size(size)
    };
    let stylesheet_metric = |field: &str, value_px| ResolvedMetric {
        value_px,
        source: if legacy.is_some() {
            MetricSource::Derived { note: format!("style.toml [{family}.metrics.{size_key}].{field}") }
        } else {
            let metric = match field {
                "padding_horizontal" => spacing_control_metric(catalog, size, SpacingField::PaddingX, value_px),
                "corner_radius" => super::helpers::radius_metric(catalog, size, value_px),
                _ => super::helpers::scaffold_control_metric(size_key, field, value_px),
            };
            metric.source
        },
    };

    let mut table = ButtonMetricTable {
        height: stylesheet_metric("height", look.height),
        icon_size: if legacy.is_some() {
            ResolvedMetric {
                value_px: look.icon_size,
                source: MetricSource::Constant { label: format!("style.toml [{family}.metrics.{size_key}].icon_size") },
            }
        } else {
            super::helpers::derived_metric("role icon size from box scale or label typography", look.icon_size)
        },
        padding_x: if matches!(role, ButtonFamilyRole::Icon) {
            super::helpers::derived_metric("icon button has no padding", look.padding_x)
        } else {
            stylesheet_metric("padding_horizontal", look.padding_x)
        },
        padding_y: if matches!(role, ButtonFamilyRole::Icon) {
            super::helpers::derived_metric("icon button has no padding", look.padding_y)
        } else {
            spacing_control_metric(catalog, size, SpacingField::PaddingY, look.padding_y)
        },
        gap: spacing_control_metric(catalog, size, SpacingField::Gap, look.gap),
        radius: if matches!(role, ButtonFamilyRole::Icon) {
            super::helpers::pill_radius_metric(catalog, look.radius)
        } else {
            stylesheet_metric("corner_radius", look.radius)
        },
        border_width: ResolvedMetric {
            value_px: metrics.border_width.default,
            source: MetricSource::Scaffold { path: "MetricTokens.border_width.default".into() },
        },
        focus_ring_width: ResolvedMetric {
            value_px: metrics.focus.width,
            source: MetricSource::Scaffold { path: "MetricTokens.focus.width".into() },
        },
        focus_ring_offset: focus_ring_offset_metric(
            gpui_luma::controls::button_family::button_family_effective_border(look.border),
            metrics,
        ),
    };
    let (geometry, _) = crate::controls::button::resolve_family_geometry(
        &ctx,
        stylesheet,
        size,
        matches!(role, ButtonFamilyRole::Toggle { .. }),
        1.0,
    );
    let shared = |metric: gpui_luma::theme::provenance::ResolvedMetric, fallback: ResolvedMetric| {
        if matches!(metric.source, gpui_luma::theme::provenance::MetricSource::Constant { .. }) {
            fallback
        } else {
            super::helpers::inspect_shared_metric(metric)
        }
    };
    table.height = shared(geometry.height, table.height);
    table.icon_size = shared(geometry.icon_size, table.icon_size);
    if !matches!(role, ButtonFamilyRole::Icon) {
        table.padding_x = shared(geometry.padding_x, table.padding_x);
        table.padding_y = shared(geometry.padding_y, table.padding_y);
        let tokens = if matches!(role, ButtonFamilyRole::Toggle { .. }) {
            &stylesheet.toggle.tokens
        } else {
            &stylesheet.button.tokens
        };
        if let Some(token) = tokens
            .get(size_key)
            .and_then(|rule| rule.corner_radius.as_ref())
            .filter(|token| crate::stylesheet::resolve_stylesheet_metric(token, metrics, size).is_some())
        {
            table.radius = super::helpers::derived_metric(
                format!("style.toml · {family}.tokens.{size_key}.corner_radius → {token}"),
                look.radius,
            );
        }
    }
    table.gap = shared(geometry.gap, table.gap);
    table
}

fn focus_ring_offset_metric(border: gpui::Hsla, metrics: &gpui_luma::theme::MetricTokens) -> ResolvedMetric {
    let border_width = metrics.border_width.default;
    let focus = metrics.focus.width;
    if border.a <= 0.0 {
        ResolvedMetric { value_px: 0.0, source: MetricSource::Derived { note: "inset · borderless".into() } }
    } else {
        ResolvedMetric {
            value_px: border_width + focus,
            source: MetricSource::Derived { note: "border_width.default + focus.width".into() },
        }
    }
}
