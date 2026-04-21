use std::sync::Arc;

use gpui::{AppContext, Entity, Pixels, Point, SharedString};

use super::{
    ContextMenu, ContextMenuState, ContextMenuTemplate, ControlFocusState, MenuPath, default_context_menu_template,
};
use crate::controls::popup_menu::PopupMenuItem;

#[derive(Clone)]
pub struct ContextMenuModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) items: Vec<PopupMenuItem>,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn ContextMenuTemplate>,
}

pub struct ContextMenuRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub items: &'a [PopupMenuItem],
    pub menu_position: Option<Point<Pixels>>,
    pub open_submenu: Option<usize>,
    pub active_path: Option<MenuPath>,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub state: ContextMenuState,
}

pub struct ContextMenuBuilder {
    pub(crate) model: ContextMenuModel,
}

impl ContextMenuBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: ContextMenuModel {
                label: id.clone(),
                id,
                items: Vec::new(),
                enabled: true,
                template: default_context_menu_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn item(mut self, item: PopupMenuItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = PopupMenuItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn ContextMenuTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ContextMenu> {
        cx.new(|cx| ContextMenu::from_builder(self, cx))
    }
}
