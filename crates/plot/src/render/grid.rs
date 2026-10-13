use gpui::{Pixels, Window, point};

use crate::presentation::GridPresentation;
use super::{
    geometry::{CartesianGeometry, TickGeometry},
    line::paint_path,
};

pub(super) fn paint_grid(geometry: &CartesianGeometry, presentation: GridPresentation, window: &mut Window) {
    // Only interior ticks paint grid lines; axis visibility and placement don't affect them.
    if let Some(stroke) = presentation.vertical {
        for tick in interior_ticks(&geometry.x_ticks, geometry.plot_bounds.origin.x, geometry.plot_bounds.right()) {
            paint_path(
                [
                    point(tick.position, geometry.plot_bounds.origin.y),
                    point(tick.position, geometry.plot_bounds.bottom()),
                ],
                stroke,
                window,
            );
        }
    }
    if let Some(stroke) = presentation.horizontal {
        for tick in interior_ticks(&geometry.y_ticks, geometry.plot_bounds.origin.y, geometry.plot_bounds.bottom()) {
            paint_path(
                [
                    point(geometry.plot_bounds.origin.x, tick.position),
                    point(geometry.plot_bounds.right(), tick.position),
                ],
                stroke,
                window,
            );
        }
    }
}

fn interior_ticks(ticks: &[TickGeometry], start: Pixels, end: Pixels) -> impl Iterator<Item = &TickGeometry> {
    ticks.iter().filter(move |tick| tick.position > start && tick.position < end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    #[test]
    fn explicit_ticks_inside_bounds_all_generate_grid_lines() {
        let ticks = [10.0, 30.0, 70.0, 90.0]
            .map(|position| TickGeometry { position: px(position), label: position.to_string() });
        let positions: Vec<_> = interior_ticks(&ticks, px(0.0), px(100.0)).map(|tick| tick.position).collect();
        assert_eq!(positions, vec![px(10.0), px(30.0), px(70.0), px(90.0)]);
        assert_eq!(interior_ticks(&ticks, px(10.0), px(90.0)).count(), 2);
        assert_eq!(interior_ticks(&[], px(0.0), px(100.0)).count(), 0);
    }
}
