use std::collections::HashMap;

use luma::theme::{ControlSize, InteractionLayer};
use serde::Deserialize;

use super::{LayeredElevationRule, matches_optional_layer};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SliderStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, SliderMetricsRule>,
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<SliderColorRule>,
}

impl SliderStylesheet {
    pub fn find_color_rule(&self, style: &str, layer: InteractionLayer) -> Option<&SliderColorRule> {
        self.color_rules.iter().find(|rule| {
            rule.style.as_deref().is_none_or(|value| value == style)
                && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }

    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&SliderMetricsRule> {
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SliderMetricsRule {
    pub width: f32,
    pub height: f32,
    pub track_height: f32,
    pub thumb_size: f32,
    pub radius: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SliderColorRule {
    pub style: Option<String>,
    pub layer: Option<String>,
    pub track_background: String,
    pub fill_background: String,
    pub thumb_background: String,
    pub thumb_border: String,
}
