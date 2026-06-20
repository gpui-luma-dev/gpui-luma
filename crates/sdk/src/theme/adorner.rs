use gpui::{Div, Hsla, div, px, prelude::*};

/// Adorner policy (current): controls support at most one adorner per look.
///
/// We intentionally use `Option<AdornerSpec>` in control look structs today
/// to match the current capability (`FocusRing`) and avoid speculative multi-adorner
/// composition paths.
///
/// If additional adorner kinds are introduced in the future (for example validation
/// or status adorners), revisit this module and control look contracts together
/// so composition/ordering is designed explicitly rather than incrementally.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdornerPlacement {
    Inset,
    Oversize,
}

#[derive(Clone, Copy, Debug)]
pub struct FocusRingAdornerSpec {
    pub color: Hsla,
    pub placement: AdornerPlacement,
    pub distance: f32,
    pub width: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum AdornerSpec {
    FocusRing(FocusRingAdornerSpec),
}

pub(crate) fn render_adorner(spec: AdornerSpec, radius: f32) -> Option<Div> {
    match spec {
        AdornerSpec::FocusRing(focus_ring) => render_focus_ring_adorner(focus_ring, radius),
    }
}

/// Returns the outside layout extent required by the current optional adorner.
pub(crate) fn adorner_oversize_extent(spec: Option<AdornerSpec>) -> f32 {
    match spec {
        Some(AdornerSpec::FocusRing(focus_ring)) if matches!(focus_ring.placement, AdornerPlacement::Oversize) => {
            focus_ring.distance.max(0.0)
        }
        _ => 0.0,
    }
}

pub(crate) fn focus_ring_radius(spec: AdornerSpec, base_radius: f32) -> Option<f32> {
    match spec {
        AdornerSpec::FocusRing(focus_ring) => Some(match focus_ring.placement {
            AdornerPlacement::Inset => {
                (base_radius - focus_ring.distance.max(0.0) - focus_ring.width.max(0.0)).max(0.0)
            }
            AdornerPlacement::Oversize => base_radius + focus_ring.distance.max(0.0),
        }),
    }
}

pub(crate) fn render_adorner_with_focus_radius(spec: AdornerSpec, base_radius: f32) -> Option<Div> {
    let adorner = render_adorner(spec, base_radius)?;
    let radius = focus_ring_radius(spec, base_radius)?;
    Some(adorner.rounded(px(radius)))
}

/// Default single-adorner render path for controls using the current one-adorner policy.
pub(crate) fn render_optional_adorner(spec: Option<AdornerSpec>, base_radius: f32) -> Option<Div> {
    spec.and_then(|spec| render_adorner(spec, base_radius))
}

/// Variant of `render_optional_adorner` that also applies focus-radius shaping.
pub(crate) fn render_optional_adorner_with_focus_radius(spec: Option<AdornerSpec>, base_radius: f32) -> Option<Div> {
    spec.and_then(|spec| render_adorner_with_focus_radius(spec, base_radius))
}

fn render_focus_ring_adorner(focus_ring: FocusRingAdornerSpec, radius: f32) -> Option<Div> {
    match focus_ring.placement {
        AdornerPlacement::Inset => {
            render_inset_focus_ring_adorner(Some(focus_ring.color), radius, focus_ring.distance, focus_ring.width)
        }
        AdornerPlacement::Oversize => {
            render_oversize_focus_ring_adorner(Some(focus_ring.color), radius, focus_ring.distance, focus_ring.width)
        }
    }
}

/// Decorative inset focus ring drawn on top of the control root.
///
/// - Returns `None` when `color` is `None`.
/// - Uses inset geometry so decoration does not consume layout space.
/// - `gap` controls the distance between control edge and ring outer edge.
/// - `width` controls ring thickness (drawn inward from the ring outer edge).
fn render_inset_focus_ring_adorner(color: Option<Hsla>, radius: f32, gap: f32, width: f32) -> Option<Div> {
    let color = color?;
    let inset = gap.max(0.0);

    Some(
        div()
            .absolute()
            .top(px(inset))
            .right(px(inset))
            .bottom(px(inset))
            .left(px(inset))
            .border(px(width.max(0.0)))
            .border_color(color)
            .rounded(px((radius - inset).max(0.0))),
    )
}

/// Decorative oversize focus ring drawn around and outside the control root.
///
/// - Returns `None` when `color` is `None`.
/// - Uses negative absolute offsets to expand beyond the control bounds.
/// - `outset` controls how far the ring outer edge extends beyond the control edge.
/// - `width` controls ring thickness (drawn inward from the expanded edge).
///
/// Note: this variant can be clipped by ancestors with `overflow_hidden`.
fn render_oversize_focus_ring_adorner(color: Option<Hsla>, radius: f32, outset: f32, width: f32) -> Option<Div> {
    let color = color?;
    let outset = outset.max(0.0);

    Some(
        div()
            .absolute()
            .top(px(-outset))
            .right(px(-outset))
            .bottom(px(-outset))
            .left(px(-outset))
            .border(px(width.max(0.0)))
            .border_color(color)
            .rounded(px(radius + outset)),
    )
}
