use std::sync::{Arc, OnceLock};

use gpui::{App, Div, MouseButton, MouseDownEvent, MouseUpEvent, Stateful, Window, div, px, prelude::*};

use super::{DockSplitterAppearance, DockSplitterRenderModel, SplitterOrientation};

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
        let DockSplitterTemplateHandlers { hover, mouse_down, mouse_up, mouse_up_out } = handlers;
        let line_color = if model.dragging || model.hovered {
            appearance.hover_color
        } else {
            appearance.line_color
        };
        let half_inset = ((appearance.hit_target_px - appearance.visible_line_px) * 0.5).max(0.0);

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex_shrink_0()
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out);

        root = match model.orientation {
            SplitterOrientation::Vertical => {
                root.w(px(appearance.hit_target_px)).h_full().when(model.enabled, |this| this.cursor_col_resize())
            }
            SplitterOrientation::Horizontal => {
                root.h(px(appearance.hit_target_px)).w_full().when(model.enabled, |this| this.cursor_row_resize())
            }
        };

        let line = match model.orientation {
            SplitterOrientation::Vertical => div()
                .absolute()
                .left(px(half_inset))
                .top(px(0.0))
                .bottom(px(0.0))
                .w(px(appearance.visible_line_px))
                .bg(line_color),
            SplitterOrientation::Horizontal => div()
                .absolute()
                .top(px(half_inset))
                .left(px(0.0))
                .right(px(0.0))
                .h(px(appearance.visible_line_px))
                .bg(line_color),
        };

        root.child(line)
    }
}
