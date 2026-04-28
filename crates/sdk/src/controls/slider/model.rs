use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{SliderControl, SliderTemplate, default_slider_template};
use crate::controls::slider::SliderState;
use crate::controls::value::{ControlRange, normalized_step, value_from_input};

#[derive(Clone)]
pub struct SliderModel {
    pub(crate) id: SharedString,
    pub(crate) range: ControlRange,
    pub(crate) step: f32,
    pub(crate) value: f32,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn SliderTemplate>,
}

pub struct SliderRenderModel<'a> {
    pub id: &'a SharedString,
    pub range: ControlRange,
    pub step: f32,
    pub value: f32,
    pub percentage: f32,
    pub enabled: bool,
    pub state: SliderState,
}

pub struct SliderBuilder {
    pub(crate) model: SliderModel,
}

impl SliderBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: SliderModel {
                id: id.into(),
                range: ControlRange::default(),
                step: 1.0,
                value: 0.0,
                enabled: true,
                template: default_slider_template(),
            },
        }
    }

    pub fn range(mut self, range: impl Into<ControlRange>) -> Self {
        self.model.range = range.into();
        self.model.value = self.model.range.snap(self.model.value, self.model.step);
        self
    }

    pub fn step(mut self, step: impl Into<f64>) -> Self {
        self.model.step = normalized_step(value_from_input(step));
        self.model.value = self.model.range.snap(self.model.value, self.model.step);
        self
    }

    pub fn value(mut self, value: impl Into<f64>) -> Self {
        self.model.value = self.model.range.snap(value_from_input(value), self.model.step);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn SliderTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<SliderControl> {
        cx.new(|cx| SliderControl::from_builder(self, cx))
    }
}
