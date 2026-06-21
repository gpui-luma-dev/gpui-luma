use std::f32::consts::PI;

use gpui::*;

#[derive(IntoElement)]
pub struct Checkerboard {
    children: Vec<AnyElement>,
    is_dark: bool,
    corner_radii: Option<Corners<Pixels>>,
}

impl Checkerboard {
    pub fn new(is_dark: bool) -> Self {
        Self { children: Vec::new(), is_dark, corner_radii: None }
    }

    pub fn corner_radius(mut self, radius: Pixels) -> Self {
        self.corner_radii = Some(Corners::all(radius));
        self
    }

    pub fn corner_radii(mut self, radii: Corners<Pixels>) -> Self {
        self.corner_radii = Some(radii);
        self
    }
}

impl ParentElement for Checkerboard {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Checkerboard {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let square_size = px(8.);
        // Use a subtle difference for the checkerboard
        let (c1, c2) = if self.is_dark {
            // Dark mode: dark grey and slightly lighter grey
            (hsla(0., 0., 0.1, 1.), hsla(0., 0., 0.13, 1.))
        } else {
            // Light mode: white and light grey
            (hsla(0., 0., 1.0, 1.), hsla(0., 0., 0.95, 1.))
        };

        let corner_radii = self.corner_radii.unwrap_or_default();

        div()
            .bg(c1)
            .overflow_hidden()
            .size_full()
            .child(
                gpui::canvas(
                    move |_, _, _| (),
                    move |bounds, _, window, _| {
                        let size = square_size;
                        let rows = (bounds.size.height / size).ceil() as i32;
                        let cols = (bounds.size.width / size).ceil() as i32;

                        for row in 0..rows {
                            for col in 0..cols {
                                if (row + col) % 2 == 0 {
                                    let origin = bounds.origin + gpui::point(size * (col as f32), size * (row as f32));

                                    window.paint_quad(gpui::PaintQuad {
                                        bounds: gpui::Bounds { origin, size: gpui::size(size, size) },
                                        corner_radii: gpui::Corners::default(),
                                        background: c2.into(),
                                        border_widths: gpui::Edges::default(),
                                        border_color: gpui::transparent_black(),
                                        border_style: gpui::BorderStyle::default(),
                                    });
                                }
                            }
                        }

                        paint_corner_occluder(window, bounds, corner_radii, c1);
                    },
                )
                .absolute()
                .size_full(),
            )
            .children(self.children)
    }
}

fn paint_corner_occluder(window: &mut Window, bounds: Bounds<Pixels>, radii: Corners<Pixels>, color: Hsla) {
    let left = bounds.origin.x.as_f32();
    let top = bounds.origin.y.as_f32();
    let right = (bounds.origin.x + bounds.size.width).as_f32();
    let bottom = (bounds.origin.y + bounds.size.height).as_f32();
    let max_radius = (bounds.size.width.min(bounds.size.height).as_f32() / 2.0).max(0.0);
    let steps = 48;

    let top_left = radii.top_left.as_f32().min(max_radius);
    if top_left > 0.0 {
        let mut builder = PathBuilder::fill();
        builder.move_to(point(px(left), px(top)));
        builder.line_to(point(px(left + top_left), px(top)));
        append_arc_points(&mut builder, left + top_left, top + top_left, top_left, -PI / 2.0, -PI, steps);
        builder.close();
        if let Ok(path) = builder.build() {
            window.paint_path(path, color);
        }
    }

    let top_right = radii.top_right.as_f32().min(max_radius);
    if top_right > 0.0 {
        let mut builder = PathBuilder::fill();
        builder.move_to(point(px(right), px(top)));
        builder.line_to(point(px(right - top_right), px(top)));
        append_arc_points(&mut builder, right - top_right, top + top_right, top_right, -PI / 2.0, 0.0, steps);
        builder.close();
        if let Ok(path) = builder.build() {
            window.paint_path(path, color);
        }
    }

    let bottom_left = radii.bottom_left.as_f32().min(max_radius);
    if bottom_left > 0.0 {
        let mut builder = PathBuilder::fill();
        builder.move_to(point(px(left), px(bottom)));
        builder.line_to(point(px(left), px(bottom - bottom_left)));
        append_arc_points(&mut builder, left + bottom_left, bottom - bottom_left, bottom_left, PI, PI / 2.0, steps);
        builder.close();
        if let Ok(path) = builder.build() {
            window.paint_path(path, color);
        }
    }

    let bottom_right = radii.bottom_right.as_f32().min(max_radius);
    if bottom_right > 0.0 {
        let mut builder = PathBuilder::fill();
        builder.move_to(point(px(right), px(bottom)));
        builder.line_to(point(px(right - bottom_right), px(bottom)));
        append_arc_points(
            &mut builder,
            right - bottom_right,
            bottom - bottom_right,
            bottom_right,
            PI / 2.0,
            0.0,
            steps,
        );
        builder.close();
        if let Ok(path) = builder.build() {
            window.paint_path(path, color);
        }
    }
}

fn append_arc_points(
    builder: &mut PathBuilder,
    center_x: f32,
    center_y: f32,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    steps: usize,
) {
    if steps == 0 {
        return;
    }

    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let angle = start_angle + (end_angle - start_angle) * t;
        builder.line_to(point(px(center_x + radius * angle.cos()), px(center_y + radius * angle.sin())));
    }
}
