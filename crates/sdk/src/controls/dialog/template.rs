use std::sync::{Arc, OnceLock};

use gpui::{App, Div, KeyDownEvent, MouseButton, MouseDownEvent, Stateful, Window, div, px, prelude::*};

use super::{DialogLook, DialogRenderModel, DialogTheme, default_dialog_theme};

pub type DialogMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type DialogKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
pub type DialogClickHandler = Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>;

pub struct DialogTemplateHandlers {
    pub close_click: DialogClickHandler,
    pub key_down: DialogKeyDownHandler,
    pub header_mouse_down: DialogMouseDownHandler,
    pub shell_mouse_down: DialogMouseDownHandler,
    pub shell_mouse_down_out: DialogMouseDownHandler,
}

pub struct DialogTemplateParts {
    pub close_button: gpui::AnyElement,
    pub cancel_button: gpui::AnyElement,
    pub confirm_button: gpui::AnyElement,
    pub header_drag_handle: Option<gpui::AnyElement>,
    pub handlers: DialogTemplateHandlers,
}

pub trait DialogTemplate: Send + Sync {
    #[allow(clippy::too_many_arguments)]
    fn render(
        &self,
        model: &DialogRenderModel<'_>,
        look: &DialogLook,
        parts: DialogTemplateParts,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedDialogTemplate {
    theme: Arc<dyn DialogTheme>,
}

impl ThemedDialogTemplate {
    pub fn new(theme: Arc<dyn DialogTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_dialog_template() -> Arc<dyn DialogTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn DialogTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedDialogTemplate::new(default_dialog_theme()))).clone()
}

impl DialogTemplate for ThemedDialogTemplate {
    fn render(
        &self,
        model: &DialogRenderModel<'_>,
        _look: &DialogLook,
        parts: DialogTemplateParts,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let look = self.theme.resolve(model.size, model.mode);
        let DialogTemplateParts { close_button, cancel_button, confirm_button, header_drag_handle, handlers } = parts;
        let DialogTemplateHandlers { close_click, key_down, header_mouse_down, shell_mouse_down, shell_mouse_down_out } =
            handlers;

        let mut shell = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .flex_col()
            .gap(px(look.section_gap))
            .w(px(model.width.unwrap_or(look.min_width).clamp(look.min_width, look.max_width)))
            .max_w(px(look.max_width))
            .p(px(look.padding))
            .bg(look.background)
            .border_1()
            .border_color(look.border)
            .rounded(px(look.radius))
            .shadow(look.shadow.clone())
            .text_color(look.body_color)
            .font_family(look.font_family.clone())
            .text_size(px(look.body.size))
            .line_height(px(look.body.line_height))
            .font_weight(look.body.weight)
            .on_key_down(key_down)
            .on_mouse_down(MouseButton::Left, shell_mouse_down)
            .on_mouse_down_out(shell_mouse_down_out);

        let mut title_block = div().w_full().min_w_0().flex().flex_col().gap(px(look.header_gap)).pr(px(56.0));
        title_block = title_block.child(
            div()
                .w_full()
                .text_size(px(look.title.size))
                .line_height(px(look.title.line_height))
                .font_weight(look.title.weight)
                .text_color(look.title_color)
                .child(model.title.clone()),
        );

        if let Some(description) = model.description {
            title_block = title_block.child(
                div()
                    .w_full()
                    .text_size(px(look.description.size))
                    .line_height(px(look.description.line_height))
                    .font_weight(look.description.weight)
                    .text_color(look.description_color)
                    .child(description.clone()),
            );
        }

        if model.dismissible {
            let _ = close_click;
            shell = shell.child(div().absolute().top(px(look.padding)).right(px(look.padding)).child(close_button));
        }

        if let Some(header_drag_handle) = header_drag_handle {
            let _ = header_mouse_down;
            shell = shell.child(header_drag_handle);
        }

        shell = shell.child(title_block);
        shell = shell.child(div().w_full().min_w_0().child((model.body)(model, window, cx)));

        if model.show_footer {
            shell = shell.child(
                div()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(px(look.footer_gap))
                    .child(cancel_button)
                    .child(confirm_button),
            );
        }

        shell
    }
}
