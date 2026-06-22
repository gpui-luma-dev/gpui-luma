use std::sync::{Arc, OnceLock};

use gpui::{
    App, AppContext as _, Bounds, Div, DragMoveEvent, Hsla, MouseButton, MouseDownEvent, MouseUpEvent, Pixels,
    Stateful, Window, canvas, div, hsla, px, relative, prelude::*,
};

const DISABLED_OPACITY: f32 = 0.56;

use super::model::RangeSliderSegmentKind;
use super::{RangeSliderDrag, RangeSliderRenderModel, SliderOrientation};
use crate::controls::color::style::StyledExt;
use crate::controls::slider::{SliderTheme, default_slider_theme};

pub type RangeSliderBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type RangeSliderHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type RangeSliderMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type RangeSliderMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type RangeSliderDragMoveHandler = Box<dyn Fn(&DragMoveEvent<RangeSliderDrag>, &mut Window, &mut App) + 'static>;

pub struct RangeSliderTemplateHandlers {
    pub track_bounds: RangeSliderBoundsHandler,
    pub hover: RangeSliderHoverHandler,
    pub mouse_down: RangeSliderMouseDownHandler,
    pub mouse_up: RangeSliderMouseUpHandler,
    pub mouse_up_out: RangeSliderMouseUpHandler,
    pub drag_move: RangeSliderDragMoveHandler,
}

pub trait RangeSliderTemplate: Send + Sync {
    fn render(
        &self,
        model: &RangeSliderRenderModel<'_>,
        handlers: RangeSliderTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedRangeSliderTemplate {
    theme: Arc<dyn SliderTheme>,
}

impl ThemedRangeSliderTemplate {
    pub fn new(theme: Arc<dyn SliderTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_range_slider_template() -> Arc<dyn RangeSliderTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn RangeSliderTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedRangeSliderTemplate::new(default_slider_theme()))).clone()
}

impl RangeSliderTemplate for ThemedRangeSliderTemplate {
    fn render(
        &self,
        model: &RangeSliderRenderModel<'_>,
        handlers: RangeSliderTemplateHandlers,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let RangeSliderTemplateHandlers { track_bounds, hover, mouse_down, mouse_up, mouse_up_out, drag_move } =
            handlers;
        let look = self.theme.resolve(model.size, model.thumb_size, model.state);
        let percentage = model.percentage.clamp(0.0, 1.0);
        let long_axis = look.width;
        let short_axis = look.height;
        let cross_axis = look.track_height;
        let root_height = match model.orientation {
            SliderOrientation::Horizontal => short_axis,
            SliderOrientation::Vertical => long_axis,
        };
        let root_width = short_axis;

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .when(model.orientation == SliderOrientation::Horizontal, |this| {
                this.w_full().min_w(px(0.0)).h(px(root_height)).px(px(look.thumb_size * 0.5))
            })
            .when(model.orientation == SliderOrientation::Vertical, |this| this.w(px(root_width)).h(px(root_height)))
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out)
            .on_drag(RangeSliderDrag::new(model.id.clone()), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(drag_move);

        root = match model.orientation {
            SliderOrientation::Horizontal => {
                let track_top = (short_axis - cross_axis) * 0.5;
                let thumb_top = (short_axis - look.thumb_size) * 0.5 - thumb_focus_offset();
                let thumb_center_offset = -(look.thumb_size * 0.5 + thumb_focus_offset());
                let track_radius =
                    model.corner_radius.map(|radius| radius.to_pixels(window.rem_size())).unwrap_or(px(look.radius));

                let track = div()
                    .id(format!("{}-track", model.id))
                    .absolute()
                    .left(px(0.0))
                    .right(px(0.0))
                    .top(px(track_top))
                    .h(px(cross_axis))
                    .bg(look.track_background)
                    .rounded(track_radius)
                    .children(model.track_segments.iter().map(|segment| {
                        let start = segment.start_percentage.clamp(0.0, 1.0);
                        let end = segment.end_percentage.clamp(0.0, 1.0);
                        let width = (end - start).clamp(0.0, 1.0);
                        let background = match segment.kind {
                            RangeSliderSegmentKind::Allowed => look.fill_background,
                            RangeSliderSegmentKind::Gap => look.track_background,
                        };

                        div()
                            .absolute()
                            .left(relative(start))
                            .top(px(0.0))
                            .h_full()
                            .w(relative(width))
                            .bg(background)
                            .corner_radii(segment_corner_radii(start, end, model.orientation, track_radius))
                    }))
                    .child(
                        canvas(move |bounds, window, cx| track_bounds(&bounds, window, cx), |_, _, _, _| {})
                            .absolute()
                            .size_full(),
                    );

                let thumb = div()
                    .id(format!("{}-thumb", model.id))
                    .absolute()
                    .left(relative(percentage))
                    .top(px(thumb_top))
                    .ml(px(thumb_center_offset))
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
                            .bg(look.thumb_background)
                            .border_1()
                            .border_color(look.thumb_border)
                            .rounded(px(look.radius))
                            .shadow(look.thumb_shadow.clone()),
                    );

                root.child(track).child(thumb)
            }
            SliderOrientation::Vertical => {
                let track_left = (short_axis - cross_axis) * 0.5;
                let thumb_left = (short_axis - look.thumb_size) * 0.5;
                let thumb_top = (long_axis - look.thumb_size).max(0.0) * (1.0 - percentage);
                let track_radius =
                    model.corner_radius.map(|radius| radius.to_pixels(window.rem_size())).unwrap_or(px(look.radius));

                let track = div()
                    .id(format!("{}-track", model.id))
                    .absolute()
                    .left(px(track_left))
                    .top(px(0.0))
                    .w(px(cross_axis))
                    .h(px(long_axis))
                    .bg(look.track_background)
                    .rounded(track_radius)
                    .children(model.track_segments.iter().map(|segment| {
                        let start = segment.start_percentage.clamp(0.0, 1.0);
                        let end = segment.end_percentage.clamp(0.0, 1.0);
                        let height = (end - start).clamp(0.0, 1.0);
                        let background = match segment.kind {
                            RangeSliderSegmentKind::Allowed => look.fill_background,
                            RangeSliderSegmentKind::Gap => look.track_background,
                        };

                        div()
                            .absolute()
                            .left(px(0.0))
                            .bottom(relative(start))
                            .w_full()
                            .h(relative(height))
                            .bg(background)
                            .corner_radii(segment_corner_radii(start, end, model.orientation, track_radius))
                    }))
                    .child(
                        canvas(move |bounds, window, cx| track_bounds(&bounds, window, cx), |_, _, _, _| {})
                            .absolute()
                            .size_full(),
                    );

                let thumb = div()
                    .id(format!("{}-thumb", model.id))
                    .absolute()
                    .left(px(thumb_left - thumb_focus_offset()))
                    .top(px(thumb_top - thumb_focus_offset()))
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
                            .bg(look.thumb_background)
                            .border_1()
                            .border_color(look.thumb_border)
                            .rounded(px(look.radius))
                            .shadow(look.thumb_shadow.clone()),
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

const THUMB_FOCUS_GAP: f32 = 0.0;
const THUMB_FOCUS_WIDTH: f32 = 2.0;

fn thumb_focus_offset() -> f32 {
    THUMB_FOCUS_GAP + THUMB_FOCUS_WIDTH
}

fn focus_ring_color(focus_ring: Option<Hsla>) -> Hsla {
    focus_ring.unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 0.0))
}

fn segment_corner_radii(start: f32, end: f32, orientation: SliderOrientation, radius: Pixels) -> gpui::Corners<Pixels> {
    let mut corner_radii = gpui::Corners::default();
    apply_edge_corner_radii(&mut corner_radii, orientation, radius, touches_track_start(start), touches_track_end(end));
    corner_radii
}

fn apply_edge_corner_radii(
    corner_radii: &mut gpui::Corners<Pixels>,
    orientation: SliderOrientation,
    radius: Pixels,
    is_first: bool,
    is_last: bool,
) {
    if is_first {
        match orientation {
            SliderOrientation::Horizontal => {
                corner_radii.top_left = radius;
                corner_radii.bottom_left = radius;
            }
            SliderOrientation::Vertical => {
                corner_radii.bottom_left = radius;
                corner_radii.bottom_right = radius;
            }
        }
    }

    if is_last {
        match orientation {
            SliderOrientation::Horizontal => {
                corner_radii.top_right = radius;
                corner_radii.bottom_right = radius;
            }
            SliderOrientation::Vertical => {
                corner_radii.top_left = radius;
                corner_radii.top_right = radius;
            }
        }
    }
}

fn touches_track_start(value: f32) -> bool {
    value <= 0.0001
}

fn touches_track_end(value: f32) -> bool {
    value >= 0.9999
}
