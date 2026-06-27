use std::sync::Arc;

use gpui::{AppContext, Bounds, Entity, Pixels, SharedString};
use lucide_icons::Icon as LucideIcon;

use super::{ControlFocusState, PopupMenu, PopupMenuState, PopupMenuTemplate, MenuPath, default_popup_menu_template};
use crate::controls::menu_item::MenuItem;
use crate::theme::ControlSize;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopupMenuPlacement {
    #[default]
    Smart,
    BelowStart,
    AboveStart,
    CenteredOnTrigger,
}

/// Trigger chrome aligned with command button variants.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopupMenuTriggerStyle {
    #[default]
    Outline,
    Ghost,
}

#[derive(Clone)]
pub struct PopupMenuModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) items: Vec<MenuItem>,
    pub(crate) enabled: bool,
    pub(crate) placement: PopupMenuPlacement,
    pub(crate) trigger_style: PopupMenuTriggerStyle,
    pub(crate) trigger_size: ControlSize,
    pub(crate) trigger_icon: Option<LucideIcon>,
    pub(crate) without_elevation: bool,
    pub(crate) template: Arc<dyn PopupMenuTemplate>,
}

pub struct PopupMenuRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub items: &'a [MenuItem],
    pub open: bool,
    pub trigger_bounds: Option<Bounds<Pixels>>,
    pub placement: PopupMenuPlacement,
    pub trigger_style: PopupMenuTriggerStyle,
    pub trigger_size: ControlSize,
    pub trigger_icon: Option<LucideIcon>,
    pub without_elevation: bool,
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
                trigger_style: PopupMenuTriggerStyle::default(),
                trigger_size: ControlSize::Md,
                trigger_icon: None,
                without_elevation: false,
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

    pub fn trigger_style(mut self, style: PopupMenuTriggerStyle) -> Self {
        self.model.trigger_style = style;
        self
    }

    pub fn ghost(self) -> Self {
        self.trigger_style(PopupMenuTriggerStyle::Ghost)
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.trigger_size = size;
        self
    }

    pub fn trigger_icon(mut self, icon: LucideIcon) -> Self {
        self.model.trigger_icon = Some(icon);
        self
    }

    pub fn without_elevation(mut self) -> Self {
        self.model.without_elevation = true;
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
