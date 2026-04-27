use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{Checkbox, CheckboxTemplate, default_checkbox_template};
use crate::controls::checkbox::CheckboxState;

#[derive(Clone)]
pub struct CheckboxModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) checked: bool,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn CheckboxTemplate>,
}

pub struct CheckboxRenderModel {
    pub id: SharedString,
    pub label: SharedString,
    pub checked: bool,
    pub enabled: bool,
    pub state: CheckboxState,
}

pub struct CheckboxBuilder {
    pub(crate) model: CheckboxModel,
}

impl CheckboxBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: CheckboxModel {
                label: id.clone(),
                id,
                checked: false,
                enabled: true,
                template: default_checkbox_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.model.checked = checked;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn CheckboxTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<Checkbox> {
        cx.new(|cx| Checkbox::from_builder(self, cx))
    }
}
