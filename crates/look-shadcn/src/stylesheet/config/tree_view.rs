use gpui_luma::theme::{InteractionLayer};
use serde::Deserialize;

use super::matches_optional_layer;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TreeViewStylesheet {
    #[serde(default)]
    pub row: TreeViewRowStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TreeViewRowStylesheet {
    #[serde(default)]
    pub color_rules: Vec<TreeViewRowColorRule>,
}

impl TreeViewRowStylesheet {
    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&TreeViewRowColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TreeViewRowColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub foreground: String,
    pub icon_color: String,
    pub chevron_color: String,
    pub background: Option<String>,
}
