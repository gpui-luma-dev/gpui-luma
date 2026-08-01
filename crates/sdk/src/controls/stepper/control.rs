use gpui::{Context, IntoElement, Render, SharedString, Window, div, prelude::*};

use super::model::{StepperBuilder, StepperLabelPlacement, StepperModel, StepperRenderModel, derive_step_states};
use super::{StepState, StepperTemplate};
use crate::controls::progress::ProgressDirection;
use crate::theme::{ControlSize, observe_theme_revision};

pub struct StepperControl {
    model: StepperModel,
    step_states: Vec<StepState>,
}

impl StepperControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>, step_count: usize) -> StepperBuilder {
        StepperBuilder::new(id, step_count)
    }

    pub(crate) fn from_builder(builder: StepperBuilder, cx: &mut Context<Self>) -> Self {
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        let step_states = derive_step_states(&builder.model);
        Self { model: builder.model, step_states }
    }

    pub fn step_count(&self) -> usize {
        self.model.step_count
    }

    pub fn current_step(&self) -> usize {
        self.model.current_step
    }

    pub fn is_complete(&self) -> bool {
        self.model.workflow_complete
    }

    pub fn label_placement(&self) -> StepperLabelPlacement {
        self.model.label_placement
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

    pub fn set_current_step(&mut self, current: usize, cx: &mut Context<Self>) {
        let current = current.min(self.model.step_count.saturating_sub(1));
        let changed = self.model.current_step != current || self.model.workflow_complete;

        self.model.workflow_complete = false;
        self.model.current_step = current;
        if !changed {
            return;
        }

        self.refresh_step_states();
        cx.notify();
    }

    pub fn complete(&mut self, cx: &mut Context<Self>) {
        if self.model.workflow_complete {
            return;
        }

        self.model.workflow_complete = true;
        self.model.current_step = self.model.step_count.saturating_sub(1);
        self.refresh_step_states();
        cx.notify();
    }

    pub fn set_step_state(&mut self, step_index: usize, state: StepState, cx: &mut Context<Self>) {
        if step_index >= self.model.step_count {
            return;
        }

        self.model.step_state_overrides[step_index] = Some(state);
        self.refresh_step_states();
        cx.notify();
    }

    pub fn clear_step_state(&mut self, step_index: usize, cx: &mut Context<Self>) {
        if step_index >= self.model.step_count {
            return;
        }

        self.model.step_state_overrides[step_index] = None;
        self.refresh_step_states();
        cx.notify();
    }

    pub fn set_labels(&mut self, labels: Vec<impl Into<SharedString>>, cx: &mut Context<Self>) {
        self.model.labels = labels.into_iter().map(Into::into).collect();
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        cx.notify();
    }

    pub fn set_size(&mut self, size: ControlSize, cx: &mut Context<Self>) {
        if self.model.size == size {
            return;
        }

        self.model.size = size;
        cx.notify();
    }

    pub fn set_direction(&mut self, direction: ProgressDirection, cx: &mut Context<Self>) {
        if self.model.direction == direction {
            return;
        }

        self.model.direction = direction;
        cx.notify();
    }

    pub fn set_label_placement(&mut self, placement: StepperLabelPlacement, cx: &mut Context<Self>) {
        if self.model.label_placement == placement {
            return;
        }

        self.model.label_placement = placement;
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn StepperTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    fn refresh_step_states(&mut self) {
        self.step_states = derive_step_states(&self.model);
    }

    fn render_model(&self) -> StepperRenderModel<'_> {
        StepperRenderModel {
            id: &self.model.id,
            step_count: self.model.step_count,
            current_step: self.model.current_step,
            step_states: &self.step_states,
            labels: &self.model.labels,
            label_placement: self.model.label_placement,
            direction: self.model.direction,
            size: self.model.size,
            enabled: self.model.enabled,
        }
    }
}

impl Render for StepperControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model();

        div().child(self.model.template.render(&model, window, cx)).into_any_element()
    }
}
