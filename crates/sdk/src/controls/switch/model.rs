use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{Switch, SwitchTemplate, default_switch_template};
use crate::controls::switch::SwitchState;

#[derive(Clone)]
pub struct SwitchModel {
    pub(crate) id: SharedString,
    pub(crate) label: Option<SharedString>,
    pub(crate) on: bool,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn SwitchTemplate>,
}

pub struct SwitchRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: Option<&'a SharedString>,
    pub on: bool,
    pub enabled: bool,
    pub state: SwitchState,
}

pub struct SwitchBuilder {
    pub(crate) model: SwitchModel,
}

impl SwitchBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: SwitchModel {
                id: id.into(),
                label: None,
                on: false,
                enabled: true,
                template: default_switch_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = Some(label.into());
        self
    }

    pub fn on(mut self, on: bool) -> Self {
        self.model.on = on;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn SwitchTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<Switch> {
        cx.new(|cx| Switch::from_builder(self, cx))
    }
}
