use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::MetricTokens;

use super::resolve::resolve_color;
use super::super::catalog::CssTokenMap;

pub(crate) fn focus_ring_color(catalog: &CssTokenMap) -> anyhow::Result<gpui::Hsla> {
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
