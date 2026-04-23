use std::sync::{Arc, OnceLock};

use gpui::{App, Div, MouseDownEvent, Stateful, Window, div, px, prelude::*};

use super::TextAreaRenderModel;
use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::theme::{TextAreaTheme, default_textarea_theme};

pub type TextAreaHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type TextAreaMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaClickHandler = Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaKeyDownHandler = Box<dyn Fn(&gpui::KeyDownEvent, &mut Window, &mut App) + 'static>;

pub struct TextAreaTemplateHandlers {
    pub hover: TextAreaHoverHandler,
    pub mouse_down: TextAreaMouseDownHandler,
    pub click: TextAreaClickHandler,
    pub key_down: TextAreaKeyDownHandler,
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
        let has_text = !model.value.is_empty();
        let text = if has_text {
            if model.state.focused {
                let mut chars = model.value.chars().collect::<Vec<_>>();
                let caret = model.state.cursor.min(chars.len());
                chars.insert(caret, '|');
                chars.into_iter().collect::<String>()
            } else {
                model.value.to_string()
            }
        } else if model.state.focused {
            "|".to_string()
        } else {
            model.placeholder.to_string()
        };
        let lines = text
            .split('\n')
            .map(|line| {
                if line.is_empty() {
                    " ".to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>();
        let min_height = appearance
            .min_height
            .max((appearance.typography.line_height * model.rows as f32) + (appearance.padding_y * 2.0));

        let control = div()
            .id(format!("{}-control", model.id))
            .w_full()
            .min_h(px(min_height))
            .flex()
            .flex_col()
            .gap(px(appearance.line_gap))
            .px(px(appearance.padding_x))
            .py(px(appearance.padding_y))
            .bg(appearance.background)
            .border(px(appearance.border_width))
            .border_color(appearance.border)
            .rounded(px(appearance.radius))
            .text_size(px(appearance.typography.size))
            .line_height(px(appearance.typography.line_height))
            .font_weight(appearance.typography.weight)
            .text_color(if has_text {
                appearance.foreground
            } else {
                appearance.placeholder
            })
            .children(lines.into_iter().map(|line| div().child(line)));

        let mut root =
            render_button_family_focus_ring(model.id.clone(), control, appearance.focus_ring, appearance.radius)
                .w_full()
                .on_hover(handlers.hover)
                .on_mouse_down(gpui::MouseButton::Left, handlers.mouse_down)
                .on_click(handlers.click)
                .on_key_down(handlers.key_down);

        if model.enabled {
            root = root.cursor_text();
        } else {
            root = root.cursor_not_allowed().opacity(0.6);
        }

        root
    }
}
