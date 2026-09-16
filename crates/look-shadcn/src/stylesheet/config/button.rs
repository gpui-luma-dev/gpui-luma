use std::collections::HashMap;

use luma::theme::{ControlSize, InteractionLayer};
use serde::Deserialize;

use super::LayeredElevationRule;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ButtonStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ButtonMetricsRule>,
    #[serde(default)]
    pub elevation_rules: Vec<ButtonElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<ButtonColorRule>,
}

impl ButtonStylesheet {
    pub fn find_color_rule(&self, style: &str, layer: &str, mode: &str, selected: bool) -> Option<&ButtonColorRule> {
        self.color_rules.iter().find(|rule| rule.matches(style, layer, mode, selected))
    }

    pub fn elevation_rule_for_style(&self, style: &str) -> Option<&ButtonElevationRule> {
        self.elevation_rules.iter().find(|rule| rule.style == style)
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ButtonMetricsRule> {
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ToggleStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ButtonMetricsRule>,
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
}

impl ToggleStylesheet {
    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ButtonMetricsRule> {
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ButtonMetricsRule {
    pub height: String,
    pub padding_horizontal: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub corner_radius: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ButtonElevationRule {
    pub style: String,
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ButtonColorRule {
    pub style: Option<String>,
    pub layer: Option<String>,
    pub mode: Option<String>,
    pub selected: Option<bool>,

    pub background: String,
    pub foreground: String,
    pub border: Option<String>,
}

impl ButtonColorRule {
    pub fn matches(&self, style: &str, layer: &str, mode: &str, selected: bool) -> bool {
        self.style.as_deref().is_none_or(|value| value == "any" || value == style)
            && self.layer.as_deref().is_none_or(|value| value == "any" || value == layer)
            && self.mode.as_deref().is_none_or(|value| value == "any" || value == mode)
            && self.selected.is_none_or(|value| value == selected)
    }
}
