use gpui_luma::theme::{InteractionLayer};
use serde::Deserialize;

use super::{LayeredElevationRule, matches_optional_bool};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CheckboxStylesheet {
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<CheckboxColorRule>,
}

impl CheckboxStylesheet {
    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn find_color_rule(&self, checked: bool, layer: InteractionLayer) -> Option<&CheckboxColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.layer.as_deref() == Some("disabled") {
                return layer == InteractionLayer::Disabled;
            }
            if layer == InteractionLayer::Disabled {
                return false;
            }
            rule.layer.is_none() && matches_optional_bool(rule.checked, checked)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CheckboxColorRule {
    pub layer: Option<String>,
    pub checked: Option<bool>,
    pub indicator_background: String,
    pub checkmark_color: String,
    pub label_color: String,
}
