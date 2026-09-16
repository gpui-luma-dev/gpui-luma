use luma::theme::{InteractionLayer};
use serde::Deserialize;

use super::{EnabledColorRule, find_enabled_color_rule, matches_optional_bool, matches_optional_layer};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TabsStylesheet {
    #[serde(default)]
    pub list: TabsListStylesheet,
    #[serde(default)]
    pub item: TabsItemStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TabsListStylesheet {
    #[serde(default)]
    pub color_rules: Vec<TabsListColorRule>,
}

impl TabsListStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&TabsListColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TabsListColorRule {
    pub enabled: Option<bool>,
    pub disabled_background: String,
}

impl EnabledColorRule for TabsListColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TabsItemStylesheet {
    #[serde(default)]
    pub color_rules: Vec<TabsItemColorRule>,
}

impl TabsItemStylesheet {
    pub fn find_color_rule(&self, active: bool, layer: InteractionLayer, focused: bool) -> Option<&TabsItemColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.layer.as_deref() == Some("disabled") {
                return layer == InteractionLayer::Disabled;
            }
            if layer == InteractionLayer::Disabled {
                return false;
            }
            matches_optional_bool(rule.active, active)
                && matches_optional_layer(rule.layer.as_deref(), layer)
                && matches_optional_bool(rule.focused, focused)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TabsItemColorRule {
    pub active: Option<bool>,
    pub layer: Option<String>,
    pub focused: Option<bool>,
    pub label_color: String,
    pub indicator: Option<String>,
}
