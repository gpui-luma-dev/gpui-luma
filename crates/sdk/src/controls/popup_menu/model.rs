use std::sync::Arc;

use gpui::{AppContext, Bounds, Entity, Pixels, SharedString};
use lucide_icons::Icon as LucideIcon;

use super::{ControlFocusState, PopupMenu, PopupMenuState, PopupMenuTemplate, MenuPath, default_popup_menu_template};

#[derive(Clone, Debug)]
pub enum PopupMenuItemIcon {
    Lucide(LucideIcon),
    SvgPath(SharedString),
}

impl PopupMenuItemIcon {
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

impl From<LucideIcon> for PopupMenuItemIcon {
    fn from(icon: LucideIcon) -> Self {
        Self::Lucide(icon)
    }
}

impl From<&str> for PopupMenuItemIcon {
    fn from(icon: &str) -> Self {
        Self::SvgPath(icon.to_string().into())
    }
}

impl From<String> for PopupMenuItemIcon {
    fn from(icon: String) -> Self {
        Self::from(icon.as_str())
    }
}

impl From<SharedString> for PopupMenuItemIcon {
    fn from(icon: SharedString) -> Self {
        Self::SvgPath(icon)
    }
}

#[derive(Clone, Debug)]
pub struct PopupMenuItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) icon: Option<PopupMenuItemIcon>,
    pub(crate) submenu_items: Vec<PopupMenuItem>,
    pub(crate) enabled: bool,
}

impl PopupMenuItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self { label: id.clone(), id, icon: None, submenu_items: Vec::new(), enabled: true }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<PopupMenuItemIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn submenu(mut self, items: impl IntoIterator<Item = PopupMenuItem>) -> Self {
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

    pub fn icon_ref(&self) -> Option<&PopupMenuItemIcon> {
        self.icon.as_ref()
    }

    pub fn submenu_items(&self) -> &[PopupMenuItem] {
        &self.submenu_items
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

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
    pub(crate) items: Vec<PopupMenuItem>,
    pub(crate) enabled: bool,
    pub(crate) placement: PopupMenuPlacement,
    pub(crate) template: Arc<dyn PopupMenuTemplate>,
}

pub struct PopupMenuRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub items: &'a [PopupMenuItem],
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
