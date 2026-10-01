use std::collections::HashMap;

use gpui_luma::theme::{ControlSize, InteractionLayer};
use serde::Deserialize;

use super::{control_size_key, matches_optional_bool, matches_optional_layer, matches_optional_str};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ScrollbarStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ScrollbarMetricsRule>,
    #[serde(default)]
    pub color_rules: Vec<ScrollbarColorRule>,
}

impl ScrollbarStylesheet {
    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ScrollbarMetricsRule> {
        self.metrics.get(control_size_key(size))
    }

    pub fn find_color_rule(&self, style: &str, disabled: bool, layer: InteractionLayer) -> Option<&ScrollbarColorRule> {
        self.color_rules.iter().find(|rule| {
            matches_optional_str(rule.style.as_deref(), style)
                && matches_optional_bool(rule.disabled, disabled)
                && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ScrollbarMetricsRule {
    pub thickness: f32,
    pub track_thickness: f32,
    pub thumb_thickness: f32,
    pub min_thumb_length: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScrollbarColorRule {
    pub style: Option<String>,
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub track_background: String,
    pub thumb_background: String,
}
