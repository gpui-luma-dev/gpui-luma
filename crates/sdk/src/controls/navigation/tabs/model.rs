use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::indicator::{TabsIndicatorMotion, TabsIndicatorPaint};
use super::template::{tabs_control_group_template, template_with_modifier};
use super::{Tabs, TabsTemplate, default_tabs_template};
use crate::controls::control_group::ControlGroupItemLike;
use crate::infra::icon::IconSource;
use crate::infra::icon::DisclosureIcons;
use crate::controls::tabs::{ControlFocusState, TabsItemState};
use crate::theme::ControlSize;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsWidthMode {
    #[default]
    Intrinsic,
    Uniform,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsTriggerKind {
    #[default]
    Select,
    Dropdown,
}

#[derive(Clone, Debug)]
pub enum TabsItemAccessory {
    Icon(IconSource),
    Disclosure { open: bool },
}

impl TabsItemAccessory {
    pub fn icon(icon: impl Into<IconSource>) -> Self {
        Self::Icon(icon.into())
    }

    pub fn disclosure(open: bool) -> Self {
        Self::Disclosure { open }
    }

    pub fn is_disclosure_open(&self) -> Option<bool> {
        match self {
            Self::Disclosure { open } => Some(*open),
            Self::Icon(_) => None,
        }
    }

    pub(crate) fn set_disclosure_open(&mut self, open: bool) -> bool {
        let Self::Disclosure { open: current } = self else {
            return false;
        };
        if *current == open {
            return false;
        }
        *current = open;
        true
    }
}

#[derive(Clone, Debug)]
pub struct TabsItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) enabled: bool,
    pub(crate) trigger_kind: TabsTriggerKind,
    pub(crate) leading_accessory: Option<TabsItemAccessory>,
    pub(crate) trailing_accessory: Option<TabsItemAccessory>,
    pub(crate) disclosure_progress: f32,
}

impl TabsItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            label: id.clone(),
            id,
            enabled: true,
            trigger_kind: TabsTriggerKind::Select,
            leading_accessory: None,
            trailing_accessory: None,
            disclosure_progress: 0.0,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn trigger_kind(mut self, trigger_kind: TabsTriggerKind) -> Self {
        self.trigger_kind = trigger_kind;
        self
    }

    pub fn dropdown_trigger(mut self) -> Self {
        self.trigger_kind = TabsTriggerKind::Dropdown;
        if self.trailing_accessory.is_none() {
            self.trailing_accessory = Some(TabsItemAccessory::disclosure(false));
        }
        self.disclosure_progress = 0.0;
        self
    }

    pub fn leading_accessory(mut self, accessory: TabsItemAccessory) -> Self {
        self.leading_accessory = Some(accessory);
        self
    }

    pub fn trailing_accessory(mut self, accessory: TabsItemAccessory) -> Self {
        self.trailing_accessory = Some(accessory);
        self
    }

    pub fn leading_icon(self, icon: impl Into<IconSource>) -> Self {
        self.leading_accessory(TabsItemAccessory::icon(icon))
    }

    pub fn trailing_icon(self, icon: impl Into<IconSource>) -> Self {
        self.trailing_accessory(TabsItemAccessory::icon(icon))
    }

    pub fn trailing_disclosure(self, open: bool) -> Self {
        let mut item = self;
        item.trailing_accessory = Some(TabsItemAccessory::disclosure(open));
        item.disclosure_progress = if open { 1.0 } else { 0.0 };
        item
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn trigger_kind_value(&self) -> TabsTriggerKind {
        self.trigger_kind
    }

    pub fn is_dropdown_trigger(&self) -> bool {
        self.trigger_kind == TabsTriggerKind::Dropdown
    }

    pub fn leading_accessory_ref(&self) -> Option<&TabsItemAccessory> {
        self.leading_accessory.as_ref()
    }

    pub fn trailing_accessory_ref(&self) -> Option<&TabsItemAccessory> {
        self.trailing_accessory.as_ref()
    }

    pub fn disclosure_progress(&self) -> f32 {
        self.disclosure_progress
    }
}

impl ControlGroupItemLike for TabsItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn label(&self) -> &SharedString {
        &self.label
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone)]
pub struct TabsModel {
    pub(crate) id: SharedString,
    pub(crate) size: ControlSize,
    pub(crate) width_mode: TabsWidthMode,
    pub(crate) items: Vec<TabsItem>,
    pub(crate) active_id: Option<SharedString>,
    pub(crate) enabled: bool,
    pub(crate) animated: bool,
    pub(crate) disclosure_icons: DisclosureIcons,
    pub(crate) template: Arc<dyn TabsTemplate>,
}

pub struct TabsRenderItem<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub trigger_kind: TabsTriggerKind,
    pub leading_accessory: Option<&'a TabsItemAccessory>,
    pub trailing_accessory: Option<&'a TabsItemAccessory>,
    pub active: bool,
    pub enabled: bool,
    pub state: TabsItemState,
    pub disclosure_progress: f32,
}

pub struct TabsRenderModel<'a> {
    pub id: &'a SharedString,
    pub size: ControlSize,
    pub width_mode: TabsWidthMode,
    pub items: Vec<TabsRenderItem<'a>>,
    pub active_id: Option<&'a SharedString>,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub indicator: Option<TabsIndicatorPaint>,
    pub indicator_motion: Option<&'a TabsIndicatorMotion>,
    pub disclosure_icons: &'a DisclosureIcons,
}

pub struct TabsBuilder {
    pub(crate) model: TabsModel,
}

impl TabsBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: TabsModel {
                id: id.into(),
                size: ControlSize::Md,
                width_mode: TabsWidthMode::Intrinsic,
                items: Vec::new(),
                active_id: None,
                enabled: true,
                animated: true,
                disclosure_icons: DisclosureIcons::new(
                    lucide_svg_static::Icon::ChevronUp,
                    lucide_svg_static::Icon::ChevronDown,
                ),
                template: default_tabs_template(),
            },
        }
    }

    pub fn item(mut self, item: TabsItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TabsItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn active(mut self, active_id: impl Into<SharedString>) -> Self {
        self.model.active_id = Some(active_id.into());
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn width_mode(mut self, width_mode: TabsWidthMode) -> Self {
        self.model.width_mode = width_mode;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.model.animated = animated;
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.model.disclosure_icons = icons;
        self
    }

    pub fn template(mut self, template: Arc<dyn TabsTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &TabsRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<Tabs> {
        cx.new(|cx| Tabs::from_builder(self, cx))
    }

    pub(crate) fn control_group_template(
        &self,
        indicator_motion: TabsIndicatorMotion,
        disclosure_progress: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<SharedString, f32>>>,
    ) -> crate::controls::control_group::ControlGroupTemplate<TabsItem> {
        tabs_control_group_template(
            self.model.size,
            self.model.width_mode,
            Arc::clone(&self.model.template),
            indicator_motion,
            self.model.disclosure_icons.clone(),
            disclosure_progress,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_tabs_template();
        let builder = TabsBuilder::new("tabs-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn item_accessories_are_configurable() {
        let item = TabsItem::new("controls").leading_icon(lucide_svg_static::Icon::Settings).trailing_disclosure(true);

        assert!(item.leading_accessory_ref().is_some());
        assert_eq!(item.trailing_accessory_ref().and_then(TabsItemAccessory::is_disclosure_open), Some(true));
    }

    #[test]
    fn dropdown_trigger_sets_trigger_kind_and_default_disclosure() {
        let item = TabsItem::new("controls").label("Controls").dropdown_trigger();

        assert!(item.is_dropdown_trigger());
        assert_eq!(item.trailing_accessory_ref().and_then(TabsItemAccessory::is_disclosure_open), Some(false));
    }
}
