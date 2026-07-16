use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Entity, FocusHandle, IntoElement, ParentElement, SharedString, Window, div};

use super::control::Toolbar;
use super::template::{ToolbarTemplate, default_toolbar_template, modified_toolbar_template};
use crate::controls::presenter::{HostedContent, Presenter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolbarItemKind {
    Button,
    Toggle,
    Menu,
    Hosted,
    Separator,
}

#[derive(Clone)]
pub struct ToolbarItem {
    pub(crate) id: SharedString,
    pub(crate) kind: ToolbarItemKind,
    pub(crate) enabled: bool,
    pub(crate) focus_handle: Option<FocusHandle>,
    pub(crate) content: Option<Presenter<ToolbarItemRenderModel>>,
}

impl ToolbarItem {
    pub fn button<F, E>(id: impl Into<SharedString>, content: F) -> Self
    where
        F: Fn(&ToolbarItemRenderModel, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        Self::content(id, ToolbarItemKind::Button, content)
    }

    pub fn toggle<F, E>(id: impl Into<SharedString>, content: F) -> Self
    where
        F: Fn(&ToolbarItemRenderModel, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        Self::content(id, ToolbarItemKind::Toggle, content)
    }

    pub fn menu<F, E>(id: impl Into<SharedString>, content: F) -> Self
    where
        F: Fn(&ToolbarItemRenderModel, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        Self::content(id, ToolbarItemKind::Menu, content)
    }

    pub fn hosted<F, E>(id: impl Into<SharedString>, content: F) -> Self
    where
        F: Fn(&ToolbarItemRenderModel, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        Self::content(id, ToolbarItemKind::Hosted, content)
    }

    pub fn separator(id: impl Into<SharedString>) -> Self {
        Self { id: id.into(), kind: ToolbarItemKind::Separator, enabled: false, focus_handle: None, content: None }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }

    fn content<F, E>(id: impl Into<SharedString>, kind: ToolbarItemKind, content: F) -> Self
    where
        F: Fn(&ToolbarItemRenderModel, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        Self {
            id: id.into(),
            kind,
            enabled: true,
            focus_handle: None,
            content: Some(Presenter::new(move |model, window, cx| HostedContent {
                element: content(model, window, cx).into_any_element(),
                focus_handle: None,
            })),
        }
    }
}

#[derive(Clone)]
pub struct ToolbarModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<ToolbarItem>,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn ToolbarTemplate>,
}

#[derive(Clone)]
pub struct ToolbarItemRenderModel {
    pub id: SharedString,
    pub kind: ToolbarItemKind,
    pub index: usize,
    pub enabled: bool,
}

pub struct ToolbarRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: Vec<ToolbarItemRenderModel>,
    pub enabled: bool,
}

pub struct ToolbarBuilder {
    pub(crate) model: ToolbarModel,
}

impl ToolbarBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: ToolbarModel {
                id: id.into(),
                items: Vec::new(),
                enabled: true,
                template: default_toolbar_template(),
            },
        }
    }

    pub fn item(mut self, item: ToolbarItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ToolbarItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn separator(self, id: impl Into<SharedString>) -> Self {
        self.item(ToolbarItem::separator(id))
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn ToolbarTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &ToolbarRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = modified_toolbar_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<Toolbar> {
        cx.new(|cx| Toolbar::from_builder(self, cx))
    }
}

pub(crate) fn fallback_item(model: &ToolbarItemRenderModel) -> AnyElement {
    div().child(model.id.clone()).into_any_element()
}
