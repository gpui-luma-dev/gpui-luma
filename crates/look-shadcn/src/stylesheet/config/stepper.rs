use std::collections::HashMap;

use gpui_luma::theme::{ControlSize};
use serde::Deserialize;

use super::control_size_key;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct StepperStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, StepperMetricsRule>,
}

impl StepperStylesheet {
    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&StepperMetricsRule> {
        self.metrics.get(control_size_key(size))
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct StepperMetricsRule {
    pub step_badge_size: f32,
    pub track_thickness: f32,
}
