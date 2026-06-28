use gpui::{
    Bounds, Corners, Edges, Hsla, PaintQuad, Pixels, Window, linear_color_stop, linear_gradient, px, transparent_black,
};

use super::color::interpolate_rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientType {
    Linear,
}

const EDGE_EPSILON: f32 = 0.001;

pub fn color_at_position(stops: &[(f32, Hsla)], position: f32) -> Hsla {
    if stops.is_empty() {
        return gpui::hsla(0.0, 0.0, 0.0, 1.0);
    }
    if stops.len() == 1 {
        return stops[0].1;
    }

    let position = position.clamp(0.0, 1.0);
    if position <= stops[0].0 {
        return stops[0].1;
    }
    if position >= stops[stops.len() - 1].0 {
        return stops[stops.len() - 1].1;
    }

    for window in stops.windows(2) {
        let (left_pos, left_color) = window[0];
        let (right_pos, right_color) = window[1];
        if position >= left_pos && position <= right_pos {
            let span = (right_pos - left_pos).max(f32::EPSILON);
            let t = (position - left_pos) / span;
            return interpolate_rgb(left_color, right_color, t);
        }
    }

    stops[stops.len() - 1].1
}

pub fn sorted_stops(stops: &[(f32, Hsla)]) -> Vec<(f32, Hsla)> {
    let mut sorted = stops.to_vec();
    sorted.sort_by(|left, right| left.0.total_cmp(&right.0));
    sorted
}

fn horizontal_edge_corner_radii(radii: Corners<Pixels>, touches_left: bool, touches_right: bool) -> Corners<Pixels> {
    Corners {
        top_left: if touches_left { radii.top_left } else { px(0.0) },
        bottom_left: if touches_left { radii.bottom_left } else { px(0.0) },
        top_right: if touches_right { radii.top_right } else { px(0.0) },
        bottom_right: if touches_right { radii.bottom_right } else { px(0.0) },
    }
}

fn vertical_edge_corner_radii(radii: Corners<Pixels>, touches_top: bool, touches_bottom: bool) -> Corners<Pixels> {
    Corners {
        top_left: if touches_top { radii.top_left } else { px(0.0) },
        top_right: if touches_top { radii.top_right } else { px(0.0) },
        bottom_left: if touches_bottom { radii.bottom_left } else { px(0.0) },
        bottom_right: if touches_bottom { radii.bottom_right } else { px(0.0) },
    }
}

pub fn paint_horizontal_gradient_track(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    stops: &[(f32, Hsla)],
    reverse: bool,
    corner_radii: Corners<Pixels>,
) {
    let stops = sorted_stops(stops);
    if stops.is_empty() {
        return;
    }

    if stops.len() == 1 {
        paint_solid(window, bounds, stops[0].1, horizontal_edge_corner_radii(corner_radii, true, true));
        return;
    }

    let width = bounds.size.width.as_f32();
    if width <= 0.0 {
        return;
    }

    let (first_pos, first_color, tail_pos, tail_color, iter) = if reverse {
        let last = stops[stops.len() - 1];
        let first = stops[0];
        let reversed = stops
            .windows(2)
            .rev()
            .map(|pair| (1.0 - pair[1].0, pair[1].1, 1.0 - pair[0].0, pair[0].1))
            .collect::<Vec<_>>();
        (1.0 - last.0, last.1, 1.0 - first.0, first.1, reversed)
    } else {
        let first = stops[0];
        let last = stops[stops.len() - 1];
        let forward = stops.windows(2).map(|pair| (pair[0].0, pair[0].1, pair[1].0, pair[1].1)).collect::<Vec<_>>();
        (first.0, first.1, last.0, last.1, forward)
    };

    if first_pos > 0.0 {
        let segment_end = bounds.origin.x + bounds.size.width * first_pos;
        paint_solid(
            window,
            Bounds { origin: bounds.origin, size: gpui::size(segment_end - bounds.origin.x, bounds.size.height) },
            first_color,
            horizontal_edge_corner_radii(corner_radii, true, false),
        );
    }

    for (start_pos, start_color, end_pos, end_color) in iter {
        let segment_start = bounds.origin.x + bounds.size.width * start_pos;
        let segment_end = bounds.origin.x + bounds.size.width * end_pos;
        if segment_end <= segment_start {
            continue;
        }
        let segment_bounds = Bounds {
            origin: gpui::point(segment_start, bounds.origin.y),
            size: gpui::size(segment_end - segment_start, bounds.size.height),
        };
        let touches_left = start_pos <= EDGE_EPSILON;
        let touches_right = end_pos >= 1.0 - EDGE_EPSILON;

        window.paint_quad(PaintQuad {
            bounds: segment_bounds,
            corner_radii: horizontal_edge_corner_radii(corner_radii, touches_left, touches_right),
            background: linear_gradient(90.0, linear_color_stop(start_color, 0.0), linear_color_stop(end_color, 1.0)),
            border_widths: Edges::default(),
            border_color: transparent_black(),
            border_style: gpui::BorderStyle::default(),
        });
    }

    if tail_pos < 1.0 {
        let segment_start = bounds.origin.x + bounds.size.width * tail_pos;
        paint_solid(
            window,
            Bounds {
                origin: gpui::point(segment_start, bounds.origin.y),
                size: gpui::size(bounds.origin.x + bounds.size.width - segment_start, bounds.size.height),
            },
            tail_color,
            horizontal_edge_corner_radii(corner_radii, false, true),
        );
    }
}

pub fn paint_linear_gradient_preview(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    stops: &[(f32, Hsla)],
    rotation_deg: f32,
    corner_radii: Corners<Pixels>,
) {
    let stops = sorted_stops(stops);
    if stops.is_empty() {
        return;
    }

    if stops.len() == 1 {
        paint_solid(window, bounds, stops[0].1, corner_radii);
        return;
    }

    if stops.len() == 2 {
        let (start_pos, start_color) = stops[0];
        let (end_pos, end_color) = stops[1];
        window.paint_quad(PaintQuad {
            bounds,
            corner_radii,
            background: linear_gradient(
                rotation_deg,
                linear_color_stop(start_color, start_pos),
                linear_color_stop(end_color, end_pos),
            ),
            border_widths: Edges::default(),
            border_color: transparent_black(),
            border_style: gpui::BorderStyle::default(),
        });
        return;
    }

    if rotation_deg.rem_euclid(180.0).abs() < f32::EPSILON {
        paint_vertical_gradient_preview(window, bounds, &stops, rotation_deg, corner_radii);
        return;
    }

    if (rotation_deg - 270.0).rem_euclid(360.0).abs() < f32::EPSILON {
        paint_horizontal_gradient_track(window, bounds, &stops, true, corner_radii);
        return;
    }

    if (rotation_deg - 90.0).rem_euclid(360.0).abs() < f32::EPSILON {
        paint_horizontal_gradient_track(window, bounds, &stops, false, corner_radii);
        return;
    }

    paint_rasterized_gradient_preview(window, bounds, &stops, rotation_deg, corner_radii);
}

fn paint_vertical_gradient_preview(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    stops: &[(f32, Hsla)],
    rotation_deg: f32,
    corner_radii: Corners<Pixels>,
) {
    if stops.len() == 1 {
        paint_solid(window, bounds, stops[0].1, vertical_edge_corner_radii(corner_radii, true, true));
        return;
    }

    let height = bounds.size.height.as_f32();
    if height <= 0.0 {
        return;
    }

    let reverse = (rotation_deg - 180.0).abs() < f32::EPSILON;
    let (first_pos, first_color) = if reverse {
        let last = stops[stops.len() - 1];
        (1.0 - last.0, last.1)
    } else {
        stops[0]
    };
    if first_pos > 0.0 {
        let segment_end = bounds.origin.y + bounds.size.height * first_pos;
        paint_solid(
            window,
            Bounds { origin: bounds.origin, size: gpui::size(bounds.size.width, segment_end - bounds.origin.y) },
            first_color,
            vertical_edge_corner_radii(corner_radii, true, false),
        );
    }

    for pair in stops.windows(2) {
        let (start_pos, start_color) = pair[0];
        let (end_pos, end_color) = pair[1];
        let (start_pos, end_pos, start_color, end_color) = if reverse {
            (1.0 - end_pos, 1.0 - start_pos, end_color, start_color)
        } else {
            (start_pos, end_pos, start_color, end_color)
        };
        let segment_start = bounds.origin.y + bounds.size.height * start_pos;
        let segment_end = bounds.origin.y + bounds.size.height * end_pos;
        let segment_bounds = Bounds {
            origin: gpui::point(bounds.origin.x, segment_start),
            size: gpui::size(bounds.size.width, segment_end - segment_start),
        };
        let touches_top = start_pos <= EDGE_EPSILON;
        let touches_bottom = end_pos >= 1.0 - EDGE_EPSILON;

        window.paint_quad(PaintQuad {
            bounds: segment_bounds,
            corner_radii: vertical_edge_corner_radii(corner_radii, touches_top, touches_bottom),
            background: linear_gradient(180.0, linear_color_stop(start_color, 0.0), linear_color_stop(end_color, 1.0)),
            border_widths: Edges::default(),
            border_color: transparent_black(),
            border_style: gpui::BorderStyle::default(),
        });
    }

    let (last_pos, last_color) = if reverse {
        let first = stops[0];
        (1.0 - first.0, first.1)
    } else {
        stops[stops.len() - 1]
    };
    if last_pos < 1.0 {
        let segment_start = bounds.origin.y + bounds.size.height * last_pos;
        paint_solid(
            window,
            Bounds {
                origin: gpui::point(bounds.origin.x, segment_start),
                size: gpui::size(bounds.size.width, bounds.origin.y + bounds.size.height - segment_start),
            },
            last_color,
            vertical_edge_corner_radii(corner_radii, false, true),
        );
    }
}

fn paint_rasterized_gradient_preview(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    stops: &[(f32, Hsla)],
    rotation_deg: f32,
    corner_radii: Corners<Pixels>,
) {
    let width = bounds.size.width.as_f32();
    let height = bounds.size.height.as_f32();
    if width <= 0.0 || height <= 0.0 {
        return;
    }

    let radians = rotation_deg.to_radians();
    let direction = (radians.sin(), -radians.cos());
    let corners = [(0.0_f32, 0.0_f32), (1.0_f32, 0.0_f32), (0.0_f32, 1.0_f32), (1.0_f32, 1.0_f32)];
    let mut min_projection = f32::INFINITY;
    let mut max_projection = f32::NEG_INFINITY;
    for (x, y) in corners {
        let projection = x * direction.0 + y * direction.1;
        min_projection = min_projection.min(projection);
        max_projection = max_projection.max(projection);
    }
    let projection_span = (max_projection - min_projection).max(f32::EPSILON);

    let columns = ((width / 8.0).ceil() as usize).clamp(24, 160);
    let rows = ((height / 8.0).ceil() as usize).clamp(16, 96);

    for row in 0..rows {
        let y0 = row as f32 / rows as f32;
        let y1 = (row + 1) as f32 / rows as f32;
        for column in 0..columns {
            let x0 = column as f32 / columns as f32;
            let x1 = (column + 1) as f32 / columns as f32;
            let sample_x = (x0 + x1) * 0.5;
            let sample_y = (y0 + y1) * 0.5;
            let projection = sample_x * direction.0 + sample_y * direction.1;
            let sample = ((projection - min_projection) / projection_span).clamp(0.0, 1.0);
            let color = color_at_position(stops, sample);

            let left = bounds.origin.x + bounds.size.width * x0;
            let right = bounds.origin.x + bounds.size.width * x1;
            let top = bounds.origin.y + bounds.size.height * y0;
            let bottom = bounds.origin.y + bounds.size.height * y1;
            let segment_bounds =
                Bounds { origin: gpui::point(left, top), size: gpui::size(right - left, bottom - top) };
            let radii = Corners {
                top_left: if row == 0 && column == 0 {
                    corner_radii.top_left
                } else {
                    px(0.0)
                },
                top_right: if row == 0 && column + 1 == columns {
                    corner_radii.top_right
                } else {
                    px(0.0)
                },
                bottom_left: if row + 1 == rows && column == 0 {
                    corner_radii.bottom_left
                } else {
                    px(0.0)
                },
                bottom_right: if row + 1 == rows && column + 1 == columns {
                    corner_radii.bottom_right
                } else {
                    px(0.0)
                },
            };
            paint_solid(window, segment_bounds, color, radii);
        }
    }
}

fn paint_solid(window: &mut Window, bounds: Bounds<Pixels>, color: Hsla, corner_radii: Corners<Pixels>) {
    window.paint_quad(PaintQuad {
        bounds,
        corner_radii,
        background: color.into(),
        border_widths: Edges::default(),
        border_color: transparent_black(),
        border_style: gpui::BorderStyle::default(),
    });
}
