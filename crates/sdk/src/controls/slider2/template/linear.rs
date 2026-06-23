use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Pixels, Stateful, Window, div, px, prelude::*};

use super::{
    Slider2BoundsHandler, Slider2InteractionHandlers, Slider2Template, Slider2TemplateHandlers,
    attach_linear_interaction, attach_thumb_drag, render_linear_thumb, slider2_thumb_size, track_bounds_canvas,
    track_surface_background,
};
use crate::controls::color::style::StyledExt;
use crate::controls::slider::{SliderTheme, default_slider_theme};

use crate::controls::slider2::layout::{display_position, segment_corner_radii, segment_display_span};
use crate::controls::slider2::model::{
    Slider2Orientation, Slider2RenderModel, ThumbId, TrackPresentation, TrackSegmentKind,
};

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
        _primary_thumb_id: ThumbId,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
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

        let Slider2TemplateHandlers {
            track_bounds,
            hover,
            mouse_down,
            mouse_up,
            mouse_up_out,
            drag_move,
            thumb_mouse_down,
        } = handlers;
        let interaction =
            Slider2InteractionHandlers { hover, mouse_down, mouse_up, mouse_up_out, drag_move: drag_move.clone() };
        let model_id = model.id.clone();
        let enabled = model.enabled;

        let root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .when(model.orientation == Slider2Orientation::Horizontal, |this| {
                this.w_full().min_w(px(0.0)).h(px(root_height)).px(px(look.thumb_size * 0.5))
            })
            .when(model.orientation == Slider2Orientation::Vertical, |this| this.w(px(root_width)).h(px(root_height)));

        let root = match model.orientation {
            Slider2Orientation::Horizontal => {
                let track_top = (short_axis - cross_axis) * 0.5;
                let thumb_top = (short_axis - look.thumb_size) * 0.5 - thumb_focus_offset();
                let thumb_center_offset = -(look.thumb_size * 0.5 + thumb_focus_offset());

                let track = render_horizontal_track(model, &look, track_top, cross_axis, track_radius, track_bounds);

                let thumbs = ordered_thumbs(model).into_iter().map(|thumb| {
                    let display_percentage = display_position(thumb.position.clamp(0.0, 1.0), model.reversed);
                    let active = model.active_thumb_id == Some(thumb.id);
                    let node = render_linear_thumb(
                        model,
                        &look,
                        format!("{}-thumb-{}", model.id, thumb.id.as_u64()),
                        display_percentage,
                        thumb_top,
                        Some(thumb_center_offset),
                        None,
                        Some(thumb),
                        active,
                    );
                    attach_thumb_drag(node, &model_id, thumb.id, thumb_mouse_down.clone(), drag_move.clone(), enabled)
                });

                root.child(track).children(thumbs)
            }
            Slider2Orientation::Vertical => {
                let track_left = (short_axis - cross_axis) * 0.5;
                let thumb_left = (short_axis - look.thumb_size) * 0.5;

                let track =
                    render_vertical_track(model, &look, track_left, long_axis, cross_axis, track_radius, track_bounds);

                let thumbs = ordered_thumbs(model).into_iter().map(|thumb| {
                    let display_percentage = display_position(thumb.position.clamp(0.0, 1.0), model.reversed);
                    let thumb_top = (long_axis - look.thumb_size).max(0.0) * (1.0 - display_percentage);
                    let active = model.active_thumb_id == Some(thumb.id);
                    let node = render_linear_thumb(
                        model,
                        &look,
                        format!("{}-thumb-{}", model.id, thumb.id.as_u64()),
                        display_percentage,
                        thumb_top - thumb_focus_offset(),
                        None,
                        Some(thumb_left - thumb_focus_offset()),
                        Some(thumb),
                        active,
                    );
                    attach_thumb_drag(node, &model_id, thumb.id, thumb_mouse_down.clone(), drag_move.clone(), enabled)
                });

                root.child(track).children(thumbs)
            }
        };

        attach_linear_interaction(root, model, interaction)
    }
}

/// Inactive thumbs first; active thumb last so it paints on top while dragging.
fn ordered_thumbs<'a>(model: &'a Slider2RenderModel<'_>) -> Vec<&'a crate::controls::slider2::model::SliderThumbValue> {
    let mut thumbs: Vec<_> = model.thumbs.iter().collect();
    thumbs.sort_by_key(|thumb| model.active_thumb_id == Some(thumb.id));
    thumbs
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
        .when(!uses_sibling_segments, |this| this.bg(track_surface_background(model, look)).rounded(track_radius));

    if model.presentation == TrackPresentation::Domain {
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

    track.child(track_bounds_canvas(track_bounds))
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
        .when(!uses_sibling_segments, |this| this.bg(track_surface_background(model, look)).rounded(track_radius));

    if model.presentation == TrackPresentation::Domain {
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

    track.child(track_bounds_canvas(track_bounds))
}

fn render_fill_segment(
    segment: &crate::controls::slider2::model::TrackSegment,
    orientation: Slider2Orientation,
    reversed: bool,
    look: &crate::controls::slider::SliderLook,
    track_radius: Pixels,
    horizontal: bool,
) -> Div {
    use gpui::relative;

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

fn segment_background(kind: TrackSegmentKind, look: &crate::controls::slider::SliderLook) -> gpui::Hsla {
    match kind {
        TrackSegmentKind::Active | TrackSegmentKind::Domain => look.fill_background,
        TrackSegmentKind::Inactive | TrackSegmentKind::Blocked => look.track_background,
    }
}

fn uses_partitioned_track(model: &Slider2RenderModel<'_>) -> bool {
    match model.presentation {
        TrackPresentation::Fill => !model.track_segments.is_empty(),
        TrackPresentation::Domain => {
            model.track_segments.iter().any(|segment| segment.kind == TrackSegmentKind::Blocked)
        }
    }
}

const THUMB_FOCUS_GAP: f32 = 0.0;
const THUMB_FOCUS_WIDTH: f32 = 2.0;

fn thumb_focus_offset() -> f32 {
    THUMB_FOCUS_GAP + THUMB_FOCUS_WIDTH
}
