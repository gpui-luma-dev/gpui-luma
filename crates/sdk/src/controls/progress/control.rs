use std::time::Duration;

use gpui::{Context, IntoElement, Render, SharedString, Window, div, prelude::*};

use super::direction::ProgressDirection;
use super::{ProgressBuilder, ProgressRenderModel};
use crate::animation::{ContinuousPhase, DEFAULT_CONTINUOUS_PERIOD, DEFAULT_TRANSITION_DURATION, VisualTransition};
use crate::controls::progress::model::ProgressModel;
use crate::controls::value::{ControlRange, value_from_input};
use crate::theme::{ControlSize, observe_theme_revision};

pub struct ProgressControl {
    model: ProgressModel,
    animated: bool,
    transition: VisualTransition,
    phase: ContinuousPhase,
}

impl ProgressControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ProgressBuilder {
        ProgressBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: ProgressBuilder, cx: &mut Context<Self>) -> Self {
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        let animated = builder.animated;
        let duration = if animated {
            DEFAULT_TRANSITION_DURATION
        } else {
            Duration::ZERO
        };
        let percentage = builder.model.range.percentage(builder.model.value);
        let transition = VisualTransition::new(percentage, duration);
        let mut phase = ContinuousPhase::new(DEFAULT_CONTINUOUS_PERIOD);
        if builder.model.indeterminate {
            phase.start();
        }
        Self { model: builder.model, animated, transition, phase }
    }

    pub fn value(&self) -> f32 {
        self.model.value
    }

    pub fn range(&self) -> ControlRange {
        self.model.range
    }

    pub fn is_enabled(&self) -> bool {
        self.model.enabled
    }

    pub fn size(&self) -> ControlSize {
        self.model.size
    }

    pub fn direction(&self) -> ProgressDirection {
        self.model.direction
    }

    pub fn show_thumb(&self) -> bool {
        self.model.show_thumb
    }

    pub fn animated(&self) -> bool {
        self.animated
    }

    pub fn is_indeterminate(&self) -> bool {
        self.model.indeterminate
    }

    pub fn set_value(&mut self, value: impl Into<f64>, cx: &mut Context<Self>) {
        self.model.value = self.model.range.clamp(value_from_input(value));
        if !self.model.indeterminate {
            self.transition.set_target(self.model.range.percentage(self.model.value));
        }
        cx.notify();
    }

    pub fn set_range(&mut self, range: impl Into<ControlRange>, cx: &mut Context<Self>) {
        self.model.range = range.into();
        self.model.value = self.model.range.clamp(self.model.value);
        if !self.model.indeterminate {
            self.transition.set_target(self.model.range.percentage(self.model.value));
        }
        cx.notify();
    }

    pub fn set_size(&mut self, size: ControlSize, cx: &mut Context<Self>) {
        if self.model.size == size {
            return;
        }

        self.model.size = size;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        cx.notify();
    }

    pub fn set_direction(&mut self, direction: ProgressDirection, cx: &mut Context<Self>) {
        if self.model.direction == direction {
            return;
        }

        self.model.direction = direction;
        cx.notify();
    }

    pub fn set_show_thumb(&mut self, show_thumb: bool, cx: &mut Context<Self>) {
        if self.model.show_thumb == show_thumb {
            return;
        }

        self.model.show_thumb = show_thumb;
        cx.notify();
    }

    pub fn set_animated(&mut self, animated: bool, cx: &mut Context<Self>) {
        if self.animated == animated {
            return;
        }

        self.animated = animated;
        let duration = if animated {
            DEFAULT_TRANSITION_DURATION
        } else {
            Duration::ZERO
        };
        let progress = self.transition.progress();
        self.transition = VisualTransition::new(progress, duration);
        if !self.model.indeterminate {
            self.transition.set_target(self.model.range.percentage(self.model.value));
        }
        cx.notify();
    }

    pub fn set_indeterminate(&mut self, indeterminate: bool, cx: &mut Context<Self>) {
        if self.model.indeterminate == indeterminate {
            return;
        }

        self.model.indeterminate = indeterminate;
        if indeterminate {
            self.phase.start();
        } else {
            self.phase.stop();
            self.transition.snap_to(self.model.range.percentage(self.model.value));
        }
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::ProgressTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    fn render_model(&self) -> ProgressRenderModel<'_> {
        let percentage = if self.model.indeterminate {
            0.0
        } else {
            self.transition.progress()
        };
        ProgressRenderModel {
            id: &self.model.id,
            range: self.model.range,
            value: self.model.value,
            percentage,
            size: self.model.size,
            enabled: self.model.enabled,
            direction: self.model.direction,
            show_thumb: self.model.show_thumb,
            indeterminate: self.model.indeterminate,
            phase: self.phase.phase(),
        }
    }
}

impl Render for ProgressControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let was_animating = self.transition.is_animating();
        let _ = self.transition.sync();
        let model = self.render_model();

        if self.model.indeterminate {
            self.phase.schedule_frame(window, cx);
        } else {
            self.transition.schedule_frame(window, cx);
            if was_animating || self.transition.is_animating() {
                cx.notify();
            }
        }

        div().child(self.model.template.render(&model, window, cx)).into_any_element()
    }
}
