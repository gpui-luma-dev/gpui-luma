use std::sync::{Arc, OnceLock};

use gpui::{
    App, AppContext as _, Bounds, Div, DragMoveEvent, Hsla, MouseButton, MouseDownEvent, MouseUpEvent, Pixels,
    Stateful, Window, canvas, div, hsla, px, relative, prelude::*,
};

const DISABLED_OPACITY: f32 = 0.56;

use super::domain::DomainTrackRenderer;
use super::layout::{display_position, segment_corner_radii, segment_display_span};
use super::model::{
    Slider2Orientation, Slider2RenderModel, Slider2ThumbSize, SliderThumbValue, ThumbId, TrackPresentation,
    TrackSegmentKind,
};
use super::Slider2Drag;
use crate::controls::color::style::StyledExt;
use crate::controls::slider::{SliderTheme, default_slider_theme};

pub type Slider2BoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type Slider2HoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type Slider2MouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type Slider2MouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type Slider2DragMoveHandler = Box<dyn Fn(&DragMoveEvent<Slider2Drag>, &mut Window, &mut App) + 'static>;

pub struct Slider2TemplateHandlers {
    pub track_bounds: Slider2BoundsHandler,
    pub hover: Slider2HoverHandler,
    pub mouse_down: Slider2MouseDownHandler,
    pub mouse_up: Slider2MouseUpHandler,
    pub mouse_up_out: Slider2MouseUpHandler,
    pub drag_move: Slider2DragMoveHandler,
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

pub struct ThemedSlider2Template {
    theme: Arc<dyn SliderTheme>,
}

impl ThemedSlider2Template {
    pub fn new(theme: Arc<dyn SliderTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_slider2_template() -> Arc<dyn Slider2Template> {
    static TEMPLATE: OnceLock<Arc<dyn Slider2Template>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedSlider2Template::new(default_slider_theme()))).clone()
}

impl Slider2Template for ThemedSlider2Template {
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
        let long_axis = look.width;
        let short_axis = look.height;
        let cross_axis = look.track_height;
        let root_height = match model.orientation {
            Slider2Orientation::Horizontal => short_axis,
            Slider2Orientation::Vertical => long_axis,
        };
        let root_width = short_axis;
        let track_radius =
            model.corner_radius.map(|radius| radius.to_pixels(window.rem_size())).unwrap_or(px(look.radius));

        let primary_thumb =
            model.thumbs.iter().find(|thumb| thumb.id == primary_thumb_id).or_else(|| model.thumbs.first());
        let thumb_position = primary_thumb.map(|thumb| thumb.position.clamp(0.0, 1.0)).unwrap_or(0.0);
        let display_percentage = display_position(thumb_position, model.reversed);

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .when(model.orientation == Slider2Orientation::Horizontal, |this| {
                this.w_full().min_w(px(0.0)).h(px(root_height)).px(px(look.thumb_size * 0.5))
            })
            .when(model.orientation == Slider2Orientation::Vertical, |this| this.w(px(root_width)).h(px(root_height)))
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out)
            .on_drag(Slider2Drag::new(model.id.clone(), primary_thumb_id), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(drag_move);

        root = match model.orientation {
            Slider2Orientation::Horizontal => {
                let track_top = (short_axis - cross_axis) * 0.5;
                let thumb_top = (short_axis - look.thumb_size) * 0.5 - thumb_focus_offset();
                let thumb_center_offset = -(look.thumb_size * 0.5 + thumb_focus_offset());

                let track = render_horizontal_track(model, &look, track_top, cross_axis, track_radius, track_bounds);

                let thumb = render_thumb(
                    model,
                    &look,
                    format!("{}-thumb", model.id),
                    display_percentage,
                    thumb_top,
                    Some(thumb_center_offset),
                    None,
                    primary_thumb,
                );

                root.child(track).child(thumb)
            }
            Slider2Orientation::Vertical => {
                let track_left = (short_axis - cross_axis) * 0.5;
                let thumb_left = (short_axis - look.thumb_size) * 0.5;
                let thumb_top = (long_axis - look.thumb_size).max(0.0) * (1.0 - display_percentage);

                let track =
                    render_vertical_track(model, &look, track_left, long_axis, cross_axis, track_radius, track_bounds);

                let thumb = render_thumb(
                    model,
                    &look,
                    format!("{}-thumb", model.id),
                    display_percentage,
                    thumb_top - thumb_focus_offset(),
                    None,
                    Some(thumb_left - thumb_focus_offset()),
                    primary_thumb,
                );

                root.child(track).child(thumb)
            }
        };

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

fn render_horizontal_track(
    model: &Slider2RenderModel<'_>,
    look: &crate::controls::slider::SliderLook,
    track_top: f32,
    cross_axis: f32,
    track_radius: Pixels,
    track_bounds: Slider2BoundsHandler,
) -> Stateful<Div> {
    let uses_sibling_segments = uses_partitioned_track(model);

    let mut track = div()
        .id(format!("{}-track", model.id))
        .absolute()
        .left(px(0.0))
        .right(px(0.0))
        .top(px(track_top))
        .h(px(cross_axis))
        .when(!uses_sibling_segments, |this| this.bg(look.track_background).rounded(track_radius));

    if model.presentation == TrackPresentation::Domain {
        if let Some(renderer) = model.domain_track.clone() {
            track = track.child(domain_canvas(renderer, model.orientation, model.reversed));
        }
        track =
            track.children(
                model.track_segments.iter().filter(|segment| segment.kind == TrackSegmentKind::Blocked).map(
                    |segment| render_fill_segment(segment, model.orientation, model.reversed, look, track_radius, true),
                ),
            );
    } else {
        track =
            track.children(model.track_segments.iter().map(|segment| {
                render_fill_segment(segment, model.orientation, model.reversed, look, track_radius, true)
            }));
    }

    track.child(
        canvas(move |bounds, window, cx| track_bounds(&bounds, window, cx), |_, _, _, _| {})
            .absolute()
            .size_full(),
    )
}

fn render_vertical_track(
    model: &Slider2RenderModel<'_>,
    look: &crate::controls::slider::SliderLook,
    track_left: f32,
    long_axis: f32,
    cross_axis: f32,
    track_radius: Pixels,
    track_bounds: Slider2BoundsHandler,
) -> Stateful<Div> {
    let uses_sibling_segments = uses_partitioned_track(model);

    let mut track = div()
        .id(format!("{}-track", model.id))
        .absolute()
        .left(px(track_left))
        .top(px(0.0))
        .w(px(cross_axis))
        .h(px(long_axis))
        .when(!uses_sibling_segments, |this| this.bg(look.track_background).rounded(track_radius));

    if model.presentation == TrackPresentation::Domain {
        if let Some(renderer) = model.domain_track.clone() {
            track = track.child(domain_canvas(renderer, model.orientation, model.reversed));
        }
        track =
            track.children(
                model.track_segments.iter().filter(|segment| segment.kind == TrackSegmentKind::Blocked).map(
                    |segment| {
                        render_fill_segment(segment, model.orientation, model.reversed, look, track_radius, false)
                    },
                ),
            );
    } else {
        track =
            track.children(model.track_segments.iter().map(|segment| {
                render_fill_segment(segment, model.orientation, model.reversed, look, track_radius, false)
            }));
    }

    track.child(
        canvas(move |bounds, window, cx| track_bounds(&bounds, window, cx), |_, _, _, _| {})
            .absolute()
            .size_full(),
    )
}

fn domain_canvas(
    renderer: Arc<dyn DomainTrackRenderer>,
    orientation: Slider2Orientation,
    reversed: bool,
) -> impl IntoElement {
    canvas(
        move |_, _, _| (),
        move |bounds, _, window, _| {
            renderer.paint(bounds, orientation, reversed, window);
        },
    )
    .absolute()
    .size_full()
}

fn render_fill_segment(
    segment: &super::model::TrackSegment,
    orientation: Slider2Orientation,
    reversed: bool,
    look: &crate::controls::slider::SliderLook,
    track_radius: Pixels,
    horizontal: bool,
) -> Div {
    let (display_start, display_span) = segment_display_span(segment, reversed);
    let background = segment_background(segment.kind, look);
    let corner_radii = segment_corner_radii(display_start, display_span, orientation, track_radius);

    if horizontal {
        div()
            .absolute()
            .left(relative(display_start))
            .top(px(0.0))
            .h_full()
            .w(relative(display_span))
            .bg(background)
            .corner_radii(corner_radii)
    } else {
        div()
            .absolute()
            .left(px(0.0))
            .bottom(relative(display_start))
            .w_full()
            .h(relative(display_span))
            .bg(background)
            .corner_radii(corner_radii)
    }
}

fn segment_background(kind: TrackSegmentKind, look: &crate::controls::slider::SliderLook) -> Hsla {
    match kind {
        TrackSegmentKind::Active | TrackSegmentKind::Domain => look.fill_background,
        TrackSegmentKind::Inactive | TrackSegmentKind::Blocked => look.track_background,
    }
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
fn render_thumb(
    model: &Slider2RenderModel<'_>,
    look: &crate::controls::slider::SliderLook,
    id: impl Into<gpui::ElementId>,
    display_percentage: f32,
    top: f32,
    main_offset: Option<f32>,
    cross_offset: Option<f32>,
    thumb: Option<&SliderThumbValue>,
) -> Stateful<Div> {
    let thumb_fill = thumb.and_then(|thumb| thumb.preview).unwrap_or(look.thumb_background);

    let mut thumb = div()
        .id(id)
        .absolute()
        .top(px(top))
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

const THUMB_FOCUS_GAP: f32 = 0.0;
const THUMB_FOCUS_WIDTH: f32 = 2.0;

fn thumb_focus_offset() -> f32 {
    THUMB_FOCUS_GAP + THUMB_FOCUS_WIDTH
}

fn focus_ring_color(focus_ring: Option<Hsla>) -> Hsla {
    focus_ring.unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 0.0))
}

fn uses_partitioned_track(model: &Slider2RenderModel<'_>) -> bool {
    match model.presentation {
        TrackPresentation::Fill => !model.track_segments.is_empty(),
        TrackPresentation::Domain => {
            model.track_segments.iter().any(|segment| segment.kind == TrackSegmentKind::Blocked)
        }
    }
}
