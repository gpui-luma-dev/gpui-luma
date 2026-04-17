use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};
use lucide_icons::Icon as LucideIcon;

use super::{DropdownMenu, DropdownMenuState, DropdownMenuTemplate, default_dropdown_menu_template};

#[derive(Clone, Debug)]
pub enum DropdownMenuItemIcon {
    Lucide(LucideIcon),
    SvgPath(SharedString),
}

impl DropdownMenuItemIcon {
    pub fn lucide(&self) -> Option<LucideIcon> {
        match self {
            Self::Lucide(icon) => Some(*icon),
            Self::SvgPath(_) => None,
        }
    }

    pub fn svg_path(&self) -> Option<&SharedString> {
        match self {
            Self::Lucide(_) => None,
            Self::SvgPath(path) => Some(path),
        }
    }
}

impl From<LucideIcon> for DropdownMenuItemIcon {
    fn from(icon: LucideIcon) -> Self {
        Self::Lucide(icon)
    }
}

impl From<&str> for DropdownMenuItemIcon {
    fn from(icon: &str) -> Self {
        Self::SvgPath(icon.to_string().into())
    }
}

impl From<String> for DropdownMenuItemIcon {
    fn from(icon: String) -> Self {
        Self::from(icon.as_str())
    }
}

impl From<SharedString> for DropdownMenuItemIcon {
    fn from(icon: SharedString) -> Self {
        Self::SvgPath(icon)
    }
}

#[derive(Clone, Debug)]
pub struct DropdownMenuItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) icon: Option<DropdownMenuItemIcon>,
    pub(crate) submenu_items: Vec<DropdownMenuItem>,
    pub(crate) enabled: bool,
}

impl DropdownMenuItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            label: id.clone(),
            id,
            icon: None,
            submenu_items: Vec::new(),
            enabled: true,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<DropdownMenuItemIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn submenu(mut self, items: impl IntoIterator<Item = DropdownMenuItem>) -> Self {
        self.submenu_items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }

    pub fn icon_ref(&self) -> Option<&DropdownMenuItemIcon> {
        self.icon.as_ref()
    }

    pub fn submenu_items(&self) -> &[DropdownMenuItem] {
        &self.submenu_items
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone)]
pub struct DropdownMenuModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) items: Vec<DropdownMenuItem>,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn DropdownMenuTemplate>,
}

pub struct DropdownMenuRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub items: &'a [DropdownMenuItem],
    pub open: bool,
    pub open_submenu: Option<usize>,
    pub enabled: bool,
    pub state: DropdownMenuState,
}

pub struct DropdownMenuBuilder {
    pub(crate) model: DropdownMenuModel,
}

impl DropdownMenuBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: DropdownMenuModel {
                label: id.clone(),
                id,
                items: Vec::new(),
                enabled: true,
                template: default_dropdown_menu_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn item(mut self, item: DropdownMenuItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = DropdownMenuItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn DropdownMenuTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<DropdownMenu> {
        cx.new(|cx| DropdownMenu::from_builder(self, cx))
    }
}
