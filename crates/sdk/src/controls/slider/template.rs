use std::sync::{Arc, OnceLock};

use gpui::{
    App, AppContext as _, Bounds, Div, DragMoveEvent, Hsla, MouseButton, MouseDownEvent, MouseUpEvent, Pixels,
    Stateful, Window, canvas, div, hsla, px, prelude::*,
};

const DISABLED_OPACITY: f32 = 0.56;

use super::{SliderDrag, SliderOrientation, SliderRenderModel};
use crate::controls::slider::{SliderTheme, default_slider_theme};

pub type SliderBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type SliderHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SliderMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SliderMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type SliderDragMoveHandler = Box<dyn Fn(&DragMoveEvent<SliderDrag>, &mut Window, &mut App) + 'static>;

pub struct SliderTemplateHandlers {
    pub track_bounds: SliderBoundsHandler,
    pub hover: SliderHoverHandler,
    pub mouse_down: SliderMouseDownHandler,
    pub mouse_up: SliderMouseUpHandler,
    pub mouse_up_out: SliderMouseUpHandler,
    pub drag_move: SliderDragMoveHandler,
}

pub trait SliderTemplate: Send + Sync {
    fn render(
        &self,
        model: &SliderRenderModel<'_>,
        handlers: SliderTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedSliderTemplate {
    theme: Arc<dyn SliderTheme>,
}

impl ThemedSliderTemplate {
    pub fn new(theme: Arc<dyn SliderTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_slider_template() -> Arc<dyn SliderTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SliderTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedSliderTemplate::new(default_slider_theme()))).clone()
}

impl SliderTemplate for ThemedSliderTemplate {
    fn render(
        &self,
        model: &SliderRenderModel<'_>,
        handlers: SliderTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let SliderTemplateHandlers { track_bounds, hover, mouse_down, mouse_up, mouse_up_out, drag_move } = handlers;
        let appearance = self.theme.resolve(model.state);
        let percentage = model.percentage.clamp(0.0, 1.0);
        let long_axis = appearance.width;
        let short_axis = appearance.height;
        let cross_axis = appearance.track_height;
        let (root_width, root_height, track_left, track_top, track_width, track_height, thumb_left, thumb_top) =
            match model.orientation {
                SliderOrientation::Horizontal => (
                    long_axis,
                    short_axis,
                    0.0,
                    (short_axis - cross_axis) * 0.5,
                    long_axis,
                    cross_axis,
                    (long_axis - appearance.thumb_size).max(0.0) * percentage,
                    (short_axis - appearance.thumb_size) * 0.5,
                ),
                SliderOrientation::Vertical => (
                    short_axis,
                    long_axis,
                    (short_axis - cross_axis) * 0.5,
                    0.0,
                    cross_axis,
                    long_axis,
                    (short_axis - appearance.thumb_size) * 0.5,
                    (long_axis - appearance.thumb_size).max(0.0) * (1.0 - percentage),
                ),
            };

        let track = div()
            .id(format!("{}-track", model.id))
            .absolute()
            .left(px(track_left))
            .top(px(track_top))
            .w(px(track_width))
            .h(px(track_height))
            .bg(appearance.track_background)
            .rounded(px(appearance.radius))
            .overflow_hidden()
            .child(match model.orientation {
                SliderOrientation::Horizontal => div()
                    .absolute()
                    .left(px(0.0))
                    .top(px(0.0))
                    .h_full()
                    .w(px(track_width * percentage))
                    .bg(appearance.fill_background)
                    .rounded(px(appearance.radius)),
                SliderOrientation::Vertical => div()
                    .absolute()
                    .left(px(0.0))
                    .bottom(px(0.0))
                    .w_full()
                    .h(px(track_height * percentage))
                    .bg(appearance.fill_background)
                    .rounded(px(appearance.radius)),
            });

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
            .border_color(focus_ring_color(appearance.focus_ring))
            .rounded(px(appearance.radius + thumb_focus_offset()))
            .child(
                div()
                    .size(px(appearance.thumb_size))
                    .bg(appearance.thumb_background)
                    .border_1()
                    .border_color(appearance.thumb_border)
                    .rounded(px(appearance.radius))
                    .shadow(appearance.thumb_shadow.clone()),
            );

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .w(px(root_width))
            .h(px(root_height))
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out)
            .on_drag(SliderDrag::new(model.id.clone()), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(drag_move)
            .child(track)
            .child(thumb)
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

const THUMB_FOCUS_GAP: f32 = 0.0;
const THUMB_FOCUS_WIDTH: f32 = 2.0;

fn thumb_focus_offset() -> f32 {
    THUMB_FOCUS_GAP + THUMB_FOCUS_WIDTH
}

fn focus_ring_color(focus_ring: Option<Hsla>) -> Hsla {
    focus_ring.unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 0.0))
}
