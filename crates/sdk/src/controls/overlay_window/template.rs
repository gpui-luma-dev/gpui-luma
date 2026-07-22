use std::sync::{Arc, OnceLock};

use gpui::{App, Div, KeyDownEvent, MouseButton, MouseDownEvent, Stateful, Window, div, px, prelude::*};

use super::{DialogLook, DialogRenderModel, DialogTheme, default_dialog_theme};

pub type DialogMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type DialogKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;

pub struct DialogTemplateHandlers {
    pub key_down: DialogKeyDownHandler,
    pub header_mouse_down: DialogMouseDownHandler,
    pub shell_mouse_down: DialogMouseDownHandler,
    pub shell_mouse_down_out: DialogMouseDownHandler,
}

pub type DialogTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &DialogRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub struct DialogTemplateParts {
    pub header_drag_handle: Option<gpui::AnyElement>,
    pub handlers: DialogTemplateHandlers,
}

pub trait DialogTemplate: Send + Sync {
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
    modifiers: Vec<DialogTemplateModifier>,
}

impl ThemedDialogTemplate {
    pub fn new(theme: Arc<dyn DialogTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &DialogRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &DialogRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

struct ModifiedDialogTemplate {
    base: Arc<dyn DialogTemplate>,
    modifiers: Vec<DialogTemplateModifier>,
}

impl ModifiedDialogTemplate {
    fn new(base: Arc<dyn DialogTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: DialogTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &DialogRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

pub fn default_dialog_template() -> Arc<dyn DialogTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn DialogTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedDialogTemplate::new(default_dialog_theme()))).clone()
}

pub(super) fn modified_dialog_template<F>(template: Arc<dyn DialogTemplate>, modifier: F) -> Arc<dyn DialogTemplate>
where
    F: Fn(Stateful<Div>, &DialogRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedDialogTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl DialogTemplate for ModifiedDialogTemplate {
    fn render(
        &self,
        model: &DialogRenderModel<'_>,
        look: &DialogLook,
        parts: DialogTemplateParts,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, look, parts, window, cx);
        self.apply_modifiers(root, model)
    }
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
        let DialogTemplateParts { header_drag_handle, handlers } = parts;
        let DialogTemplateHandlers { key_down, header_mouse_down, shell_mouse_down, shell_mouse_down_out } = handlers;

        let shell_width = match model.width {
            Some(width) => width.max(look.min_width),
            None => look.min_width.clamp(look.min_width, look.max_width),
        };

        let mut shell = div()
            .id(model.id.clone())
            .relative()
            .w(px(shell_width))
            .when(model.width.is_none(), |shell| shell.max_w(px(look.max_width)))
            .p(px(look.padding))
            .bg(look.background)
            .border_1()
            .border_color(look.border)
            .rounded(px(look.radius))
            .shadow(look.shadow.clone())
            .text_color(look.foreground)
            .font_family(look.font_family.clone())
            .text_size(px(look.body.size))
            .line_height(px(look.body.line_height))
            .font_weight(look.body.weight)
            .on_key_down(key_down)
            .on_mouse_down(MouseButton::Left, shell_mouse_down)
            .on_mouse_down_out(shell_mouse_down_out);

        if let Some(header_drag_handle) = header_drag_handle {
            let _ = header_mouse_down;
            shell = shell.child(header_drag_handle);
        }

        let root = shell.child(div().w_full().min_w_0().child((model.content)(model, window, cx)));
        self.apply_modifiers(root, model)
    }
}
