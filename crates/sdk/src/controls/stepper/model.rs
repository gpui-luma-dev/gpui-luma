use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::control::StepperControl;
use super::template::{default_stepper_template, template_with_modifier};
use super::StepperTemplate;
use crate::controls::progress::ProgressDirection;
use crate::theme::ControlSize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StepState {
    Complete,
    #[default]
    InProgress,
    Incomplete,
}

/// Label position relative to the step badge. Side placements apply to vertical steppers; horizontal steppers keep labels below the badge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StepperLabelPlacement {
    #[default]
    Below,
    /// Left of the badge in vertical layouts.
    Start,
    /// Right of the badge in vertical layouts.
    End,
}

#[derive(Clone, Debug)]
pub struct StepperItem {
    pub index: usize,
    pub state: StepState,
    pub label: Option<SharedString>,
}

#[derive(Clone)]
pub struct StepperModel {
    pub(crate) id: SharedString,
    pub(crate) step_count: usize,
    pub(crate) current_step: usize,
    pub(crate) step_state_overrides: Vec<Option<StepState>>,
    pub(crate) labels: Vec<SharedString>,
    pub(crate) label_placement: StepperLabelPlacement,
    pub(crate) direction: ProgressDirection,
    pub(crate) size: ControlSize,
    pub(crate) enabled: bool,
    pub(crate) workflow_complete: bool,
    pub(crate) template: Arc<dyn StepperTemplate>,
}

pub struct StepperRenderModel<'a> {
    pub id: &'a SharedString,
    pub step_count: usize,
    pub current_step: usize,
    pub step_states: &'a [StepState],
    pub labels: &'a [SharedString],
    pub label_placement: StepperLabelPlacement,
    pub direction: ProgressDirection,
    pub size: ControlSize,
    pub enabled: bool,
}

pub struct StepperBuilder {
    pub(crate) model: StepperModel,
}

impl StepperBuilder {
    pub fn new(id: impl Into<SharedString>, step_count: usize) -> Self {
        let step_count = step_count.max(1);
        Self {
            model: StepperModel {
                id: id.into(),
                step_count,
                current_step: 0,
                step_state_overrides: vec![None; step_count],
                labels: Vec::new(),
                label_placement: StepperLabelPlacement::default(),
                direction: ProgressDirection::default(),
                size: ControlSize::Md,
                enabled: true,
                workflow_complete: false,
                template: default_stepper_template(),
            },
        }
    }

    pub fn current_step(mut self, current: usize) -> Self {
        self.model.current_step = current.min(self.model.step_count.saturating_sub(1));
        self
    }

    pub fn step_state(mut self, step_index: usize, state: StepState) -> Self {
        if step_index < self.model.step_count {
            self.model.step_state_overrides[step_index] = Some(state);
        }
        self
    }

    pub fn labels(mut self, labels: Vec<impl Into<SharedString>>) -> Self {
        self.model.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    pub fn label_placement(mut self, placement: StepperLabelPlacement) -> Self {
        self.model.label_placement = placement;
        self
    }

    pub fn complete(mut self) -> Self {
        self.model.workflow_complete = true;
        self.model.current_step = self.model.step_count.saturating_sub(1);
        self
    }

    pub fn direction(mut self, direction: ProgressDirection) -> Self {
        self.model.direction = direction;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn StepperTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &StepperRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<StepperControl> {
        cx.new(|cx| StepperControl::from_builder(self, cx))
    }
}

pub(crate) fn derive_step_states(model: &StepperModel) -> Vec<StepState> {
    if model.workflow_complete {
        return vec![StepState::Complete; model.step_count];
    }

    (0..model.step_count)
        .map(|index| {
            if let Some(state) = model.step_state_overrides[index] {
                return state;
            }

            if index < model.current_step {
                StepState::Complete
            } else if index == model.current_step {
                StepState::InProgress
            } else {
                StepState::Incomplete
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_step_clamps_to_last_index() {
        let builder = StepperBuilder::new("stepper", 4).current_step(99);
        assert_eq!(builder.model.current_step, 3);
    }

    #[test]
    fn derive_step_states_at_start() {
        let model = StepperBuilder::new("stepper", 4).current_step(0).model;
        let states = derive_step_states(&model);
        assert_eq!(
            states,
            vec![StepState::InProgress, StepState::Incomplete, StepState::Incomplete, StepState::Incomplete]
        );
    }

    #[test]
    fn derive_step_states_at_end() {
        let model = StepperBuilder::new("stepper", 4).current_step(3).model;
        let states = derive_step_states(&model);
        assert_eq!(states, vec![StepState::Complete, StepState::Complete, StepState::Complete, StepState::InProgress]);
    }

    #[test]
    fn derive_step_states_when_workflow_complete() {
        let model = StepperBuilder::new("stepper", 4).complete().model;
        let states = derive_step_states(&model);
        assert!(states.iter().all(|state| *state == StepState::Complete));
    }

    #[test]
    fn explicit_step_state_override() {
        let model = StepperBuilder::new("stepper", 3).current_step(0).step_state(2, StepState::Complete).model;
        let states = derive_step_states(&model);
        assert_eq!(states[2], StepState::Complete);
    }
}
