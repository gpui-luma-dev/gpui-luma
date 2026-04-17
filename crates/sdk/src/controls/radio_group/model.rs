use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{RadioGroup, RadioGroupTemplate, default_radio_group_template};
use crate::controls::radio_group::RadioGroupItemState;

#[derive(Clone, Debug)]
pub struct RadioGroupItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) enabled: bool,
}

impl RadioGroupItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            label: id.clone(),
            id,
            enabled: true,
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
pub struct RadioGroupModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<RadioGroupItem>,
    pub(crate) selected_id: Option<SharedString>,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn RadioGroupTemplate>,
}

pub struct RadioGroupRenderItem<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub selected: bool,
    pub enabled: bool,
    pub state: RadioGroupItemState,
}

pub struct RadioGroupRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: Vec<RadioGroupRenderItem<'a>>,
    pub selected_id: Option<&'a SharedString>,
    pub enabled: bool,
}

pub struct RadioGroupBuilder {
    pub(crate) model: RadioGroupModel,
}

impl RadioGroupBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: RadioGroupModel {
                id: id.into(),
                items: Vec::new(),
                selected_id: None,
                enabled: true,
                template: default_radio_group_template(),
            },
        }
    }

    pub fn item(mut self, item: RadioGroupItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = RadioGroupItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.selected_id = Some(selected_id.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn RadioGroupTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<RadioGroup> {
        cx.new(|cx| RadioGroup::from_builder(self, cx))
    }
}
