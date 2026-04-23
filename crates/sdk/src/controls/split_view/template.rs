use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, ClickEvent, Div, DragMoveEvent, Hsla, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Stateful,
    Window, div, prelude::*, px,
};

use super::{SplitViewRenderModel, SplitViewSeparatorVisibility, control::SplitViewSeparatorDrag};

const SEPARATOR_HITBOX_WIDTH: f32 = 20.0;
const EXPAND_SEPARATOR_INSET_Y: f32 = 10.0;
const EXPAND_SEPARATOR_CUE_LEFT: f32 = 2.0;
const SEPARATOR_CUE_INSET_Y: f32 = 16.0;
const SEPARATOR_CUE_RADIUS: f32 = 4.0;
const SEPARATOR_CUE_WIDTH: f32 = 4.0;
const SEPARATOR_CUE_HOVERED_WIDTH: f32 = 8.0;
const CENTERED_CUE_OFFSET_FACTOR: f32 = 0.5;

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
            SplitViewSeparatorVisibility::Always if model.separator_hovered => {
                model.separator_hover_color.or(model.separator_color)
            }
            SplitViewSeparatorVisibility::Always => model.separator_color,
            SplitViewSeparatorVisibility::Hover if model.separator_hovered => model.separator_hover_color,
            SplitViewSeparatorVisibility::Hover => None,
        };
        let cue_width = separator_cue_width(model.separator_hovered);
        let resize_enabled = model.enabled && model.resizable && !model.collapsed;

        let mut row = div()
            .size_full()
            .flex()
            .child(div().h_full().w(model.effective_sidebar_width).flex_none().overflow_hidden().child(sidebar));

        let expand_separator = if model.collapsed {
            Some(
                div()
                    .id(format!("{}-expand-separator", model.id))
                    .absolute()
                    .left(collapsed_expand_separator_left(model.effective_sidebar_width))
                    .top(px(EXPAND_SEPARATOR_INSET_Y))
                    .bottom(px(EXPAND_SEPARATOR_INSET_Y))
                    .w(px(SEPARATOR_HITBOX_WIDTH))
                    .on_hover(separator_hover)
                    .on_click(separator_click)
                    .cursor_e_resize()
                    .child(render_separator_cue(EXPAND_SEPARATOR_CUE_LEFT, cue_width, cue_color)),
            )
        } else {
            let mut separator = div()
                .id(format!("{}-separator", model.id))
                .relative()
                .h_full()
                .w(px(SEPARATOR_HITBOX_WIDTH))
                .flex_none()
                .on_hover(separator_hover)
                .on_click(separator_click)
                .on_mouse_down(MouseButton::Left, separator_mouse_down)
                .when(resize_enabled, |this| this.cursor_col_resize())
                .when(!resize_enabled, |this| this.cursor_pointer())
                .child(render_separator_cue(centered_separator_cue_left(cue_width), cue_width, cue_color));

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

        row = row.child(div().h_full().flex_1().overflow_hidden().child(content));

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

fn separator_cue_width(hovered: bool) -> f32 {
    if hovered {
        SEPARATOR_CUE_HOVERED_WIDTH
    } else {
        SEPARATOR_CUE_WIDTH
    }
}

fn centered_separator_cue_left(width: f32) -> f32 {
    (SEPARATOR_HITBOX_WIDTH - width) * CENTERED_CUE_OFFSET_FACTOR
}

pub(crate) fn collapsed_expand_separator_left(effective_sidebar_width: Pixels) -> Pixels {
    effective_sidebar_width.max(px(0.0))
}

fn render_separator_cue(left: f32, width: f32, color: Option<Hsla>) -> Div {
    let mut cue = div()
        .absolute()
        .left(px(left))
        .top(px(SEPARATOR_CUE_INSET_Y))
        .bottom(px(SEPARATOR_CUE_INSET_Y))
        .w(px(width))
        .rounded(px(SEPARATOR_CUE_RADIUS));

    if let Some(color) = color {
        cue = cue.bg(color);
    }

    cue
}

#[cfg(test)]
mod tests {
    use gpui::px;

    use super::collapsed_expand_separator_left;

    #[test]
    fn collapsed_expand_separator_starts_at_effective_sidebar_edge() {
        assert_eq!(collapsed_expand_separator_left(px(0.0)), px(0.0));
        assert_eq!(collapsed_expand_separator_left(px(56.0)), px(56.0));
    }
}
