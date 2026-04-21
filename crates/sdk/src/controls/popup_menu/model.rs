use std::sync::Arc;

use gpui::{AppContext, Bounds, Entity, Pixels, SharedString};

use super::{ControlFocusState, PopupMenu, PopupMenuState, PopupMenuTemplate, MenuPath, default_popup_menu_template};
use crate::controls::menu_item::MenuItem;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PopupMenuPlacement {
    Smart,
    BelowStart,
    AboveStart,
    CenteredOnTrigger,
}

#[derive(Clone)]
pub struct PopupMenuModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) items: Vec<MenuItem>,
    pub(crate) enabled: bool,
    pub(crate) placement: PopupMenuPlacement,
    pub(crate) template: Arc<dyn PopupMenuTemplate>,
}

pub struct PopupMenuRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub items: &'a [MenuItem],
    pub open: bool,
    pub trigger_bounds: Option<Bounds<Pixels>>,
    pub placement: PopupMenuPlacement,
    pub open_submenu: Option<usize>,
    pub active_path: Option<MenuPath>,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub state: PopupMenuState,
}

pub struct PopupMenuBuilder {
    pub(crate) model: PopupMenuModel,
}

impl PopupMenuBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: PopupMenuModel {
                label: id.clone(),
                id,
                items: Vec::new(),
                enabled: true,
                placement: PopupMenuPlacement::Smart,
                template: default_popup_menu_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn item(mut self, item: MenuItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn placement(mut self, placement: PopupMenuPlacement) -> Self {
        self.model.placement = placement;
        self
    }

    pub fn template(mut self, template: Arc<dyn PopupMenuTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<PopupMenu> {
        cx.new(|cx| PopupMenu::from_builder(self, cx))
    }
}
