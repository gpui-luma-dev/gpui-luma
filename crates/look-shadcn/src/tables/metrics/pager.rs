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

    PagerMetricTable {
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
    }
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
