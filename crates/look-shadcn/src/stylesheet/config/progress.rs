use std::collections::HashMap;

use luma::theme::{ControlSize};
use serde::Deserialize;

use super::{EnabledColorRule, control_size_key, find_enabled_color_rule};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ProgressStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ProgressMetricsRule>,
    #[serde(default)]
    pub color_rules: Vec<ProgressColorRule>,
}

impl ProgressStylesheet {
    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ProgressMetricsRule> {
        self.metrics.get(control_size_key(size))
    }

    pub fn find_color_rule(&self, enabled: bool) -> Option<&ProgressColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ProgressMetricsRule {
    pub size: f32,
    pub stroke_width: f32,
    #[serde(default = "default_progress_track_height")]
    pub track_height: f32,
    #[serde(default = "default_progress_thumb_size")]
    pub thumb_size: f32,
}

fn default_progress_track_height() -> f32 {
    6.0
}

fn default_progress_thumb_size() -> f32 {
    16.0
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProgressColorRule {
    pub enabled: Option<bool>,
    pub track_color: String,
    pub progress_color: String,
}

impl EnabledColorRule for ProgressColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}
