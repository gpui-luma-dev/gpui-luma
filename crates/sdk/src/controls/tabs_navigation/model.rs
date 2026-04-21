use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{TabsNavigation, TabsNavigationTemplate, default_tabs_navigation_template};
use crate::controls::tabs_navigation::{ControlFocusState, TabsNavigationItemState};

#[derive(Clone, Debug)]
pub struct TabsNavigationItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) enabled: bool,
}

impl TabsNavigationItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self { label: id.clone(), id, enabled: true }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
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

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone)]
pub struct TabsNavigationModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<TabsNavigationItem>,
    pub(crate) active_id: Option<SharedString>,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn TabsNavigationTemplate>,
}

pub struct TabsNavigationRenderItem<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub active: bool,
    pub enabled: bool,
    pub state: TabsNavigationItemState,
}

pub struct TabsNavigationRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: Vec<TabsNavigationRenderItem<'a>>,
    pub active_id: Option<&'a SharedString>,
    pub enabled: bool,
    pub focus: ControlFocusState,
}

pub struct TabsNavigationBuilder {
    pub(crate) model: TabsNavigationModel,
}

impl TabsNavigationBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: TabsNavigationModel {
                id: id.into(),
                items: Vec::new(),
                active_id: None,
                enabled: true,
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

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn TabsNavigationTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<TabsNavigation> {
        cx.new(|cx| TabsNavigation::from_builder(self, cx))
    }
}
