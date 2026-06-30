mod axes;
mod decimate;
mod domain;
mod layers;
mod layout;
mod overlay;
mod ticks;
mod viewport;
mod x_axis;

pub use axes::{AxisLabelStyle, AxisLabelsLayer, RightAxisLabelsLayer, paint_axis_labels, paint_right_axis_labels};
pub use decimate::{minmax_decimate, uniform_subsample};
pub use domain::PlotDomain2D;
pub use layers::{
    AreaFillLayer, DotsLayer, GridLinesLayer, InteractionCursorLayer, LinePathLayer, paint_area_fill, paint_background,
    paint_dot_scatter, paint_grid_lines, paint_interaction_cursor, paint_line_path, paint_plot_frame,
};
pub use layout::build_telemetry_chart_stack;
pub use overlay::build_speed_hr_overlay_stack;
pub use x_axis::format_x_axis_tick;
pub use viewport::{ChartMargins, Viewport2D};

use gpui::{App, Bounds, Pixels, Window};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeriesStyle {
    Line,
    AreaFilled,
    Dots,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChartStack {
    pub background_color: gpui::Hsla,
    pub x_domain: PlotDomain2D,
    pub y_domain: PlotDomain2D,
    pub margins: ChartMargins,
    pub y_inverted: bool,
    pub zero_baseline_color: Option<gpui::Hsla>,
    pub grid: GridLinesLayer,
    pub axes: AxisLabelsLayer,
    pub right_axes: Option<RightAxisLabelsLayer>,
    pub area_fill: Option<AreaFillLayer>,
    pub dots: Option<DotsLayer>,
    pub overlay_dots: Option<DotsLayer>,
    pub line: LinePathLayer,
    pub cursor: InteractionCursorLayer,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DualAxisOverlayStack {
    pub background_color: gpui::Hsla,
    pub x_domain: PlotDomain2D,
    pub primary_y_domain: PlotDomain2D,
    pub overlay_y_domain: PlotDomain2D,
    pub margins: ChartMargins,
    pub grid: GridLinesLayer,
    pub left_axes: AxisLabelsLayer,
    pub right_axes: RightAxisLabelsLayer,
    pub area_fill: AreaFillLayer,
    pub base_line: LinePathLayer,
    pub overlay_line: LinePathLayer,
    pub cursor: InteractionCursorLayer,
}

pub fn paint_chart_stack(window: &mut Window, cx: &mut App, bounds: Bounds<Pixels>, stack: &ChartStack) {
    paint_background(window, bounds, stack.background_color);

    let viewport = Viewport2D::with_margins(bounds, stack.margins);
    paint_plot_frame(window, &viewport, stack.grid.grid_color);
    paint_grid_lines(
        window,
        &stack.x_domain,
        &stack.y_domain,
        &viewport,
        &stack.grid,
        stack.y_inverted,
        stack.zero_baseline_color,
    );
    if let Some(area_fill) = &stack.area_fill {
        paint_area_fill(window, &stack.x_domain, &stack.y_domain, &viewport, area_fill, stack.y_inverted);
    }
    if let Some(dots) = &stack.dots {
        paint_dot_scatter(window, &stack.x_domain, &stack.y_domain, &viewport, dots, stack.y_inverted);
    }
    if let Some(overlay_dots) = &stack.overlay_dots {
        paint_dot_scatter(window, &stack.x_domain, &stack.y_domain, &viewport, overlay_dots, stack.y_inverted);
    }
    if stack.dots.is_none() {
        paint_line_path(window, &stack.x_domain, &stack.y_domain, &viewport, &stack.line, stack.y_inverted);
    }
    paint_interaction_cursor(window, &stack.x_domain, &stack.y_domain, &viewport, &stack.cursor, stack.y_inverted);
    paint_axis_labels(window, cx, &viewport, &stack.x_domain, &stack.y_domain, &stack.axes);
    if let Some(right_axes) = &stack.right_axes {
        paint_right_axis_labels(window, cx, &viewport, &stack.y_domain, right_axes);
    }
}

pub fn paint_dual_axis_overlay_stack(
    window: &mut Window,
    cx: &mut App,
    bounds: Bounds<Pixels>,
    stack: &DualAxisOverlayStack,
) {
    paint_background(window, bounds, stack.background_color);

    let viewport = Viewport2D::with_margins(bounds, stack.margins);
    paint_plot_frame(window, &viewport, stack.grid.grid_color);
    paint_grid_lines(window, &stack.x_domain, &stack.primary_y_domain, &viewport, &stack.grid, false, None);
    paint_area_fill(window, &stack.x_domain, &stack.primary_y_domain, &viewport, &stack.area_fill, false);
    paint_line_path(window, &stack.x_domain, &stack.primary_y_domain, &viewport, &stack.base_line, false);
    paint_line_path(window, &stack.x_domain, &stack.overlay_y_domain, &viewport, &stack.overlay_line, false);
    paint_interaction_cursor(window, &stack.x_domain, &stack.primary_y_domain, &viewport, &stack.cursor, false);
    paint_axis_labels(window, cx, &viewport, &stack.x_domain, &stack.primary_y_domain, &stack.left_axes);
    paint_right_axis_labels(window, cx, &viewport, &stack.overlay_y_domain, &stack.right_axes);
}
