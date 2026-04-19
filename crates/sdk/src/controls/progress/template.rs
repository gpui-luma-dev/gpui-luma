use std::f32::consts::PI;
use std::sync::{Arc, OnceLock};

use gpui::{App, Div, PathBuilder, Stateful, Window, canvas, div, point, px, prelude::*};

use super::ProgressRenderModel;
use crate::theme::{ProgressTheme, default_progress_theme};

pub trait ProgressTemplate: Send + Sync {
    fn render(&self, model: &ProgressRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

pub struct ThemedProgressTemplate {
    theme: Arc<dyn ProgressTheme>,
}

impl ThemedProgressTemplate {
    pub fn new(theme: Arc<dyn ProgressTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_progress_template() -> Arc<dyn ProgressTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ProgressTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedProgressTemplate::new(default_progress_theme()))).clone()
}

impl ProgressTemplate for ThemedProgressTemplate {
    fn render(&self, model: &ProgressRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let appearance = self.theme.resolve(model.enabled);
        let percentage = model.percentage.clamp(0.0, 1.0);
        let size = px(appearance.size);
        let stroke_width = px(appearance.stroke_width);
        let track_color = appearance.track_color;
        let progress_color = appearance.progress_color;

        div().id(model.id.clone()).relative().size(size).child(
            canvas(
                |_, _, _| {},
                move |bounds, _, window, _cx| {
                    let center_x = bounds.origin.x + bounds.size.width / 2.0;
                    let center_y = bounds.origin.y + bounds.size.height / 2.0;
                    let radius = (size / 2.0) - stroke_width;

                    paint_circle(center_x, center_y, radius, stroke_width, track_color, window);
                    paint_progress_arc(center_x, center_y, radius, stroke_width, percentage, progress_color, window);
                },
            )
            .size_full(),
        )
    }
}

fn paint_circle(
    center_x: gpui::Pixels,
    center_y: gpui::Pixels,
    radius: gpui::Pixels,
    stroke_width: gpui::Pixels,
    color: gpui::Hsla,
    window: &mut Window,
) {
    let mut builder = PathBuilder::stroke(stroke_width);

    builder.move_to(point(center_x + radius, center_y));
    builder.arc_to(point(radius, radius), px(0.0), false, true, point(center_x - radius, center_y));
    builder.arc_to(point(radius, radius), px(0.0), false, true, point(center_x + radius, center_y));
    builder.close();

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

fn paint_progress_arc(
    center_x: gpui::Pixels,
    center_y: gpui::Pixels,
    radius: gpui::Pixels,
    stroke_width: gpui::Pixels,
    percentage: f32,
    color: gpui::Hsla,
    window: &mut Window,
) {
    if percentage <= 0.0 {
        return;
    }

    if percentage >= 0.999 {
        paint_circle(center_x, center_y, radius, stroke_width, color, window);
        return;
    }

    let mut builder = PathBuilder::stroke(stroke_width);
    let start_x = center_x;
    let start_y = center_y - radius;
    let angle = -PI / 2.0 + percentage * 2.0 * PI;
    let end_x = center_x + radius * angle.cos();
    let end_y = center_y + radius * angle.sin();

    builder.move_to(point(start_x, start_y));
    builder.arc_to(point(radius, radius), px(0.0), percentage > 0.5, true, point(end_x, end_y));

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}
