use gpui::{PathBuilder, Pixels, Point, Window, point};
use crate::{plots::Interpolation, presentation::AreaPresentation};
use super::{
    geometry::AreaGeometry,
    line::{append_boundary, paint_line},
};

pub(super) fn paint_area(
    area: &AreaGeometry,
    style: AreaPresentation,
    hovered: Option<Point<Pixels>>,
    window: &mut Window,
) {
    for run in &area.boundary.runs {
        if let Some(path) = filled_path(run, area.baseline, area.boundary.interpolation) {
            window.paint_path(path, style.fill);
        }
    }
    if let Some(outline) = style.outline {
        paint_line(&area.boundary.runs, outline, area.boundary.interpolation, hovered, window);
    }
}

fn filled_path(run: &[Point<Pixels>], baseline: Pixels, interpolation: Interpolation) -> Option<gpui::Path<Pixels>> {
    if run.len() < 2 {
        return None;
    }
    let mut path = PathBuilder::fill();
    append_boundary(&mut path, run, interpolation);
    path.line_to(point(run.last()?.x, baseline));
    path.line_to(point(run.first()?.x, baseline));
    path.close();
    path.build().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    #[test]
    fn positive_negative_and_curved_regions_close_at_the_baseline() {
        for interpolation in [Interpolation::Linear, Interpolation::Monotone] {
            for y in [-10.0, 10.0] {
                let run = [point(px(0.0), px(y)), point(px(10.0), px(y * 0.5)), point(px(30.0), px(y))];
                let path = filled_path(&run, px(0.0), interpolation).expect("filled area");
                assert!(path.bounds.top() <= px(y.min(0.0)));
                assert!(path.bounds.bottom() >= px(y.max(0.0)));
                assert!(path.bounds.left() <= px(0.0) && path.bounds.right() >= px(30.0));
            }
        }
        assert!(filled_path(&[], px(0.0), Interpolation::Linear).is_none());
        assert!(filled_path(&[point(px(0.0), px(2.0))], px(0.0), Interpolation::Linear).is_none());
    }
}
