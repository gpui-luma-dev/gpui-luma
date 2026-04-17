use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{Progress, ProgressTemplate, default_progress_template};
use crate::controls::value::{ControlRange, value_from_input};

#[derive(Clone)]
pub struct ProgressModel {
    pub(crate) id: SharedString,
    pub(crate) range: ControlRange,
    pub(crate) value: f32,
    pub(crate) template: Arc<dyn ProgressTemplate>,
}

pub struct ProgressRenderModel<'a> {
    pub id: &'a SharedString,
    pub range: ControlRange,
    pub value: f32,
    pub percentage: f32,
}

pub struct ProgressBuilder {
    pub(crate) model: ProgressModel,
}

impl ProgressBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: ProgressModel {
                id: id.into(),
                range: ControlRange::default(),
                value: 0.0,
                template: default_progress_template(),
            },
        }
    }

    pub fn range(mut self, range: impl Into<ControlRange>) -> Self {
        self.model.range = range.into();
        self.model.value = self.model.range.clamp(self.model.value);
        self
    }

    pub fn value(mut self, value: impl Into<f64>) -> Self {
        self.model.value = self.model.range.clamp(value_from_input(value));
        self
    }

    pub fn template(mut self, template: Arc<dyn ProgressTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<Progress> {
        cx.new(|cx| Progress::from_builder(self, cx))
    }
}
