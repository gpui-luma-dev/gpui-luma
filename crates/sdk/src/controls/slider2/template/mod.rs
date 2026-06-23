mod angular_dial;
mod circular_ring;
mod linear;

use std::sync::Arc;
use std::f32::consts::FRAC_PI_2;

use gpui::{
    App, BorderStyle, Bounds, Corners, Div, DragMoveEvent, Edges, Hsla, IntoElement, MouseButton, MouseDownEvent,
    MouseUpEvent, PaintQuad, Pixels, Point, SharedString, Stateful, Window, canvas, div, hsla, point, px, size,
    transparent_black, prelude::*,
};

use crate::controls::color::shape::{Arc as ShapeArc, ArcData};

use super::model::{Slider2RenderModel, Slider2ThumbSize, SliderThumbValue, ThumbId};
use super::Slider2Drag;

pub use angular_dial::{ThemedAngularDialTemplate, default_angular_dial_template};
pub use circular_ring::{ThemedCircularRingTemplate, default_circular_ring_template};
pub use linear::{ThemedSlider2Template, default_slider2_template};

pub type Slider2BoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + Send + Sync>;
pub type Slider2HoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + Send + Sync>;
pub type Slider2MouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + Send + Sync>;
pub type Slider2MouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + Send + Sync>;
pub type Slider2DragMoveHandler = Arc<dyn Fn(&DragMoveEvent<Slider2Drag>, &mut Window, &mut App) + Send + Sync>;
pub type Slider2ThumbMouseDownHandler = Arc<dyn Fn(&ThumbId, &MouseDownEvent, &mut Window, &mut App) + Send + Sync>;

pub struct Slider2TemplateHandlers {
    pub track_bounds: Slider2BoundsHandler,
    pub hover: Slider2HoverHandler,
    pub mouse_down: Slider2MouseDownHandler,
    pub mouse_up: Slider2MouseUpHandler,
    pub mouse_up_out: Slider2MouseUpHandler,
    pub drag_move: Slider2DragMoveHandler,
    pub thumb_mouse_down: Slider2ThumbMouseDownHandler,
}

pub(crate) struct Slider2InteractionHandlers {
    pub hover: Slider2HoverHandler,
    pub mouse_down: Slider2MouseDownHandler,
    pub mouse_up: Slider2MouseUpHandler,
    pub mouse_up_out: Slider2MouseUpHandler,
    pub drag_move: Slider2DragMoveHandler,
}

impl From<Slider2TemplateHandlers> for (Slider2BoundsHandler, Slider2InteractionHandlers) {
    fn from(handlers: Slider2TemplateHandlers) -> Self {
        (
            handlers.track_bounds,
            Slider2InteractionHandlers {
                hover: handlers.hover,
                mouse_down: handlers.mouse_down,
                mouse_up: handlers.mouse_up,
                mouse_up_out: handlers.mouse_up_out,
                drag_move: handlers.drag_move,
            },
        )
    }
}

pub trait Slider2Template: Send + Sync {
    fn render(
        &self,
        model: &Slider2RenderModel<'_>,
        handlers: Slider2TemplateHandlers,
        primary_thumb_id: ThumbId,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub(crate) const DISABLED_OPACITY: f32 = 0.56;
pub(crate) const DIAL_SIZE: f32 = 200.0;
pub(crate) const TRACK_RADIUS: f32 = 72.0;

const THUMB_FOCUS_GAP: f32 = 0.0;
const THUMB_FOCUS_WIDTH: f32 = 2.0;

pub(crate) fn slider2_thumb_size(size: Slider2ThumbSize) -> crate::controls::slider::SliderThumbSize {
    match size {
        Slider2ThumbSize::Sm => crate::controls::slider::SliderThumbSize::Sm,
        Slider2ThumbSize::Md => crate::controls::slider::SliderThumbSize::Md,
        Slider2ThumbSize::Lg => crate::controls::slider::SliderThumbSize::Lg,
    }
}

pub(crate) fn uses_static_track_surface(model: &Slider2RenderModel<'_>) -> bool {
    model.thumb_policy.is_multi_thumb() || model.presentation == super::model::TrackPresentation::Domain
}

pub(crate) fn track_surface_background(
    model: &Slider2RenderModel<'_>,
    look: &crate::controls::slider::SliderLook,
) -> Hsla {
    if uses_static_track_surface(model) {
        look.fill_background
    } else {
        look.track_background
    }
}

pub(crate) fn track_muted_background(look: &crate::controls::slider::SliderLook) -> Hsla {
    look.track_background
}

pub(crate) fn render_slider2_thumb_at(
    look: &crate::controls::slider::SliderLook,
    id: impl Into<gpui::ElementId>,
    center_x: f32,
    center_y: f32,
    thumb: Option<&SliderThumbValue>,
) -> Stateful<Div> {
    let thumb_fill = thumb.and_then(|thumb| thumb.preview).unwrap_or(look.thumb_background);
    let offset = look.thumb_size * 0.5 + thumb_focus_offset();

    div()
        .id(id)
        .absolute()
        .left(px(center_x - offset))
        .top(px(center_y - offset))
        .flex()
        .items_center()
        .justify_center()
        .p(px(THUMB_FOCUS_GAP))
        .border(px(THUMB_FOCUS_WIDTH))
        .border_color(focus_ring_color(look.focus_ring))
        .rounded(px(look.radius + thumb_focus_offset()))
        .child(
            div()
                .size(px(look.thumb_size))
                .bg(thumb_fill)
                .border_1()
                .border_color(look.thumb_border)
                .rounded(px(look.radius))
                .shadow(look.thumb_shadow.clone()),
        )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_linear_thumb(
    model: &Slider2RenderModel<'_>,
    look: &crate::controls::slider::SliderLook,
    id: impl Into<gpui::ElementId>,
    display_percentage: f32,
    top: f32,
    main_offset: Option<f32>,
    cross_offset: Option<f32>,
    thumb: Option<&SliderThumbValue>,
    active: bool,
) -> Stateful<Div> {
    use super::model::Slider2Orientation;
    use gpui::relative;

    let thumb_fill = thumb.and_then(|thumb| thumb.preview).unwrap_or(look.thumb_background);
    let focus_ring = if active { look.focus_ring } else { None };

    let mut thumb = div()
        .id(id)
        .absolute()
        .top(px(top))
        .flex()
        .items_center()
        .justify_center()
        .p(px(THUMB_FOCUS_GAP))
        .border(px(THUMB_FOCUS_WIDTH))
        .border_color(focus_ring_color(focus_ring))
        .rounded(px(look.radius + thumb_focus_offset()))
        .child(
            div()
                .size(px(look.thumb_size))
                .bg(thumb_fill)
                .border_1()
                .border_color(look.thumb_border)
                .rounded(px(look.radius))
                .shadow(look.thumb_shadow.clone()),
        );

    thumb = match model.orientation {
        Slider2Orientation::Horizontal => {
            let mut node = thumb.left(relative(display_percentage));
            if let Some(offset) = main_offset {
                node = node.ml(px(offset));
            }
            node
        }
        Slider2Orientation::Vertical => {
            if let Some(offset) = cross_offset {
                thumb = thumb.left(px(offset));
            }
            thumb
        }
    };

    thumb
}

pub(crate) fn track_bounds_canvas(track_bounds: Slider2BoundsHandler) -> impl IntoElement {
    canvas(move |bounds, window, cx| track_bounds(&bounds, window, cx), |_, _, _, _| {})
        .absolute()
        .size_full()
}

pub(crate) fn attach_radial_interaction(
    mut root: Stateful<Div>,
    model: &Slider2RenderModel<'_>,
    handlers: Slider2InteractionHandlers,
    primary_thumb_id: ThumbId,
) -> Stateful<Div> {
    let Slider2InteractionHandlers { hover, mouse_down, mouse_up, mouse_up_out, drag_move, .. } = handlers;

    root = root
        .on_hover(hover)
        .on_mouse_down(MouseButton::Left, mouse_down)
        .on_mouse_up(MouseButton::Left, mouse_up)
        .on_mouse_up_out(MouseButton::Left, mouse_up_out)
        .on_drag(Slider2Drag::new(model.id.clone(), primary_thumb_id), |drag, _, _, cx| {
            cx.stop_propagation();
            cx.new(|_| drag.clone())
        })
        .on_drag_move({
            let drag_move = drag_move.clone();
            move |event, window, cx| drag_move(event, window, cx)
        });

    if model.enabled {
        root.cursor_pointer()
    } else {
        root.opacity(DISABLED_OPACITY)
    }
}

pub(crate) fn attach_linear_interaction(
    mut root: Stateful<Div>,
    model: &Slider2RenderModel<'_>,
    handlers: Slider2InteractionHandlers,
) -> Stateful<Div> {
    let Slider2InteractionHandlers { hover, mouse_down, mouse_up, mouse_up_out, drag_move, .. } = handlers;

    root = root
        .on_hover(hover)
        .on_mouse_down(MouseButton::Left, mouse_down)
        .on_mouse_up(MouseButton::Left, mouse_up)
        .on_mouse_up_out(MouseButton::Left, mouse_up_out)
        .on_drag_move({
            let drag_move = drag_move.clone();
            move |event, window, cx| drag_move(event, window, cx)
        });

    if model.enabled {
        root.cursor_pointer()
    } else {
        root.opacity(DISABLED_OPACITY)
    }
}

pub(crate) fn attach_thumb_drag(
    mut thumb: Stateful<Div>,
    model_id: &SharedString,
    thumb_id: ThumbId,
    mouse_down: Slider2ThumbMouseDownHandler,
    drag_move: Slider2DragMoveHandler,
    enabled: bool,
) -> Stateful<Div> {
    thumb = thumb
        .on_mouse_down(MouseButton::Left, {
            let mouse_down = mouse_down.clone();
            move |event, window, cx| mouse_down(&thumb_id, event, window, cx)
        })
        .on_drag(Slider2Drag::new(model_id.clone(), thumb_id), |drag, _, _, cx| {
            cx.stop_propagation();
            cx.new(|_| drag.clone())
        })
        .on_drag_move({
            let drag_move = drag_move.clone();
            move |event, window, cx| drag_move(event, window, cx)
        });

    if enabled {
        thumb.cursor_pointer()
    } else {
        thumb.opacity(DISABLED_OPACITY)
    }
}

pub(crate) fn arc_annulus_radii(arc_thickness: f32) -> (f32, f32) {
    let outer = TRACK_RADIUS + arc_thickness * 0.5;
    let inner = (TRACK_RADIUS - arc_thickness * 0.5).max(0.0);
    (inner, outer)
}

fn arc_shape_angles(start_angle: f32, end_angle: f32) -> (f32, f32) {
    // `Arc` path math applies -π/2 internally; offset so angles match thumb polar coords.
    (start_angle + FRAC_PI_2, end_angle + FRAC_PI_2)
}

pub(crate) fn paint_radial_annulus(
    bounds: Bounds<Pixels>,
    start_angle: f32,
    end_angle: f32,
    color: Hsla,
    arc_thickness: f32,
    window: &mut Window,
) {
    if end_angle <= start_angle + f32::EPSILON || arc_thickness <= f32::EPSILON {
        return;
    }

    let (inner, outer) = arc_annulus_radii(arc_thickness);
    let (start_angle, end_angle) = arc_shape_angles(start_angle, end_angle);
    let arc_shape = ShapeArc::new().inner_radius(inner).outer_radius(outer);
    let data = ArcData { data: &(), index: 0, value: 1.0, start_angle, end_angle, pad_angle: 0.0 };
    arc_shape.paint(&data, color, None, None, &bounds, window);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_radial_fill_track(
    bounds: Bounds<Pixels>,
    min_angle: f32,
    max_angle: f32,
    percentage: f32,
    track_background: Hsla,
    fill_background: Hsla,
    arc_thickness: f32,
    window: &mut Window,
) {
    let span = max_angle - min_angle;
    let percentage = percentage.clamp(0.0, 1.0);

    paint_radial_annulus(bounds, min_angle, max_angle, track_background, arc_thickness, window);

    if percentage > f32::EPSILON {
        let fill_end = min_angle + span * percentage;
        paint_radial_annulus(bounds, min_angle, fill_end, fill_background, arc_thickness, window);
    }

    let center = bounds.center();
    let fill_segments = usize::from(percentage > f32::EPSILON);
    let segment_count = if percentage >= 1.0 - f32::EPSILON { 1 } else { 2 };
    let (start_color, end_color) =
        arc_fill_end_cap_colors(fill_segments, segment_count, track_background, fill_background);
    paint_arc_open_end_caps(center, TRACK_RADIUS, min_angle, max_angle, start_color, end_color, arc_thickness, window);
}

pub(crate) fn paint_arc_round_cap(
    center: Point<Pixels>,
    radius: f32,
    angle: f32,
    diameter: f32,
    color: Hsla,
    window: &mut Window,
) {
    if diameter <= f32::EPSILON {
        return;
    }

    let cap_center = polar_point(center, radius, angle);
    let cap_radius = diameter * 0.5;
    window.paint_quad(PaintQuad {
        bounds: Bounds {
            origin: point(cap_center.x - px(cap_radius), cap_center.y - px(cap_radius)),
            size: size(px(diameter), px(diameter)),
        },
        corner_radii: Corners::all(px(cap_radius)),
        background: color.into(),
        border_widths: Edges::default(),
        border_color: transparent_black(),
        border_style: BorderStyle::default(),
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_arc_open_end_caps(
    center: Point<Pixels>,
    radius: f32,
    min_angle: f32,
    max_angle: f32,
    start_color: Hsla,
    end_color: Hsla,
    stroke_width: f32,
    window: &mut Window,
) {
    use std::f32::consts::TAU;

    let span = max_angle - min_angle;
    if span <= f32::EPSILON || span >= TAU - f32::EPSILON {
        return;
    }

    paint_arc_round_cap(center, radius, min_angle, stroke_width, start_color, window);
    paint_arc_round_cap(center, radius, max_angle, stroke_width, end_color, window);
}

pub(crate) fn arc_fill_end_cap_colors(
    fill_segments: usize,
    segment_count: usize,
    track_background: Hsla,
    fill_background: Hsla,
) -> (Hsla, Hsla) {
    let start_color = if fill_segments > 0 {
        fill_background
    } else {
        track_background
    };
    let end_color = if fill_segments >= segment_count {
        fill_background
    } else {
        track_background
    };
    (start_color, end_color)
}

pub(crate) fn polar_point(center: Point<Pixels>, radius: f32, angle: f32) -> Point<Pixels> {
    gpui::point(center.x + px(radius * angle.cos()), center.y + px(radius * angle.sin()))
}

fn thumb_focus_offset() -> f32 {
    THUMB_FOCUS_GAP + THUMB_FOCUS_WIDTH
}

fn focus_ring_color(focus_ring: Option<Hsla>) -> Hsla {
    focus_ring.unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 0.0))
}

#[cfg(test)]
mod tests {
    use gpui::hsla;

    use super::*;

    #[test]
    fn arc_fill_end_cap_colors_match_outer_segment_ends() {
        let track = hsla(0.0, 0.0, 0.5, 1.0);
        let fill = hsla(0.6, 0.8, 0.5, 1.0);

        assert_eq!(arc_fill_end_cap_colors(0, 64, track, fill), (track, track));
        assert_eq!(arc_fill_end_cap_colors(32, 64, track, fill), (fill, track));
        assert_eq!(arc_fill_end_cap_colors(64, 64, track, fill), (fill, fill));
    }
}
