use serde::Deserialize;

use super::{EnabledColorRule, find_enabled_color_rule};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ControlGroupStylesheet {
    #[serde(default)]
    pub list: ControlGroupListStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ControlGroupListStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ControlGroupListColorRule>,
}

impl ControlGroupListStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&ControlGroupListColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ControlGroupListColorRule {
    pub enabled: Option<bool>,
    pub background: String,
    pub border: String,
}

impl EnabledColorRule for ControlGroupListColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}
