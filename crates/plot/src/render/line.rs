use gpui::{Bounds, PathBuilder, Pixels, Window, fill, point, px, size};

use crate::{
    plots::Interpolation,
    presentation::{LinePresentation, MarkerVisibility, Stroke},
};
use super::axis::positive;

pub(super) fn paint_path(points: impl IntoIterator<Item = gpui::Point<Pixels>>, stroke: Stroke, window: &mut Window) {
    let mut points = points.into_iter();
    let Some(first) = points.next() else { return };
    let width = positive(stroke.width);
    if width == px(0.0) {
        return;
    }
    let mut path = PathBuilder::stroke(width);
    path.move_to(first);
    for point in points {
        path.line_to(point);
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, stroke.color);
    }
}

pub(super) fn paint_line(
    runs: &[Vec<gpui::Point<Pixels>>],
    presentation: LinePresentation,
    interpolation: Interpolation,
    hovered: Option<gpui::Point<Pixels>>,
    window: &mut Window,
) {
    let width = positive(presentation.stroke.width);
    if width > px(0.0) {
        for run in runs.iter().filter(|run| run.len() > 1) {
            let mut path = PathBuilder::stroke(width);
            append_boundary(&mut path, run, interpolation);
            if let Ok(path) = path.build() {
                window.paint_path(path, presentation.stroke.color);
            }
        }
    }
    let Some(marker) = presentation.markers else { return };
    let radius = positive(marker.radius);
    if radius == px(0.0) {
        return;
    }
    for sample in marker_points(runs, marker.visibility, hovered) {
        let bounds = Bounds::new(*sample - point(radius, radius), size(radius * 2.0, radius * 2.0));
        window.paint_quad(fill(bounds, marker.fill).corner_radii(radius));
    }
}

/// Append the identical boundary for strokes and filled paths.
pub(super) fn append_boundary(path: &mut PathBuilder, run: &[gpui::Point<Pixels>], interpolation: Interpolation) {
    let Some(first) = run.first() else { return };
    path.move_to(*first);
    if interpolation == Interpolation::Monotone
        && let Some(segments) = super::interpolation::monotone(run)
    {
        for segment in segments {
            path.cubic_bezier_to(segment.end, segment.control_a, segment.control_b);
        }
    } else {
        for point in &run[1..] {
            path.line_to(*point);
        }
    }
}

fn marker_points(
    runs: &[Vec<gpui::Point<Pixels>>],
    visibility: MarkerVisibility,
    hovered: Option<gpui::Point<Pixels>>,
) -> impl Iterator<Item = &gpui::Point<Pixels>> {
    runs.iter()
        .flatten()
        .filter(move |sample| visibility == MarkerVisibility::Always || Some(**sample) == hovered)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marker_modes_show_every_observation_or_exactly_the_hovered_one() {
        let a = point(px(0.0), px(1.0));
        let b = point(px(2.0), px(3.0));
        let c = point(px(4.0), px(1.0));
        let runs = [vec![a, b], vec![c]];
        assert_eq!(marker_points(&runs, MarkerVisibility::Always, Some(b)).copied().collect::<Vec<_>>(), [a, b, c]);
        assert_eq!(marker_points(&runs, MarkerVisibility::Hover, Some(b)).copied().collect::<Vec<_>>(), [b]);
        assert_eq!(marker_points(&runs, MarkerVisibility::Hover, None).count(), 0);
    }
}
