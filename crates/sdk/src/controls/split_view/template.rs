use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, ClickEvent, Div, DragMoveEvent, MouseButton, MouseDownEvent, MouseUpEvent, Stateful, Window, div,
    prelude::*, px, rgb,
};

use super::{SplitViewRenderModel, SplitViewSeparatorVisibility, control::SplitViewSeparatorDrag};

pub type SplitViewHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SplitViewMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SplitViewMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type SplitViewClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SplitViewDragMoveHandler =
    Box<dyn Fn(&DragMoveEvent<SplitViewSeparatorDrag>, &mut Window, &mut App) + 'static>;

pub struct SplitViewTemplateHandlers {
    pub separator_hover: SplitViewHoverHandler,
    pub separator_mouse_down: SplitViewMouseDownHandler,
    pub separator_click: SplitViewClickHandler,
    pub mouse_up: SplitViewMouseUpHandler,
    pub mouse_up_out: SplitViewMouseUpHandler,
    pub drag_move: SplitViewDragMoveHandler,
}

pub trait SplitViewTemplate: Send + Sync {
    fn render(
        &self,
        model: &SplitViewRenderModel<'_>,
        sidebar: AnyElement,
        content: AnyElement,
        handlers: SplitViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedSplitViewTemplate;

impl ThemedSplitViewTemplate {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ThemedSplitViewTemplate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_split_view_template() -> Arc<dyn SplitViewTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SplitViewTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedSplitViewTemplate)).clone()
}

impl SplitViewTemplate for ThemedSplitViewTemplate {
    fn render(
        &self,
        model: &SplitViewRenderModel<'_>,
        sidebar: AnyElement,
        content: AnyElement,
        handlers: SplitViewTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let SplitViewTemplateHandlers {
            separator_hover,
            separator_mouse_down,
            separator_click,
            mouse_up,
            mouse_up_out,
            drag_move,
        } = handlers;
        let cue_color = match model.separator_visibility {
            SplitViewSeparatorVisibility::Always => rgb(0xcbd5e1),
            SplitViewSeparatorVisibility::Hover if model.separator_hovered => rgb(0x94a3b8),
            SplitViewSeparatorVisibility::Hover => rgb(0x00000000),
        };
        let cue_width = if model.separator_hovered { 8.0 } else { 4.0 };
        let resize_enabled = model.enabled && model.resizable && !model.collapsed;

        let mut row = div().size_full().flex().child(
            div()
                .h_full()
                .w(model.effective_sidebar_width)
                .flex_none()
                .overflow_hidden()
                .bg(rgb(0xffffff))
                .child(sidebar),
        );

        let expand_separator = if model.collapsed {
            Some(
                div()
                    .id(format!("{}-expand-separator", model.id))
                    .absolute()
                    .left(px(0.0))
                    .top(px(10.0))
                    .bottom(px(10.0))
                    .w(px(20.0))
                    .on_hover(separator_hover)
                    .on_click(separator_click)
                    .cursor_e_resize()
                    .child(
                        div()
                            .absolute()
                            .left(px(2.0))
                            .top(px(16.0))
                            .bottom(px(16.0))
                            .w(px(cue_width))
                            .rounded(px(4.0))
                            .bg(cue_color),
                    ),
            )
        } else {
            let mut separator = div()
                .id(format!("{}-separator", model.id))
                .relative()
                .h_full()
                .w(px(20.0))
                .flex_none()
                .on_hover(separator_hover)
                .on_click(separator_click)
                .on_mouse_down(MouseButton::Left, separator_mouse_down)
                .when(resize_enabled, |this| this.cursor_col_resize())
                .when(!resize_enabled, |this| this.cursor_pointer())
                .child(
                    div()
                        .absolute()
                        .left(px((20.0 - cue_width) * 0.5))
                        .top(px(16.0))
                        .bottom(px(16.0))
                        .w(px(cue_width))
                        .rounded(px(4.0))
                        .bg(cue_color),
                );

            if resize_enabled {
                separator = separator.on_drag(
                    SplitViewSeparatorDrag::new(model.id.clone()),
                    |drag: &SplitViewSeparatorDrag, _, _, cx: &mut App| {
                        cx.stop_propagation();
                        cx.new(|_| drag.clone())
                    },
                );
            }

            row = row.child(separator);
            None
        };

        row = row.child(div().h_full().flex_1().overflow_hidden().bg(rgb(0xf8fafc)).child(content));

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .size_full()
            .overflow_hidden()
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out)
            .on_drag_move(drag_move)
            .child(row);

        if let Some(expand_separator) = expand_separator {
            root = root.child(expand_separator);
        }

        root
    }
}
