use luma::theme::{InteractionLayer};
use serde::Deserialize;

use super::{EnabledColorRule, find_enabled_color_rule, matches_optional_bool, matches_optional_layer};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListboxStylesheet {
    #[serde(default)]
    pub list: ListboxListStylesheet,
    #[serde(default)]
    pub row: ListboxRowStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListboxListStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ListboxListColorRule>,
}

impl ListboxListStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&ListboxListColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListboxListColorRule {
    pub enabled: Option<bool>,
    pub background: String,
    pub border: String,
    pub divider: String,
}

impl EnabledColorRule for ListboxListColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListboxRowStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ListboxRowColorRule>,
}

impl ListboxRowStylesheet {
    pub fn find_color_rule(
        &self,
        disabled: bool,
        focused: bool,
        layer: InteractionLayer,
    ) -> Option<&ListboxRowColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            matches_optional_bool(rule.focused, focused) && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListboxRowColorRule {
    pub disabled: Option<bool>,
    pub focused: Option<bool>,
    pub layer: Option<String>,
    pub label_color: String,
    pub background: String,
}
