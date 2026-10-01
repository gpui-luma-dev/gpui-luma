use std::collections::HashMap;

use gpui_luma::theme::{ControlSize, InteractionLayer};
use serde::Deserialize;

use crate::controls::ShadcnButtonStyle;

use super::{LayeredElevationRule, control_size_key, matches_optional_bool};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SwitchStylesheet {
    /// Switch geometry keyed by `sm` / `md` / `lg`. All variants share these metrics.
    #[serde(default)]
    pub metrics: HashMap<String, SwitchMetricsRule>,
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<SwitchColorRule>,
}

impl SwitchStylesheet {
    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn find_color_rule(&self, on: bool, disabled: bool) -> Option<&SwitchColorRule> {
        self.color_rules
            .iter()
            .find(|rule| matches_optional_bool(rule.disabled, disabled) && matches_optional_bool(rule.on, on))
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&SwitchMetricsRule> {
        self.metrics.get(control_size_key(size))
    }

    pub fn metrics_for_style(&self, _style: ShadcnButtonStyle, size: ControlSize) -> Option<&SwitchMetricsRule> {
        self.metrics_for_size(size)
    }
}
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct SwitchMetricsRule {
    pub width: f32,
    pub height: f32,
    pub thumb_size: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SwitchColorRule {
    pub on: Option<bool>,
    pub disabled: Option<bool>,
    pub track_background: String,
    pub thumb_background: String,
    pub thumb_border: String,
    pub label_color: String,
}
