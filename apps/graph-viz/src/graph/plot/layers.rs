use gpui::{Bounds, Corners, Hsla, PathBuilder, Pixels, Window, fill, point, px, size};

use super::{PlotDomain2D, Viewport2D};
use crate::graph::metrics::value_at_time;

#[derive(Clone, Debug, PartialEq)]
pub struct GridLinesLayer {
    pub x_tick_values: Vec<f32>,
    pub y_tick_values: Vec<f32>,
    pub grid_color: Hsla,
}

impl Default for GridLinesLayer {
    fn default() -> Self {
        Self { x_tick_values: Vec::new(), y_tick_values: Vec::new(), grid_color: gpui::transparent_black() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DotsLayer {
    pub points: Vec<(f32, f32)>,
    pub color: Hsla,
    pub radius: Pixels,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinePathLayer {
    pub points: Vec<(f32, f32)>,
    pub stroke_width: Pixels,
    pub color: Hsla,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AreaFillLayer {
    pub points: Vec<(f32, f32)>,
    pub fill_color: Hsla,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InteractionCursorLayer {
    pub active_fraction: f32,
    pub cursor_color: Hsla,
    pub marker_color: Hsla,
    pub marker_samples: Vec<(f32, f32)>,
}

pub fn paint_background(window: &mut Window, bounds: Bounds<Pixels>, color: Hsla) {
    window.paint_quad(fill(bounds, color).corner_radii(Corners::all(px(6.0))));
}

pub fn paint_grid_lines(
    window: &mut Window,
    x_domain: &PlotDomain2D,
    y_domain: &PlotDomain2D,
    viewport: &Viewport2D,
    layer: &GridLinesLayer,
    y_inverted: bool,
    zero_baseline_color: Option<Hsla>,
) {
    for &y_value in &layer.y_tick_values {
        let fraction = y_fraction(y_domain, y_value, y_inverted);
        let y = viewport.uv_to_pixels(gpui::point(0.0, fraction)).y;
        let color = if zero_baseline_color.is_some() && y_value.abs() < f32::EPSILON {
            zero_baseline_color.unwrap()
        } else {
            layer.grid_color
        };
        let thickness = if zero_baseline_color.is_some() && y_value.abs() < f32::EPSILON {
            px(1.5)
        } else {
            px(1.0)
        };
        paint_horizontal_line(window, viewport.bounds.left(), viewport.bounds.right(), y, color, thickness);
    }

    let _ = x_domain;
}

pub fn paint_plot_frame(window: &mut Window, viewport: &Viewport2D, border_color: Hsla) {
    paint_vertical_line(
        window,
        viewport.bounds.left(),
        viewport.bounds.top(),
        viewport.bounds.bottom(),
        border_color,
        px(1.0),
    );
    paint_vertical_line(
        window,
        viewport.bounds.right(),
        viewport.bounds.top(),
        viewport.bounds.bottom(),
        border_color,
        px(1.0),
    );
    paint_horizontal_line_legacy(
        window,
        viewport.bounds.left(),
        viewport.bounds.right(),
        viewport.bounds.top(),
        border_color,
    );
    paint_horizontal_line_legacy(
        window,
        viewport.bounds.left(),
        viewport.bounds.right(),
        viewport.bounds.bottom(),
        border_color,
    );
}

pub fn paint_area_fill(
    window: &mut Window,
    x_domain: &PlotDomain2D,
    y_domain: &PlotDomain2D,
    viewport: &Viewport2D,
    layer: &AreaFillLayer,
    y_inverted: bool,
) {
    if layer.points.len() < 2 {
        return;
    }

    let baseline_y = *y_domain.y_range.start();
    let mut builder = PathBuilder::fill();
    let mut points = layer.points.iter();
    let Some(first) = points.next() else {
        return;
    };

    let first_pixel = combined_data_to_pixels(x_domain, y_domain, viewport, first.0, first.1, y_inverted);
    builder.move_to(first_pixel);
    for &(x, y) in points {
        builder.line_to(combined_data_to_pixels(x_domain, y_domain, viewport, x, y, y_inverted));
    }

    let last = layer.points[layer.points.len() - 1];
    let baseline_last = combined_data_to_pixels(x_domain, y_domain, viewport, last.0, baseline_y, y_inverted);
    let baseline_first = combined_data_to_pixels(x_domain, y_domain, viewport, first.0, baseline_y, y_inverted);
    builder.line_to(baseline_last);
    builder.line_to(baseline_first);
    builder.close();

    if let Ok(path) = builder.build() {
        window.paint_path(path, layer.fill_color);
    }
}

pub fn paint_line_path(
    window: &mut Window,
    x_domain: &PlotDomain2D,
    y_domain: &PlotDomain2D,
    viewport: &Viewport2D,
    layer: &LinePathLayer,
    y_inverted: bool,
) {
    if layer.points.len() < 2 {
        return;
    }

    let mut builder = PathBuilder::stroke(layer.stroke_width);
    let mut points = layer.points.iter();
    let Some(first) = points.next() else {
        return;
    };

    builder.move_to(combined_data_to_pixels(x_domain, y_domain, viewport, first.0, first.1, y_inverted));
    for &(x, y) in points {
        builder.line_to(combined_data_to_pixels(x_domain, y_domain, viewport, x, y, y_inverted));
    }

    if let Ok(path) = builder.build() {
        window.paint_path(path, layer.color);
    }
}

pub fn paint_dot_scatter(
    window: &mut Window,
    x_domain: &PlotDomain2D,
    y_domain: &PlotDomain2D,
    viewport: &Viewport2D,
    layer: &DotsLayer,
    y_inverted: bool,
) {
    if layer.points.is_empty() {
        return;
    }

    let diameter = layer.radius * 2.0;
    for &(x, y) in &layer.points {
        let center = combined_data_to_pixels(x_domain, y_domain, viewport, x, y, y_inverted);
        let bounds =
            Bounds { origin: point(center.x - layer.radius, center.y - layer.radius), size: size(diameter, diameter) };
        window.paint_quad(fill(bounds, layer.color).corner_radii(Corners::all(layer.radius)));
    }
}

pub fn paint_interaction_cursor(
    window: &mut Window,
    x_domain: &PlotDomain2D,
    y_domain: &PlotDomain2D,
    viewport: &Viewport2D,
    layer: &InteractionCursorLayer,
    y_inverted: bool,
) {
    let fraction = layer.active_fraction.clamp(0.0, 1.0);
    let scrub_time = x_domain.x_from_fraction(fraction);
    let x = viewport.uv_to_pixels(gpui::point(x_domain.x_to_fraction(scrub_time), 0.0)).x;
    paint_vertical_line(window, x, viewport.bounds.top(), viewport.bounds.bottom(), layer.cursor_color, px(2.0));

    let Some(value) = value_at_time(&layer.marker_samples, scrub_time) else {
        return;
    };

    let marker = combined_data_to_pixels(x_domain, y_domain, viewport, scrub_time, value, y_inverted);
    let marker_bounds = Bounds { origin: point(marker.x - px(4.0), marker.y - px(4.0)), size: size(px(8.0), px(8.0)) };
    window.paint_quad(fill(marker_bounds, layer.marker_color).corner_radii(Corners::all(px(4.0))));
}

fn combined_data_to_pixels(
    x_domain: &PlotDomain2D,
    y_domain: &PlotDomain2D,
    viewport: &Viewport2D,
    x: f32,
    y: f32,
    y_inverted: bool,
) -> gpui::Point<Pixels> {
    let uv = gpui::point(x_domain.x_to_fraction(x), y_fraction(y_domain, y, y_inverted));
    viewport.uv_to_pixels(uv)
}

fn y_fraction(y_domain: &PlotDomain2D, y: f32, invert: bool) -> f32 {
    let fraction = y_domain.y_to_fraction(y);
    if invert { 1.0 - fraction } else { fraction }
}

fn paint_horizontal_line(window: &mut Window, left: Pixels, right: Pixels, y: Pixels, color: Hsla, thickness: Pixels) {
    let half = thickness / 2.0;
    let line = Bounds { origin: point(left, y - half), size: size(right - left, thickness) };
    window.paint_quad(fill(line, color).corner_radii(Corners::all(half)));
}

fn paint_horizontal_line_legacy(window: &mut Window, left: Pixels, right: Pixels, y: Pixels, color: Hsla) {
    paint_horizontal_line(window, left, right, y, color, px(1.0));
}

fn paint_vertical_line(window: &mut Window, x: Pixels, top: Pixels, bottom: Pixels, color: Hsla, thickness: Pixels) {
    let half = thickness / 2.0;
    let line = Bounds { origin: point(x - half, top), size: size(thickness, bottom - top) };
    window.paint_quad(fill(line, color).corner_radii(Corners::all(half)));
}
