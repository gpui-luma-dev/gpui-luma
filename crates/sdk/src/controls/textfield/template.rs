use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Div, DragMoveEvent, FontWeight, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Stateful, Window,
    div, px, svg, prelude::*,
};

use super::{TextFieldDrag, TextFieldLook, TextFieldRenderModel, TextFieldState, TextFieldVariant};
use crate::controls::choice_indicator_layout::reserve_shadow_extent;
use crate::controls::command::button::ControlIcon;
use crate::controls::textfield::{TextFieldTheme, default_textfield_theme};
use crate::theme::{ControlSize, LayoutCacheKey, LumaLayoutCacheExt, StandardBoxScale};

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
pub type TextFieldDragMoveHandler = Box<dyn Fn(&DragMoveEvent<TextFieldDrag>, &mut Window, &mut App) + 'static>;

pub struct TextFieldTemplateHandlers {
    pub hover: TextFieldHoverHandler,
    pub mouse_down: TextFieldMouseDownHandler,
    pub mouse_move: TextFieldMouseMoveHandler,
    pub mouse_up: TextFieldMouseUpHandler,
    pub mouse_up_out: TextFieldMouseUpHandler,
    pub click: TextFieldClickHandler,
    pub key_down: TextFieldKeyDownHandler,
    pub drag_move: TextFieldDragMoveHandler,
}

pub type TextFieldTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &TextFieldRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait TextFieldTemplate: Send + Sync {
    fn resolve_look(
        &self,
        variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
    ) -> TextFieldLook {
        default_textfield_theme().resolve_look(
            variant,
            state,
            enabled,
            size,
            &StandardBoxScale::compute(size, &default_textfield_theme().metrics(), 1.0),
        )
    }

    fn resolve_look_with_scale(
        &self,
        variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        _scale_factor: f32,
        _cx: &mut App,
    ) -> TextFieldLook {
        self.resolve_look(variant, state, enabled, size)
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
    modifiers: Vec<TextFieldTemplateModifier>,
}

impl ThemedTextFieldTemplate {
    pub fn new(theme: Arc<dyn TextFieldTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TextFieldRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &TextFieldRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

struct ModifiedTextFieldTemplate {
    base: Arc<dyn TextFieldTemplate>,
    modifiers: Vec<TextFieldTemplateModifier>,
}

impl ModifiedTextFieldTemplate {
    fn new(base: Arc<dyn TextFieldTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: TextFieldTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &TextFieldRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

pub fn default_textfield_template() -> Arc<dyn TextFieldTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn TextFieldTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedTextFieldTemplate::new(default_textfield_theme()))).clone()
}

pub(super) fn template_with_modifier<F>(template: Arc<dyn TextFieldTemplate>, modifier: F) -> Arc<dyn TextFieldTemplate>
where
    F: Fn(Stateful<Div>, &TextFieldRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTextFieldTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl TextFieldTemplate for ModifiedTextFieldTemplate {
    fn resolve_look(
        &self,
        variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
    ) -> TextFieldLook {
        self.base.resolve_look(variant, state, enabled, size)
    }

    fn resolve_look_with_scale(
        &self,
        variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale_factor: f32,
        cx: &mut App,
    ) -> TextFieldLook {
        self.base.resolve_look_with_scale(variant, state, enabled, size, scale_factor, cx)
    }

    fn render(
        &self,
        model: &TextFieldRenderModel<'_>,
        handlers: TextFieldTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply_modifiers(root, model)
    }
}

impl TextFieldTemplate for ThemedTextFieldTemplate {
    fn resolve_look(
        &self,
        variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
    ) -> TextFieldLook {
        self.theme.resolve_look(
            variant,
            state,
            enabled,
            size,
            &StandardBoxScale::compute(size, &self.theme.metrics(), 1.0),
        )
    }

    fn resolve_look_with_scale(
        &self,
        variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale_factor: f32,
        cx: &mut App,
    ) -> TextFieldLook {
        let scale = cx.use_cached_layout(
            self.theme.metrics(),
            LayoutCacheKey { size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(size, metrics, scale_factor),
        );
        self.theme.resolve_look(variant, state, enabled, size, &scale)
    }

    fn render(
        &self,
        model: &TextFieldRenderModel<'_>,
        handlers: TextFieldTemplateHandlers,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let scale_factor = window.scale_factor();
        let look = model.look.clone();
        let show_placeholder = model.value.is_empty() && !model.state.focused;
        let chars = model.value.chars().collect::<Vec<_>>();
        let cursor = model.state.cursor.min(chars.len());
        let selection = model.state.selection_range();
        let caret_height = look.typography.size + TEXTFIELD_CARET_HEIGHT_EXTRA;

        let mut text_viewport = if show_placeholder {
            div()
                .min_w(px(0.0))
                .flex_1()
                .flex()
                .items_center()
                .overflow_hidden()
                .text_color(look.placeholder)
                .text_size(px(look.typography.size))
                .line_height(px(look.typography.line_height))
                .font_family(look.font_family.clone())
                .font_weight(look.typography.weight)
                .child(
                    div()
                        .h(px(caret_height))
                        .flex()
                        .items_center()
                        .text_size(px(look.typography.size))
                        .line_height(px(look.typography.line_height))
                        .font_family(look.font_family.clone())
                        .font_weight(look.typography.weight)
                        .child(model.placeholder.clone()),
                )
        } else {
            let mut row = div()
                .min_w(px(0.0))
                .flex()
                .items_center()
                .text_color(look.foreground)
                .text_size(px(look.typography.size))
                .line_height(px(look.typography.line_height))
                .font_family(look.font_family.clone())
                .font_weight(look.typography.weight);

            for caret_ix in 0..=chars.len() {
                let in_selection = selection.map(|(start, end)| caret_ix >= start && caret_ix < end).unwrap_or(false);
                let char_color = if in_selection {
                    look.selection_foreground
                } else {
                    look.foreground
                };

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
                            .when(in_selection, |cell| {
                                cell.bg(look.selection_background).text_color(look.selection_foreground)
                            })
                            .when(!in_selection, |cell| cell.text_color(char_color))
                            .child(ch.to_string())
                            .when(
                                model.enabled && model.caret_visible && cursor == caret_ix && selection.is_none(),
                                |cell| {
                                    cell.child(
                                        div()
                                            .absolute()
                                            .left(px(CARET_EDGE_OFFSET))
                                            .top(px(CARET_EDGE_OFFSET))
                                            .w(px(TEXTFIELD_CARET_WIDTH))
                                            .h(px(caret_height))
                                            .bg(look.caret),
                                    )
                                },
                            ),
                    );
                } else {
                    row = row.child(
                        div().relative().flex_none().w(px(TEXTFIELD_TRAILING_HITBOX_WIDTH)).h(px(caret_height)).when(
                            model.enabled && model.caret_visible && cursor == caret_ix && selection.is_none(),
                            |cell| {
                                cell.child(
                                    div()
                                        .absolute()
                                        .left(px(CARET_EDGE_OFFSET))
                                        .top(px(CARET_EDGE_OFFSET))
                                        .w(px(TEXTFIELD_CARET_WIDTH))
                                        .h(px(caret_height))
                                        .bg(look.caret),
                                )
                            },
                        ),
                    );
                }
            }

            div()
                .min_w(px(0.0))
                .flex_1()
                .overflow_hidden()
                .child(div().relative().left(px(-model.horizontal_scroll)).flex().items_center().child(row))
        };

        if let Some(icon) = model.prefix_icon {
            text_viewport = div()
                .min_w(px(0.0))
                .flex_1()
                .flex()
                .items_center()
                .gap(px(look.gap))
                .child(render_prefix_icon(icon, look.icon, look.icon_size))
                .child(text_viewport);
        }

        // Always fill the focus-ring root so a stretched ring cannot leave a content-sized
        // entry box centered inside it (e.g. fixed-width hosts without full_width).
        let mut control = div()
            .id(format!("{}-control", model.id))
            .relative()
            .w_full()
            .min_w(px(0.0))
            .h(px(look.min_height))
            .flex()
            .items_center()
            .px(px(look.padding_x))
            .py(px(look.padding_y))
            .bg(look.background)
            .border(px(look.border_width))
            .border_color(look.border)
            .rounded(px(look.radius))
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .font_family(look.font_family.clone())
            .font_weight(look.typography.weight)
            .when(model.enabled, |root| root.cursor_text())
            .when(!model.enabled, |root| root.cursor_not_allowed().opacity(0.6))
            .child(text_viewport);

        let has_elevation = look.shadow.as_ref().is_some_and(|shadows| !shadows.is_empty());
        if model.enabled
            && let Some(shadows) = look.shadow.as_ref().filter(|shadows| !shadows.is_empty())
        {
            control = control.shadow(shadows.clone());
        }

        let shadow_extent = reserve_shadow_extent(look.shadow.as_ref(), None, scale_factor, has_elevation);
        let oversize_extent = shadow_extent;
        let mut root = div().id(model.id.clone()).relative().child(control);
        if oversize_extent > 0.0 {
            root = div().id(format!("{}-elevation", model.id)).relative().p(px(oversize_extent)).child(root);
        }

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
            .on_key_down(handlers.key_down)
            .on_drag(TextFieldDrag::new(model.id.clone()), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(handlers.drag_move);

        self.apply_modifiers(root, model)
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
