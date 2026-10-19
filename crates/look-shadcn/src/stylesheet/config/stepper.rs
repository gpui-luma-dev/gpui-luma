use std::collections::HashMap;

use gpui_luma::theme::{ControlSize};
use serde::Deserialize;

use super::control_size_key;

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct StepperStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, StepperMetricsRule>,
}

impl StepperStylesheet {
    /// Bundled stepper sizes are supplied by the common stylesheet.
    pub fn native_default() -> Self {
        Self::default()
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&StepperMetricsRule> {
        self.metrics.get(control_size_key(size))
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
pub struct StepperMetricsRule {
    pub step_badge_size: f32,
    pub track_thickness: f32,
}
