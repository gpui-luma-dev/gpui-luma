//! Shared pager metric resolution.

use crate::ResolvedMetric;
use crate::{ShadcnLook, ResolvedTypography, TypographySource};
use gpui_luma::controls::{button_family::ButtonFamilyRole, pager::PagerStyle};
use gpui_luma::infra::shadow_layout::shadow_projection_extent;

#[derive(Clone, Debug)]
pub struct PagerMetricTable {
    pub control_height: ResolvedMetric,
    pub button_size: ResolvedMetric,
    pub button_min_width: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub group_gap: ResolvedMetric,
    pub font_family: ResolvedTypography,
    pub reserved_shadow_extent: ResolvedMetric,
}

pub fn resolve_pager_metrics(look: &ShadcnLook, style: PagerStyle) -> PagerMetricTable {
    use super::helpers::derived_metric;

    let pager = crate::paint::pager_look(look, true, style);
    let style_label = pager_style_label(style);
    let button = look.resolve_outline_button(
        ButtonFamilyRole::Toggle { selected: false },
        gpui_luma::theme::ControlSize::Sm,
        gpui_luma::theme::InteractionState::default(),
    );
    let reserved_shadow_extent = shadow_projection_extent(button.shadow.as_deref(), 1.0, true);

    let mut table = PagerMetricTable {
        control_height: derived_metric(format!("{style_label} page-size trigger height"), pager.control_height),
        button_size: derived_metric(format!("{style_label} button size"), pager.button_size),
        button_min_width: derived_metric(format!("{style_label} button min width"), pager.button_min_width),
        radius: derived_metric(format!("{style_label} radius = radius.sm"), pager.radius),
        padding_x: derived_metric(format!("{style_label} shell padding x"), pager.padding_x),
        padding_y: derived_metric(format!("{style_label} shell padding y"), pager.padding_y),
        gap: derived_metric(format!("{style_label} item gap"), pager.gap),
        group_gap: derived_metric(format!("{style_label} group gap"), pager.group_gap),
        font_family: pager_font_family(look),
        reserved_shadow_extent: ResolvedMetric {
            value_px: reserved_shadow_extent,
            source: crate::MetricSource::Derived { note: "button outline shadow projection extent".into() },
        },
    };
    let geometry = look.mode_tokens().stylesheet().common.pager.resolve_geometry(
        style_label,
        gpui_luma::theme::stylesheet::PagerGeometry {
            control_height: table.control_height.value_px,
            button_size: table.button_size.value_px,
            button_min_width: table.button_min_width.value_px,
            padding_x: table.padding_x.value_px,
            padding_y: table.padding_y.value_px,
            gap: table.gap.value_px,
            group_gap: table.group_gap.value_px,
            ..Default::default()
        },
    );
    table.control_height = super::helpers::prefer_shared_metric(geometry.control_height, table.control_height);
    table.button_size = super::helpers::prefer_shared_metric(geometry.button_size, table.button_size);
    table.button_min_width = super::helpers::prefer_shared_metric(geometry.button_min_width, table.button_min_width);
    table.padding_x = super::helpers::prefer_shared_metric(geometry.padding_x, table.padding_x);
    table.padding_y = super::helpers::prefer_shared_metric(geometry.padding_y, table.padding_y);
    table.gap = super::helpers::prefer_shared_metric(geometry.gap, table.gap);
    table.group_gap = super::helpers::prefer_shared_metric(geometry.group_gap, table.group_gap);

    table
}

fn pager_font_family(look: &ShadcnLook) -> ResolvedTypography {
    let family = look.mode_tokens().typography.font.sans.family.clone();
    if look.mode_tokens().catalog.get("font-sans").is_some() {
        ResolvedTypography { value: family, source: TypographySource::CssVar { token: "font-sans".into() } }
    } else {
        ResolvedTypography {
            value: family,
            source: TypographySource::Scaffold { path: "LumaTypography.font.sans.family".into() },
        }
    }
}

fn pager_style_label(style: PagerStyle) -> &'static str {
    match style {
        PagerStyle::Minimal => "minimal",
        PagerStyle::MinimalEdge => "minimal-edge",
        PagerStyle::Numeric => "numeric",
    }
}
