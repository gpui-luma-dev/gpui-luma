use std::sync::{Arc, OnceLock};

use gpui::{
    App, AppContext as _, Bounds, Div, DragMoveEvent, MouseButton, MouseDownEvent, MouseUpEvent,
    Pixels, Stateful, Window, canvas, div, px, prelude::*,
};

use super::{SliderDrag, SliderRenderModel};
use crate::theme::{SliderTheme, default_slider_theme};

pub type SliderBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type SliderHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SliderMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SliderMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type SliderDragMoveHandler =
    Box<dyn Fn(&DragMoveEvent<SliderDrag>, &mut Window, &mut App) + 'static>;

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

    TEMPLATE
        .get_or_init(|| Arc::new(ThemedSliderTemplate::new(default_slider_theme())))
        .clone()
}

impl SliderTemplate for ThemedSliderTemplate {
    fn render(
        &self,
        model: &SliderRenderModel<'_>,
        handlers: SliderTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let SliderTemplateHandlers {
            track_bounds,
            hover,
            mouse_down,
            mouse_up,
            mouse_up_out,
            drag_move,
        } = handlers;
        let appearance = self.theme.resolve(model.state);
        let percentage = model.percentage.clamp(0.0, 1.0);
        let fill_width = appearance.width * percentage;
        let thumb_left = (appearance.width - appearance.thumb_size).max(0.0) * percentage;

        let track = div()
            .id(format!("{}-track", model.id))
            .relative()
            .w(px(appearance.width))
            .h(px(appearance.track_height))
            .bg(appearance.track_background)
            .rounded(px(appearance.radius))
            .overflow_hidden()
            .child(
                div()
                    .absolute()
                    .left(px(0.0))
                    .top(px(0.0))
                    .h_full()
                    .w(px(fill_width))
                    .bg(appearance.fill_background)
                    .rounded(px(appearance.radius)),
            );

        let thumb = div()
            .id(format!("{}-thumb", model.id))
            .absolute()
            .left(px(thumb_left))
            .top(px((appearance.height - appearance.thumb_size) * 0.5))
            .size(px(appearance.thumb_size))
            .bg(appearance.thumb_background)
            .border_1()
            .border_color(appearance.fill_background)
            .rounded(px(appearance.radius))
            .shadow_sm();

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .w(px(appearance.width))
            .h(px(appearance.height))
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
                canvas(
                    move |bounds, window, cx| track_bounds(&bounds, window, cx),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            );

        if model.enabled {
            root = root.cursor_pointer();
        } else {
            root = root.opacity(0.56);
        }

        if let Some(focus_ring) = appearance.focus_ring {
            root = root.focus_visible(move |style| style.border_1().border_color(focus_ring));
        }

        root
    }
}
