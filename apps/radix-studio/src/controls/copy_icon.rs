//! Native rendering of the two-outline Copy.tsx success animation.
use gpui::{Hsla, IntoElement, PathBuilder, RenderOnce, Window, App, canvas, div, point, px, prelude::*};

#[derive(IntoElement)]
pub(crate) struct CopyIcon {
    pub color: Hsla,
    pub displacement: f32,
}

/// Damped spring matching stiffness 160, damping 17 and mass 1.
pub(crate) fn success_displacement(progress: f32) -> f32 {
    let seconds = progress * 1.4;
    let spring = |t: f32| {
        let damping = 8.5_f32;
        let frequency = (160.0 - damping * damping).sqrt();
        1.0 - (-damping * t).exp() * ((frequency * t).cos() + damping / frequency * (frequency * t).sin())
    };
    if seconds < 0.4 {
        spring(seconds)
    } else if seconds < 1.0 {
        1.0
    } else if progress < 1.0 {
        1.0 - spring(seconds - 1.0)
    } else {
        0.0
    }
}

impl RenderOnce for CopyIcon {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .size(px(16.0))
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        let scale = bounds.size.width.as_f32() / 24.0;
                        let shift = self.displacement * 3.0;
                        let mut front = PathBuilder::stroke(px(2.0 * scale));
                        let p = |x: f32, y: f32| {
                            point(bounds.left() + px((x - shift) * scale), bounds.top() + px((y - shift) * scale))
                        };
                        front.move_to(p(10.0, 8.0));
                        front.line_to(p(20.0, 8.0));
                        front.curve_to(p(22.0, 10.0), p(22.0, 8.0));
                        front.line_to(p(22.0, 20.0));
                        front.curve_to(p(20.0, 22.0), p(22.0, 22.0));
                        front.line_to(p(10.0, 22.0));
                        front.curve_to(p(8.0, 20.0), p(8.0, 22.0));
                        front.line_to(p(8.0, 10.0));
                        front.curve_to(p(10.0, 8.0), p(8.0, 8.0));
                        front.close();
                        if let Ok(path) = front.build() {
                            window.paint_path(path, self.color);
                        }
                        let p = |x: f32, y: f32| {
                            point(bounds.left() + px((x + shift) * scale), bounds.top() + px((y + shift) * scale))
                        };
                        let mut back = PathBuilder::stroke(px(2.0 * scale));
                        back.move_to(p(4.0, 16.0));
                        back.cubic_bezier_to(p(2.0, 14.0), p(2.9, 16.0), p(2.0, 15.1));
                        back.line_to(p(2.0, 4.0));
                        back.cubic_bezier_to(p(4.0, 2.0), p(2.0, 2.9), p(2.9, 2.0));
                        back.line_to(p(14.0, 2.0));
                        back.cubic_bezier_to(p(16.0, 4.0), p(15.1, 2.0), p(16.0, 2.9));
                        if let Ok(path) = back.build() {
                            window.paint_path(path, self.color);
                        }
                    },
                )
                .size_full(),
            )
            .into_any_element()
    }
}
