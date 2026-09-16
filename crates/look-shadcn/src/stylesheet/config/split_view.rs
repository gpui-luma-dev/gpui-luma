use serde::Deserialize;

use super::{EnabledColorRule, find_enabled_color_rule};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SplitViewStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SplitViewColorRule>,
}

impl SplitViewStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&SplitViewColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SplitViewColorRule {
    pub enabled: Option<bool>,
    pub separator: String,
    pub separator_hover: String,
}

impl EnabledColorRule for SplitViewColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}
