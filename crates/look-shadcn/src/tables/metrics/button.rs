//! Shared button metric resolution.

use luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, ResolvedMetric, ShadcnModeTokens};
use crate::catalog::SpacingField;
use crate::{MetricSource, ShadcnButtonStyle};
use luma::controls::button_family::ButtonFamilyRole;
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
    let look = crate::paint::button_look(mode, theme_mode, style, role, size, state);
    let ctx = LookContext::new(mode, theme_mode, state);
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();

    let size_key = control_size_key(size);
    let family = if matches!(role, ButtonFamilyRole::Toggle { .. }) {
        "toggle"
    } else {
        "button"
    };
    let stylesheet_metric = |field: &str, value_px| ResolvedMetric {
        value_px,
        source: MetricSource::Derived { note: format!("style.toml [{family}.metrics.{size_key}].{field}") },
    };

    ButtonMetricTable {
        height: stylesheet_metric("height", look.height),
        icon_size: ResolvedMetric {
            value_px: look.icon_size,
            source: MetricSource::Constant { label: format!("style.toml [{family}.metrics.{size_key}].icon_size") },
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
            luma::controls::button_family::button_family_effective_border(look.border),
            metrics,
        ),
    }
}

fn focus_ring_offset_metric(border: gpui::Hsla, metrics: &luma::theme::MetricTokens) -> ResolvedMetric {
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
