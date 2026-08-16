use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::indicator::{TabsNavigationIndicatorMotion, TabsNavigationIndicatorPaint};
use super::template::{tabs_navigation_control_group_template, template_with_modifier};
use super::{TabsNavigation, TabsNavigationTemplate, default_tabs_navigation_template};
use crate::controls::control_group::ControlGroupItemLike;
use crate::controls::icon::IconSource;
use crate::controls::icon::DisclosureIcons;
use crate::controls::tabs_navigation::{ControlFocusState, TabsNavigationItemState};
use crate::theme::ControlSize;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsNavigationWidthMode {
    #[default]
    Intrinsic,
    Uniform,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsNavigationTriggerKind {
    #[default]
    Select,
    Dropdown,
}

#[derive(Clone, Debug)]
pub enum TabsNavigationItemAccessory {
    Icon(IconSource),
    Disclosure { open: bool },
}

impl TabsNavigationItemAccessory {
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
pub struct TabsNavigationItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) enabled: bool,
    pub(crate) trigger_kind: TabsNavigationTriggerKind,
    pub(crate) leading_accessory: Option<TabsNavigationItemAccessory>,
    pub(crate) trailing_accessory: Option<TabsNavigationItemAccessory>,
}

impl TabsNavigationItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            label: id.clone(),
            id,
            enabled: true,
            trigger_kind: TabsNavigationTriggerKind::Select,
            leading_accessory: None,
            trailing_accessory: None,
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

    pub fn trigger_kind(mut self, trigger_kind: TabsNavigationTriggerKind) -> Self {
        self.trigger_kind = trigger_kind;
        self
    }

    pub fn dropdown_trigger(mut self) -> Self {
        self.trigger_kind = TabsNavigationTriggerKind::Dropdown;
        if self.trailing_accessory.is_none() {
            self.trailing_accessory = Some(TabsNavigationItemAccessory::disclosure(false));
        }
        self
    }

    pub fn leading_accessory(mut self, accessory: TabsNavigationItemAccessory) -> Self {
        self.leading_accessory = Some(accessory);
        self
    }

    pub fn trailing_accessory(mut self, accessory: TabsNavigationItemAccessory) -> Self {
        self.trailing_accessory = Some(accessory);
        self
    }

    pub fn leading_icon(self, icon: impl Into<IconSource>) -> Self {
        self.leading_accessory(TabsNavigationItemAccessory::icon(icon))
    }

    pub fn trailing_icon(self, icon: impl Into<IconSource>) -> Self {
        self.trailing_accessory(TabsNavigationItemAccessory::icon(icon))
    }

    pub fn trailing_disclosure(self, open: bool) -> Self {
        self.trailing_accessory(TabsNavigationItemAccessory::disclosure(open))
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

    pub fn trigger_kind_value(&self) -> TabsNavigationTriggerKind {
        self.trigger_kind
    }

    pub fn is_dropdown_trigger(&self) -> bool {
        self.trigger_kind == TabsNavigationTriggerKind::Dropdown
    }

    pub fn leading_accessory_ref(&self) -> Option<&TabsNavigationItemAccessory> {
        self.leading_accessory.as_ref()
    }

    pub fn trailing_accessory_ref(&self) -> Option<&TabsNavigationItemAccessory> {
        self.trailing_accessory.as_ref()
    }
}

impl ControlGroupItemLike for TabsNavigationItem {
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
pub struct TabsNavigationModel {
    pub(crate) id: SharedString,
    pub(crate) size: ControlSize,
    pub(crate) width_mode: TabsNavigationWidthMode,
    pub(crate) items: Vec<TabsNavigationItem>,
    pub(crate) active_id: Option<SharedString>,
    pub(crate) enabled: bool,
    pub(crate) animated: bool,
    pub(crate) disclosure_icons: DisclosureIcons,
    pub(crate) template: Arc<dyn TabsNavigationTemplate>,
}

pub struct TabsNavigationRenderItem<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub trigger_kind: TabsNavigationTriggerKind,
    pub leading_accessory: Option<&'a TabsNavigationItemAccessory>,
    pub trailing_accessory: Option<&'a TabsNavigationItemAccessory>,
    pub active: bool,
    pub enabled: bool,
    pub state: TabsNavigationItemState,
}

pub struct TabsNavigationRenderModel<'a> {
    pub id: &'a SharedString,
    pub size: ControlSize,
    pub width_mode: TabsNavigationWidthMode,
    pub items: Vec<TabsNavigationRenderItem<'a>>,
    pub active_id: Option<&'a SharedString>,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub indicator: Option<TabsNavigationIndicatorPaint>,
    pub indicator_motion: Option<&'a TabsNavigationIndicatorMotion>,
    pub disclosure_icons: &'a DisclosureIcons,
}

pub struct TabsNavigationBuilder {
    pub(crate) model: TabsNavigationModel,
}

impl TabsNavigationBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: TabsNavigationModel {
                id: id.into(),
                size: ControlSize::Md,
                width_mode: TabsNavigationWidthMode::Intrinsic,
                items: Vec::new(),
                active_id: None,
                enabled: true,
                animated: true,
                disclosure_icons: DisclosureIcons::new(lucide_icons::Icon::ChevronUp, lucide_icons::Icon::ChevronDown),
                template: default_tabs_navigation_template(),
            },
        }
    }

    pub fn item(mut self, item: TabsNavigationItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TabsNavigationItem>) -> Self {
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

    pub fn width_mode(mut self, width_mode: TabsNavigationWidthMode) -> Self {
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

    pub fn template(mut self, template: Arc<dyn TabsNavigationTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &TabsNavigationRenderModel<'_>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<TabsNavigation> {
        cx.new(|cx| TabsNavigation::from_builder(self, cx))
    }

    pub(crate) fn control_group_template(
        &self,
        indicator_motion: TabsNavigationIndicatorMotion,
    ) -> crate::controls::control_group::ControlGroupTemplate<TabsNavigationItem> {
        tabs_navigation_control_group_template(
            self.model.size,
            self.model.width_mode,
            Arc::clone(&self.model.template),
            indicator_motion,
            self.model.disclosure_icons.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_tabs_navigation_template();
        let builder = TabsNavigationBuilder::new("tabs-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn item_accessories_are_configurable() {
        let item = TabsNavigationItem::new("controls")
            .leading_icon(lucide_icons::Icon::Settings)
            .trailing_disclosure(true);

        assert!(item.leading_accessory_ref().is_some());
        assert_eq!(item.trailing_accessory_ref().and_then(TabsNavigationItemAccessory::is_disclosure_open), Some(true));
    }

    #[test]
    fn dropdown_trigger_sets_trigger_kind_and_default_disclosure() {
        let item = TabsNavigationItem::new("controls").label("Controls").dropdown_trigger();

        assert!(item.is_dropdown_trigger());
        assert_eq!(
            item.trailing_accessory_ref().and_then(TabsNavigationItemAccessory::is_disclosure_open),
            Some(false)
        );
    }
}
