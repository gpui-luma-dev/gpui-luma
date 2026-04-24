use std::sync::{Arc, OnceLock};

use gpui::{
    App, AppContext as _, Div, DragMoveEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Stateful, Window, div, px,
    prelude::*,
};

use super::{TextAreaDrag, TextAreaRenderModel};
use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::theme::{TextAreaTheme, default_textarea_theme};

const TEXTAREA_SELECTION_OPACITY: f32 = 0.28;
const TEXTAREA_CARET_WIDTH: f32 = 1.5;
const TEXTAREA_CARET_HEIGHT_EXTRA: f32 = 2.0;
const TEXTAREA_TRAILING_HITBOX_WIDTH: f32 = 4.0;
const TEXTAREA_CARET_EDGE_OFFSET: f32 = 0.0;
const TEXTAREA_MIN_WIDTH: f32 = 0.0;
const TEXTAREA_DISABLED_OPACITY: f32 = 0.6;

pub type TextAreaHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type TextAreaMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaMouseMoveHandler = Box<dyn Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaKeyDownHandler = Box<dyn Fn(&gpui::KeyDownEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaClickHandler = Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaDragMoveHandler = Box<dyn Fn(&DragMoveEvent<TextAreaDrag>, &mut Window, &mut App) + 'static>;

pub struct TextAreaTemplateHandlers {
    pub hover: TextAreaHoverHandler,
    pub mouse_down: TextAreaMouseDownHandler,
    pub mouse_move: TextAreaMouseMoveHandler,
    pub mouse_up: TextAreaMouseUpHandler,
    pub mouse_up_out: TextAreaMouseUpHandler,
    pub click: TextAreaClickHandler,
    pub key_down: TextAreaKeyDownHandler,
    pub drag_move: TextAreaDragMoveHandler,
}

pub trait TextAreaTemplate: Send + Sync {
    fn render(
        &self,
        model: &TextAreaRenderModel<'_>,
        handlers: TextAreaTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedTextAreaTemplate {
    theme: Arc<dyn TextAreaTheme>,
}

impl ThemedTextAreaTemplate {
    pub fn new(theme: Arc<dyn TextAreaTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_textarea_template() -> Arc<dyn TextAreaTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn TextAreaTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedTextAreaTemplate::new(default_textarea_theme()))).clone()
}

impl TextAreaTemplate for ThemedTextAreaTemplate {
    fn render(
        &self,
        model: &TextAreaRenderModel<'_>,
        handlers: TextAreaTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let appearance = self.theme.resolve(model.state, model.enabled);
        let show_placeholder = model.value.is_empty() && !model.state.focused;
        let selection = model.state.selection_range();
        let cursor = model.state.cursor.min(model.value.chars().count());
        let caret_height = appearance.typography.size + TEXTAREA_CARET_HEIGHT_EXTRA;
        let row_height = appearance.typography.line_height;
        let viewport_height = row_height * model.rows.max(1) as f32;

        let text_viewport = if show_placeholder {
            div()
                .min_w(px(TEXTAREA_MIN_WIDTH))
                .w_full()
                .h(px(viewport_height))
                .overflow_hidden()
                .text_color(appearance.placeholder)
                .child(model.placeholder.clone())
        } else {
            let mut lines = div().relative().top(px(-model.vertical_scroll)).flex().flex_col();

            for line in &model.line_metrics {
                let line_chars = line.text.chars().collect::<Vec<_>>();
                let mut row =
                    div().relative().h(px(line.height)).flex().items_center().text_color(appearance.foreground);

                for local_ix in 0..=line_chars.len() {
                    let global_ix = line.start + local_ix;
                    if let Some(ch) = line_chars.get(local_ix) {
                        let selected =
                            selection.map(|(start, end)| global_ix >= start && global_ix < end).unwrap_or(false);
                        let width = line
                            .character_offsets
                            .get(local_ix + 1)
                            .zip(line.character_offsets.get(local_ix))
                            .map(|(next, current)| (next - current).max(0.0))
                            .unwrap_or(0.0);

                        row = row.child(
                            div()
                                .relative()
                                .flex_none()
                                .w(px(width))
                                .h(px(caret_height))
                                .flex()
                                .items_center()
                                .when(selected, |cell| {
                                    cell.bg(appearance.selection_background.opacity(TEXTAREA_SELECTION_OPACITY))
                                })
                                .child(ch.to_string())
                                .when(
                                    model.enabled && model.caret_visible && cursor == global_ix && selection.is_none(),
                                    |cell| {
                                        cell.child(
                                            div()
                                                .absolute()
                                                .left(px(TEXTAREA_CARET_EDGE_OFFSET))
                                                .top(px(TEXTAREA_CARET_EDGE_OFFSET))
                                                .w(px(TEXTAREA_CARET_WIDTH))
                                                .h(px(caret_height))
                                                .bg(appearance.caret),
                                        )
                                    },
                                ),
                        );
                    } else {
                        row = row.child(
                            div()
                                .relative()
                                .flex_none()
                                .w(px(TEXTAREA_TRAILING_HITBOX_WIDTH))
                                .h(px(caret_height))
                                .when(
                                    model.enabled && model.caret_visible && cursor == global_ix && selection.is_none(),
                                    |cell| {
                                        cell.child(
                                            div()
                                                .absolute()
                                                .left(px(TEXTAREA_CARET_EDGE_OFFSET))
                                                .top(px(TEXTAREA_CARET_EDGE_OFFSET))
                                                .w(px(TEXTAREA_CARET_WIDTH))
                                                .h(px(caret_height))
                                                .bg(appearance.caret),
                                        )
                                    },
                                ),
                        );
                    }
                }

                lines = lines.child(row);
            }

            div().min_w(px(TEXTAREA_MIN_WIDTH)).w_full().h(px(viewport_height)).overflow_hidden().child(lines)
        };

        let control = div()
            .id(format!("{}-control", model.id))
            .relative()
            .h(px(appearance.padding_y * 2.0 + viewport_height))
            .flex()
            .items_start()
            .px(px(appearance.padding_x))
            .py(px(appearance.padding_y))
            .bg(appearance.background)
            .border(px(appearance.border_width))
            .border_color(appearance.border)
            .rounded(px(appearance.radius))
            .text_size(px(appearance.typography.size))
            .line_height(px(appearance.typography.line_height))
            .font_family(appearance.font_family.clone())
            .font_weight(appearance.typography.weight)
            .when(model.full_width, |root| root.w_full())
            .when(model.enabled, |root| root.cursor_text())
            .when(!model.enabled, |root| root.cursor_not_allowed().opacity(TEXTAREA_DISABLED_OPACITY))
            .child(text_viewport);

        let mut root =
            render_button_family_focus_ring(model.id.clone(), control, appearance.focus_ring, appearance.radius);

        if model.full_width {
            root = root.w_full();
        }

        root.on_hover(handlers.hover)
            .on_mouse_down(gpui::MouseButton::Left, handlers.mouse_down)
            .on_mouse_move(handlers.mouse_move)
            .on_mouse_up(gpui::MouseButton::Left, handlers.mouse_up)
            .on_mouse_up_out(gpui::MouseButton::Left, handlers.mouse_up_out)
            .on_click(handlers.click)
            .on_key_down(handlers.key_down)
            .on_drag(TextAreaDrag::new(model.id.clone()), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(handlers.drag_move)
    }
}
