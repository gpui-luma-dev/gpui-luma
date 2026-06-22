use std::sync::{Arc, OnceLock};

use gpui::{App, Bounds, Div, Hsla, MouseButton, PathBuilder, Pixels, Stateful, Window, canvas, div, hsla, px, prelude::*};

use super::input::{Slider2InputStrategy, angle_for_percentage};
use super::layout::display_position;
use super::model::{Slider2RenderModel, Slider2ThumbSize, ThumbId, TrackPresentation};
use super::template::{Slider2Template, Slider2TemplateHandlers, render_slider2_thumb_at};
use super::Slider2Drag;
use crate::controls::slider::{SliderTheme, default_slider_theme};

const DISABLED_OPACITY: f32 = 0.56;
const DIAL_SIZE: f32 = 200.0;
const TRACK_RADIUS: f32 = 72.0;

pub struct ThemedAngularDialTemplate {
    theme: Arc<dyn SliderTheme>,
}

impl ThemedAngularDialTemplate {
    pub fn new(theme: Arc<dyn SliderTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_angular_dial_template() -> Arc<dyn Slider2Template> {
    static TEMPLATE: OnceLock<Arc<dyn Slider2Template>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedAngularDialTemplate::new(default_slider_theme()))).clone()
}

impl Slider2Template for ThemedAngularDialTemplate {
    fn render(
        &self,
        model: &Slider2RenderModel<'_>,
        handlers: Slider2TemplateHandlers,
        primary_thumb_id: ThumbId,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let Slider2TemplateHandlers { track_bounds, hover, mouse_down, mouse_up, mouse_up_out, drag_move } = handlers;
        let look = self.theme.resolve(model.size, model.thumb_size.map(slider2_thumb_size), model.state);
        let Slider2InputStrategy::Angular { min_angle, max_angle } = model.strategy else {
            return div().id(model.id.clone());
        };

        let primary_thumb =
            model.thumbs.iter().find(|thumb| thumb.id == primary_thumb_id).or_else(|| model.thumbs.first());
        let thumb_position = primary_thumb.map(|thumb| thumb.position).unwrap_or(0.0);
        let display_percentage = display_position(thumb_position, model.reversed);
        let angle = angle_for_percentage(min_angle, max_angle, display_percentage);
        let center = DIAL_SIZE * 0.5;
        let thumb_x = center + TRACK_RADIUS * angle.cos();
        let thumb_y = center + TRACK_RADIUS * angle.sin();
        let track_radius =
            model.corner_radius.map(|radius| radius.to_pixels(window.rem_size())).unwrap_or(px(look.radius));
        let presentation = model.presentation;
        let track_background = look.track_background;
        let fill_background = look.fill_background;

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .size(px(DIAL_SIZE))
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out)
            .on_drag(Slider2Drag::new(model.id.clone(), primary_thumb_id), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(drag_move)
            .child(
                canvas(
                    move |_, _, _| (),
                    move |bounds, _, window, _| {
                        paint_arc_track(
                            bounds,
                            min_angle,
                            max_angle,
                            track_background,
                            fill_background,
                            presentation,
                            display_percentage,
                            track_radius,
                            window,
                        );
                    },
                )
                .absolute()
                .size_full(),
            )
            .child(render_slider2_thumb_at(&look, format!("{}-thumb", model.id), thumb_x, thumb_y, primary_thumb))
            .child(
                canvas(move |bounds, window, cx| track_bounds(&bounds, window, cx), |_, _, _, _| {})
                    .absolute()
                    .size_full(),
            );

        if model.enabled {
            root = root.cursor_pointer();
        } else {
            root = root.opacity(DISABLED_OPACITY);
        }

        root
    }
}

fn slider2_thumb_size(size: Slider2ThumbSize) -> crate::controls::slider::SliderThumbSize {
    match size {
        Slider2ThumbSize::Sm => crate::controls::slider::SliderThumbSize::Sm,
        Slider2ThumbSize::Md => crate::controls::slider::SliderThumbSize::Md,
        Slider2ThumbSize::Lg => crate::controls::slider::SliderThumbSize::Lg,
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_arc_track(
    bounds: Bounds<Pixels>,
    min_angle: f32,
    max_angle: f32,
    track_background: Hsla,
    fill_background: Hsla,
    presentation: TrackPresentation,
    percentage: f32,
    _track_radius: Pixels,
    window: &mut Window,
) {
    let center = bounds.center();
    let radius = TRACK_RADIUS;
    let segments = 64;
    let span = max_angle - min_angle;
    let fill_segments = ((segments as f32) * percentage.clamp(0.0, 1.0)).round() as usize;

    for index in 0..segments {
        let t0 = index as f32 / segments as f32;
        let t1 = (index + 1) as f32 / segments as f32;
        let a0 = min_angle + span * t0;
        let a1 = min_angle + span * t1;
        let color = match presentation {
            TrackPresentation::Fill if index < fill_segments => fill_background,
            TrackPresentation::Fill => track_background,
            TrackPresentation::Domain => {
                let mid_t = (t0 + t1) * 0.5;
                hsla(mid_t, 1.0, 0.5, 1.0)
            }
        };

        paint_arc_segment(center, radius, a0, a1, color, window);
    }
}

fn paint_arc_segment(
    center: gpui::Point<Pixels>,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    color: Hsla,
    window: &mut Window,
) {
    let stroke_width = px(6.0);
    let mut builder = PathBuilder::stroke(stroke_width);
    let steps = 4;

    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let angle = start_angle + (end_angle - start_angle) * t;
        let point = polar_point(center, radius, angle);

        if step == 0 {
            builder.move_to(point);
        } else {
            builder.line_to(point);
        }
    }

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

fn polar_point(center: gpui::Point<Pixels>, radius: f32, angle: f32) -> gpui::Point<Pixels> {
    gpui::point(center.x + px(radius * angle.cos()), center.y + px(radius * angle.sin()))
}
