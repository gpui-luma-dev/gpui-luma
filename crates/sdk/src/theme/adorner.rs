use gpui::{Div, Hsla, div, px, prelude::*};

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
