use gpui::{
    Bounds, Corners, Div, FontWeight, Hsla, ObjectFit, PathBuilder, Pixels, Window, canvas, div, fill, hsla, img,
    point, prelude::*, px, size,
};

const DISPLAY_WIDTH: f32 = 246.0;
const DISPLAY_HEIGHT: f32 = 148.0;
const DISPLAY_RADIUS: f32 = 22.0;
const ANALYZER_SHELL_ASSET: &str = "assets/inset-box.png";
const CHART_LEFT: f32 = 24.0;
const CHART_RIGHT: f32 = 30.0;
const CHART_TOP: f32 = 22.0;
const CHART_BOTTOM: f32 = 30.0;

const ORANGE_TRACE: &[(f32, f32)] = &[
    (0.00, 0.53),
    (0.06, 0.52),
    (0.12, 0.48),
    (0.18, 0.38),
    (0.26, 0.38),
    (0.32, 0.54),
    (0.36, 0.42),
    (0.43, 0.66),
    (0.50, 0.49),
    (0.56, 0.58),
    (0.64, 0.44),
    (0.70, 0.62),
    (0.78, 0.48),
    (0.86, 0.36),
    (0.92, 0.36),
    (1.00, 0.52),
];

const GRAY_TRACE: &[(f32, f32)] = &[
    (0.00, 0.74),
    (0.08, 0.76),
    (0.16, 0.86),
    (0.22, 0.80),
    (0.30, 0.94),
    (0.36, 0.72),
    (0.42, 0.88),
    (0.50, 0.80),
    (0.58, 0.92),
    (0.66, 0.75),
    (0.74, 0.86),
    (0.82, 0.78),
    (0.90, 0.88),
    (1.00, 0.82),
];

pub fn fake_analyzer_display() -> Div {
    div()
        .relative()
        .w(px(DISPLAY_WIDTH))
        .h(px(DISPLAY_HEIGHT))
        .rounded(px(DISPLAY_RADIUS))
        .overflow_hidden()
        .child(
            img(ANALYZER_SHELL_ASSET)
                .absolute()
                .left(px(0.0))
                .top(px(0.0))
                .size_full()
                .object_fit(ObjectFit::Fill)
                .with_fallback(|| {
                    div()
                        .size_full()
                        .bg(hsla(220.0 / 360.0, 0.08, 0.84, 0.95))
                        .border_1()
                        .border_color(hsla(220.0 / 360.0, 0.08, 0.44, 0.86))
                        .rounded(px(DISPLAY_RADIUS))
                        .into_any_element()
                }),
        )
        .child(
            canvas(|_, _, _| {}, |bounds, _, window, _| paint_analyzer(bounds, window))
                .absolute()
                .left(px(0.0))
                .top(px(0.0))
                .size_full(),
        )
        .child(y_axis_label("+12", 16.0, 34.0))
        .child(y_axis_label("0", 34.0, 68.0))
        .child(y_axis_label("-12", 18.0, 101.0))
        .child(x_axis_label("100Hz", 62.0))
        .child(x_axis_label("500Hz", 118.0))
        .child(x_axis_label("2kHz", 170.0))
        .child(x_axis_label("10kHz", 210.0))
}

fn y_axis_label(text: &'static str, left: f32, top: f32) -> Div {
    div()
        .absolute()
        .left(px(left))
        .top(px(top))
        .text_size(px(9.0))
        .line_height(px(10.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(hsla(220.0 / 360.0, 0.08, 0.42, 0.88))
        .child(text)
}

fn x_axis_label(text: &'static str, center_x: f32) -> Div {
    div()
        .absolute()
        .left(px(center_x - 18.0))
        .bottom(px(12.0))
        .w(px(36.0))
        .text_size(px(9.0))
        .line_height(px(10.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(hsla(220.0 / 360.0, 0.08, 0.40, 0.88))
        .text_center()
        .child(text)
}

fn paint_analyzer(bounds: Bounds<Pixels>, window: &mut Window) {
    let chart_bounds = Bounds {
        origin: point(bounds.left() + px(CHART_LEFT), bounds.top() + px(CHART_TOP)),
        size: size(bounds.size.width - px(CHART_LEFT + CHART_RIGHT), bounds.size.height - px(CHART_TOP + CHART_BOTTOM)),
    };

    paint_vertical_guides(window, chart_bounds);
    paint_horizontal_guides(window, chart_bounds);
    paint_zero_line(window, chart_bounds);
    paint_wave_fill(window, chart_bounds);
    paint_trace(window, chart_bounds, GRAY_TRACE, hsla(220.0 / 360.0, 0.08, 0.56, 0.90), px(1.4));
    paint_trace(window, chart_bounds, ORANGE_TRACE, hsla(20.0 / 360.0, 0.62, 0.43, 0.98), px(2.0));
}

fn paint_vertical_guides(window: &mut Window, chart_bounds: Bounds<Pixels>) {
    for fraction in [0.20_f32, 0.42, 0.68, 0.88] {
        let x = chart_bounds.left() + chart_bounds.size.width * fraction;
        let guide =
            Bounds { origin: point(x - px(0.5), chart_bounds.top()), size: size(px(1.0), chart_bounds.size.height) };
        window.paint_quad(fill(guide, hsla(220.0 / 360.0, 0.08, 0.54, 0.52)).corner_radii(Corners::all(px(0.5))));
    }
}

fn paint_horizontal_guides(window: &mut Window, chart_bounds: Bounds<Pixels>) {
    for fraction in [0.24_f32, 0.76] {
        let y = chart_bounds.top() + chart_bounds.size.height * fraction;
        paint_dashed_horizontal(window, chart_bounds.left(), chart_bounds.right(), y, px(1.6), px(4.5), px(3.5));
    }
}

fn paint_zero_line(window: &mut Window, chart_bounds: Bounds<Pixels>) {
    let y = chart_bounds.top() + chart_bounds.size.height * 0.50;
    let line = Bounds { origin: point(chart_bounds.left(), y - px(0.9)), size: size(chart_bounds.size.width, px(1.8)) };
    window.paint_quad(fill(line, hsla(220.0 / 360.0, 0.08, 0.38, 0.70)).corner_radii(Corners::all(px(0.9))));
}

fn paint_wave_fill(window: &mut Window, chart_bounds: Bounds<Pixels>) {
    let mut builder = PathBuilder::fill();
    let base_y = chart_bounds.bottom() - px(6.0);

    let start = to_chart_point(chart_bounds, GRAY_TRACE[0], 0.14);
    builder.move_to(point(start.x, base_y));
    builder.line_to(start);

    for sample in GRAY_TRACE.iter().skip(1) {
        builder.line_to(to_chart_point(chart_bounds, *sample, 0.14));
    }

    let end = to_chart_point(chart_bounds, *GRAY_TRACE.last().unwrap_or(&GRAY_TRACE[0]), 0.14);
    builder.line_to(point(end.x, base_y));
    builder.close();

    if let Ok(path) = builder.build() {
        window.paint_path(path, hsla(220.0 / 360.0, 0.08, 0.74, 0.66));
    }
}

fn paint_trace(
    window: &mut Window,
    chart_bounds: Bounds<Pixels>,
    samples: &[(f32, f32)],
    color: Hsla,
    stroke_width: Pixels,
) {
    let mut builder = PathBuilder::stroke(stroke_width);
    let mut points = samples.iter();
    let Some(first) = points.next() else {
        return;
    };

    builder.move_to(to_chart_point(chart_bounds, *first, 0.0));
    for sample in points {
        builder.line_to(to_chart_point(chart_bounds, *sample, 0.0));
    }

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

fn paint_dashed_horizontal(
    window: &mut Window,
    start_x: Pixels,
    end_x: Pixels,
    y: Pixels,
    thickness: Pixels,
    dash: Pixels,
    gap: Pixels,
) {
    let mut x = start_x;
    while x < end_x {
        let dash_end = (x + dash).min(end_x);
        let segment = Bounds { origin: point(x, y - (thickness / 2.0)), size: size(dash_end - x, thickness) };
        window.paint_quad(fill(segment, hsla(220.0 / 360.0, 0.08, 0.50, 0.58)).corner_radii(Corners::all(px(0.8))));
        x = dash_end + gap;
    }
}

fn to_chart_point(chart_bounds: Bounds<Pixels>, sample: (f32, f32), vertical_offset: f32) -> gpui::Point<Pixels> {
    point(
        chart_bounds.left() + chart_bounds.size.width * sample.0,
        chart_bounds.top() + chart_bounds.size.height * (sample.1 + vertical_offset),
    )
}
