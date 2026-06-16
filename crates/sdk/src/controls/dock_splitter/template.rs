use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Hsla, MouseButton, MouseDownEvent, MouseUpEvent, Stateful, Window, div, px, prelude::*};

use super::{DockSplitterAppearance, DockSplitterDrag, DockSplitterRenderModel, SplitterOrientation};

pub type DockSplitterHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type DockSplitterMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type DockSplitterMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub struct DockSplitterTemplateHandlers {
    pub hover: DockSplitterHoverHandler,
    pub mouse_down: DockSplitterMouseDownHandler,
    pub mouse_up: DockSplitterMouseUpHandler,
    pub mouse_up_out: DockSplitterMouseUpHandler,
}

pub trait DockSplitterTemplate: Send + Sync {
    fn render(
        &self,
        model: &DockSplitterRenderModel<'_>,
        appearance: &DockSplitterAppearance,
        handlers: DockSplitterTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedDockSplitterTemplate;

impl ThemedDockSplitterTemplate {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ThemedDockSplitterTemplate {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ThumbDockSplitterTemplate {
    thumb_color: Hsla,
}

impl ThumbDockSplitterTemplate {
    pub fn new(thumb_color: Hsla) -> Self {
        Self { thumb_color }
    }
}

pub fn default_dock_splitter_template() -> Arc<dyn DockSplitterTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn DockSplitterTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedDockSplitterTemplate)).clone()
}

impl DockSplitterTemplate for ThemedDockSplitterTemplate {
    fn render(
        &self,
        model: &DockSplitterRenderModel<'_>,
        appearance: &DockSplitterAppearance,
        handlers: DockSplitterTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        render_splitter(model, appearance, handlers, None)
    }
}

impl DockSplitterTemplate for ThumbDockSplitterTemplate {
    fn render(
        &self,
        model: &DockSplitterRenderModel<'_>,
        appearance: &DockSplitterAppearance,
        handlers: DockSplitterTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        render_splitter(model, appearance, handlers, Some(self.thumb_color))
    }
}

fn render_splitter(
    model: &DockSplitterRenderModel<'_>,
    appearance: &DockSplitterAppearance,
    handlers: DockSplitterTemplateHandlers,
    thumb_color: Option<Hsla>,
) -> Stateful<Div> {
    let DockSplitterTemplateHandlers { hover, mouse_down, mouse_up, mouse_up_out } = handlers;
    let line_color = if model.dragging || model.hovered {
        appearance.hover_color
    } else {
        appearance.line_color
    };
    let half_inset = ((appearance.hit_target_px - appearance.visible_line_px) * 0.5).max(0.0);

    let mut root = div().id(format!("{}-layout", model.id)).relative().flex_shrink_0();

    root = match model.orientation {
        SplitterOrientation::Vertical => root.w(px(appearance.visible_line_px)).h_full(),
        SplitterOrientation::Horizontal => root.h(px(appearance.visible_line_px)).w_full(),
    };

    let drag_payload = DockSplitterDrag { id: model.id.clone() };
    let mut hit_target = match model.orientation {
        SplitterOrientation::Vertical => div()
            .id(model.id.clone())
            .absolute()
            .left(px(-half_inset))
            .top(px(0.0))
            .bottom(px(0.0))
            .w(px(appearance.hit_target_px))
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out)
            .on_drag(drag_payload.clone(), {
                let drag = drag_payload.clone();
                move |_: &DockSplitterDrag, _, _, cx| {
                    cx.stop_propagation();
                    cx.new(|_| drag.clone())
                }
            })
            .when(model.enabled, |this| this.cursor_col_resize())
            .child(
                div()
                    .absolute()
                    .left(px(half_inset))
                    .top(px(0.0))
                    .bottom(px(0.0))
                    .w(px(appearance.visible_line_px))
                    .bg(line_color),
            ),
        SplitterOrientation::Horizontal => div()
            .id(model.id.clone())
            .absolute()
            .top(px(-half_inset))
            .left(px(0.0))
            .right(px(0.0))
            .h(px(appearance.hit_target_px))
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out)
            .on_drag(drag_payload, |drag: &DockSplitterDrag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .when(model.enabled, |this| this.cursor_row_resize())
            .child(
                div()
                    .absolute()
                    .top(px(half_inset))
                    .left(px(0.0))
                    .right(px(0.0))
                    .h(px(appearance.visible_line_px))
                    .bg(line_color),
            ),
    };

    if let (SplitterOrientation::Vertical, Some(thumb_color)) = (model.orientation, thumb_color) {
        hit_target = hit_target.child(
            div().absolute().inset_0().flex().justify_center().items_center().child(
                div()
                    .rounded(px(8.0))
                    .bg(if model.hovered || model.dragging {
                        thumb_color
                    } else {
                        thumb_color.opacity(0.4)
                    })
                    .w(px(4.0))
                    .h(px(36.0)),
            ),
        );
    }

    root.child(hit_target)
}
