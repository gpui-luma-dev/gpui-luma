use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Stateful, Window, div, px, prelude::*};

use super::model::{ToolbarItemKind, ToolbarRenderModel, fallback_item};
use super::theme::{ToolbarTheme, default_toolbar_theme};
use crate::theme::ControlSize;

pub type ToolbarTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &ToolbarRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait ToolbarTemplate: Send + Sync {
    fn render(
        &self,
        model: &ToolbarRenderModel<'_>,
        items: Vec<ToolbarRenderedItem>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ToolbarRenderedItem {
    pub kind: ToolbarItemKind,
    pub element: gpui::AnyElement,
}

pub struct ThemedToolbarTemplate {
    theme: Arc<dyn ToolbarTheme>,
    size: ControlSize,
    modifiers: Vec<ToolbarTemplateModifier>,
}

impl ThemedToolbarTemplate {
    pub fn new(theme: Arc<dyn ToolbarTheme>) -> Self {
        Self { theme, size: ControlSize::Md, modifiers: Vec::new() }
    }

    pub fn with_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ToolbarRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &ToolbarRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

struct ModifiedToolbarTemplate {
    base: Arc<dyn ToolbarTemplate>,
    modifiers: Vec<ToolbarTemplateModifier>,
}

impl ModifiedToolbarTemplate {
    fn new(base: Arc<dyn ToolbarTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: ToolbarTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &ToolbarRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

impl ToolbarTemplate for ThemedToolbarTemplate {
    fn render(
        &self,
        model: &ToolbarRenderModel<'_>,
        items: Vec<ToolbarRenderedItem>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let look = self.theme.resolve(model.enabled, self.size);
        let mut root = div()
            .id(model.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(px(look.gap))
            .px(px(look.padding_x))
            .py(px(look.padding_y))
            .rounded(px(look.radius))
            .bg(look.background)
            .border_1()
            .border_color(look.border);

        for item in items {
            let child = match item.kind {
                ToolbarItemKind::Separator => {
                    div().w(px(1.0)).h(px(look.separator_height)).flex_none().bg(look.separator).into_any_element()
                }
                _ => item.element,
            };
            root = root.child(child);
        }

        self.apply_modifiers(root, model)
    }
}

impl ToolbarTemplate for ModifiedToolbarTemplate {
    fn render(
        &self,
        model: &ToolbarRenderModel<'_>,
        items: Vec<ToolbarRenderedItem>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, items, window, cx);
        self.apply_modifiers(root, model)
    }
}

pub fn default_toolbar_template() -> Arc<dyn ToolbarTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ToolbarTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(ThemedToolbarTemplate::new(default_toolbar_theme()))).clone()
}

pub fn toolbar_template_with_theme(theme: Arc<dyn ToolbarTheme>) -> Arc<dyn ToolbarTemplate> {
    Arc::new(ThemedToolbarTemplate::new(theme))
}

pub(super) fn modified_toolbar_template<F>(template: Arc<dyn ToolbarTemplate>, modifier: F) -> Arc<dyn ToolbarTemplate>
where
    F: Fn(Stateful<Div>, &ToolbarRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedToolbarTemplate::new(template).with_modifier(Box::new(modifier)))
}

pub(crate) fn render_fallback_toolbar_item(model: &super::model::ToolbarItemRenderModel) -> gpui::AnyElement {
    fallback_item(model)
}
