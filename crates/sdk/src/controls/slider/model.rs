use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{SliderTemplate, default_slider_template};
use super::control::SliderControl;
use crate::controls::slider::SliderState;
use crate::controls::value::{ControlRange, normalized_step, value_from_input};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SliderOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SliderInputStrategy {
    #[default]
    Horizontal,
    Vertical,
    Angular {
        min_angle: f32,
        max_angle: f32,
    },
}

impl SliderInputStrategy {
    pub fn orientation(self) -> SliderOrientation {
        match self {
            Self::Horizontal | Self::Angular { .. } => SliderOrientation::Horizontal,
            Self::Vertical => SliderOrientation::Vertical,
        }
    }
}

impl From<SliderOrientation> for SliderInputStrategy {
    fn from(value: SliderOrientation) -> Self {
        match value {
            SliderOrientation::Horizontal => Self::Horizontal,
            SliderOrientation::Vertical => Self::Vertical,
        }
    }
}

#[derive(Clone)]
pub struct SliderModel {
    pub(crate) id: SharedString,
    pub(crate) strategy: SliderInputStrategy,
    pub(crate) range: ControlRange,
    pub(crate) step: f32,
    pub(crate) value: f32,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn SliderTemplate>,
}

pub struct SliderRenderModel<'a> {
    pub id: &'a SharedString,
    pub strategy: SliderInputStrategy,
    pub orientation: SliderOrientation,
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
                strategy: SliderInputStrategy::default(),
                range: ControlRange::default(),
                step: 1.0,
                value: 0.0,
                enabled: true,
                template: default_slider_template(),
            },
        }
    }

    pub fn strategy(mut self, strategy: SliderInputStrategy) -> Self {
        self.model.strategy = strategy;
        self
    }

    pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
        self.model.strategy = orientation.into();
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.model.strategy = SliderOrientation::Horizontal.into();
        self
    }

    pub fn vertical(mut self) -> Self {
        self.model.strategy = SliderOrientation::Vertical.into();
        self
    }

    pub fn angular(mut self, min_angle: f32, max_angle: f32) -> Self {
        self.model.strategy = SliderInputStrategy::Angular { min_angle, max_angle };
        self
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
