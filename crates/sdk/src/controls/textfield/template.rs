use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Div, FontWeight, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Stateful, Window, div, px, svg,
    prelude::*,
};

use super::{TextFieldAppearance, TextFieldRenderModel, TextFieldState, TextFieldVariant};
use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::controls::command::button::ControlIcon;
use crate::controls::textfield::{TextFieldTheme, default_textfield_theme};

const TEXTFIELD_SELECTION_OPACITY: f32 = 0.28;
const TEXTFIELD_CARET_WIDTH: f32 = 1.5;
const TEXTFIELD_CARET_HEIGHT_EXTRA: f32 = 2.0;
const TEXTFIELD_TRAILING_HITBOX_WIDTH: f32 = 4.0;
const CARET_EDGE_OFFSET: f32 = 0.0;

pub type TextFieldHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type TextFieldMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type TextFieldMouseMoveHandler = Box<dyn Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static>;
pub type TextFieldMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type TextFieldKeyDownHandler = Box<dyn Fn(&gpui::KeyDownEvent, &mut Window, &mut App) + 'static>;
pub type TextFieldClickHandler = Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>;

pub struct TextFieldTemplateHandlers {
    pub hover: TextFieldHoverHandler,
    pub mouse_down: TextFieldMouseDownHandler,
    pub mouse_move: TextFieldMouseMoveHandler,
    pub mouse_up: TextFieldMouseUpHandler,
    pub mouse_up_out: TextFieldMouseUpHandler,
    pub click: TextFieldClickHandler,
    pub key_down: TextFieldKeyDownHandler,
}

pub trait TextFieldTemplate: Send + Sync {
    fn resolve_appearance(
        &self,
        variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
    ) -> TextFieldAppearance {
        default_textfield_theme().resolve(variant, state, enabled)
    }

    fn render(
        &self,
        model: &TextFieldRenderModel<'_>,
        handlers: TextFieldTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedTextFieldTemplate {
    theme: Arc<dyn TextFieldTheme>,
}

impl ThemedTextFieldTemplate {
    pub fn new(theme: Arc<dyn TextFieldTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_textfield_template() -> Arc<dyn TextFieldTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn TextFieldTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedTextFieldTemplate::new(default_textfield_theme()))).clone()
}

impl TextFieldTemplate for ThemedTextFieldTemplate {
    fn resolve_appearance(
        &self,
        variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
    ) -> TextFieldAppearance {
        self.theme.resolve(variant, state, enabled)
    }

    fn render(
        &self,
        model: &TextFieldRenderModel<'_>,
        handlers: TextFieldTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let appearance = model.appearance;
        let show_placeholder = model.value.is_empty() && !model.state.focused;
        let chars = model.value.chars().collect::<Vec<_>>();
        let cursor = model.state.cursor.min(chars.len());
        let selection = model.state.selection_range();
        let caret_height = appearance.typography.size + TEXTFIELD_CARET_HEIGHT_EXTRA;

        let mut text_viewport = if show_placeholder {
            div()
                .min_w(px(0.0))
                .flex_1()
                .flex()
                .items_center()
                .overflow_hidden()
                .text_color(appearance.placeholder)
                .child(div().h(px(caret_height)).flex().items_center().child(model.placeholder.clone()))
        } else {
            let mut row = div().min_w(px(0.0)).flex().items_center().text_color(appearance.foreground);

            for caret_ix in 0..=chars.len() {
                if let Some(ch) = chars.get(caret_ix) {
                    let width = model
                        .character_offsets
                        .get(caret_ix + 1)
                        .zip(model.character_offsets.get(caret_ix))
                        .map(|(next, current)| (next - current).max(0.0))
                        .unwrap_or(0.0);

                    row = row.child(
                        div()
                            .relative()
                            .flex_none()
                            .w(px(width))
                            .flex()
                            .items_center()
                            .h(px(caret_height))
                            .child(ch.to_string())
                            .when(
                                model.enabled && model.caret_visible && cursor == caret_ix && !selection.is_some(),
                                |cell| {
                                    cell.child(
                                        div()
                                            .absolute()
                                            .left(px(CARET_EDGE_OFFSET))
                                            .top(px(CARET_EDGE_OFFSET))
                                            .w(px(TEXTFIELD_CARET_WIDTH))
                                            .h(px(caret_height))
                                            .bg(appearance.caret),
                                    )
                                },
                            ),
                    );
                } else {
                    row = row.child(
                        div().relative().flex_none().w(px(TEXTFIELD_TRAILING_HITBOX_WIDTH)).h(px(caret_height)).when(
                            model.enabled && model.caret_visible && cursor == caret_ix && !selection.is_some(),
                            |cell| {
                                cell.child(
                                    div()
                                        .absolute()
                                        .left(px(CARET_EDGE_OFFSET))
                                        .top(px(CARET_EDGE_OFFSET))
                                        .w(px(TEXTFIELD_CARET_WIDTH))
                                        .h(px(caret_height))
                                        .bg(appearance.caret),
                                )
                            },
                        ),
                    );
                }
            }

            let selection_overlay = selection.and_then(|(start, end)| {
                let start_x = model.character_offsets.get(start).copied().unwrap_or(0.0);
                let end_x = model.character_offsets.get(end).copied().unwrap_or(start_x);
                let width = (end_x - start_x).max(0.0);
                (width > 0.0).then(|| {
                    div()
                        .absolute()
                        .left(px(start_x))
                        .top(px(0.0))
                        .w(px(width))
                        .h(px(caret_height))
                        .bg(appearance.selection_background.opacity(TEXTFIELD_SELECTION_OPACITY))
                })
            });

            div().min_w(px(0.0)).flex_1().overflow_hidden().child(
                div()
                    .relative()
                    .left(px(-model.horizontal_scroll))
                    .flex()
                    .items_center()
                    .when_some(selection_overlay, |text, overlay| text.child(overlay))
                    .child(row),
            )
        };

        if let Some(icon) = model.prefix_icon {
            text_viewport = div()
                .min_w(px(0.0))
                .flex_1()
                .flex()
                .items_center()
                .gap(px(appearance.gap))
                .child(render_prefix_icon(icon, appearance.icon, appearance.icon_size))
                .child(text_viewport);
        }

        let control = div()
            .id(format!("{}-control", model.id))
            .relative()
            .h(px(appearance.min_height))
            .flex()
            .items_center()
            .px(px(appearance.padding_x))
            .py(px(appearance.padding_y))
            .bg(appearance.background)
            .border(px(appearance.border_width))
            .border_color(appearance.border)
            .rounded(px(appearance.radius))
            .text_size(px(appearance.typography.size))
            .line_height(px(appearance.typography.line_height))
            .font_weight(appearance.typography.weight)
            .when(model.full_width, |root| root.w_full())
            .when(model.enabled, |root| root.cursor_text())
            .when(!model.enabled, |root| root.cursor_not_allowed().opacity(0.6))
            .child(text_viewport);

        let mut root =
            render_button_family_focus_ring(model.id.clone(), control, appearance.focus_ring, appearance.radius);

        if model.full_width {
            root = root.w_full();
        }

        root = root
            .on_hover(handlers.hover)
            .on_mouse_down(gpui::MouseButton::Left, handlers.mouse_down)
            .on_mouse_move(handlers.mouse_move)
            .on_mouse_up(gpui::MouseButton::Left, handlers.mouse_up)
            .on_mouse_up_out(gpui::MouseButton::Left, handlers.mouse_up_out)
            .on_click(handlers.click)
            .on_key_down(handlers.key_down);

        root
    }
}

fn render_prefix_icon(icon: &ControlIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    match icon {
        ControlIcon::Lucide(icon) => div()
            .size(px(size))
            .flex()
            .items_center()
            .justify_center()
            .font_family("lucide")
            .font_weight(FontWeight::NORMAL)
            .text_size(px(size))
            .line_height(px(size))
            .text_color(color)
            .child(char::from(*icon).to_string())
            .into_any_element(),
        ControlIcon::SvgPath(path) => svg().size(px(size)).text_color(color).path(path.clone()).into_any_element(),
    }
}
