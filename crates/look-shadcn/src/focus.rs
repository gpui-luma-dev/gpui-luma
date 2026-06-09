use gpui_luma::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use gpui_luma::theme::MetricTokens;

use super::resolve::resolve_color;
use super::catalog::CssTokenMap;

use super::palette::ShadcnPalette;

pub(crate) fn focus_adorner_from_palette(
    palette: &ShadcnPalette,
    metrics: &MetricTokens,
    focused: bool,
) -> Option<AdornerSpec> {
    if !focused {
        return None;
    }

    Some(AdornerSpec::FocusRing(FocusRingAdornerSpec {
        color: palette.focus_ring,
        placement: AdornerPlacement::Oversize,
        distance: metrics.border_width.default + metrics.focus.width,
        width: metrics.focus.width,
    }))
}

pub fn focus_ring_color(catalog: &CssTokenMap) -> anyhow::Result<gpui::Hsla> {
    resolve_color(catalog, "ring")
}

pub(crate) fn focus_adorner(
    catalog: &CssTokenMap,
    metrics: &MetricTokens,
    focused: bool,
) -> anyhow::Result<Option<AdornerSpec>> {
    if !focused {
        return Ok(None);
    }

    Ok(Some(AdornerSpec::FocusRing(FocusRingAdornerSpec {
        color: focus_ring_color(catalog)?,
        placement: AdornerPlacement::Oversize,
        distance: metrics.border_width.default + metrics.focus.width,
        width: metrics.focus.width,
    })))
}
