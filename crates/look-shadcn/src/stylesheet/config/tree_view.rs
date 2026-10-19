use gpui_luma::theme::{InteractionLayer};
use serde::Deserialize;

use super::matches_optional_layer;

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TreeViewStylesheet {
    #[serde(default)]
    pub row: TreeViewRowStylesheet,
}

impl TreeViewStylesheet {
    /// Native bundled row rules in matching order.
    pub fn native_default() -> Self {
        Self {
            row: TreeViewRowStylesheet {
                color_rules: [
                    (true, None, "muted-foreground", None),
                    (false, Some("default"), "first(sidebar-foreground,foreground)", None),
                    (false, Some("disabled"), "first(sidebar-foreground,foreground)", None),
                    (
                        false,
                        Some("hover"),
                        "first(sidebar-foreground,foreground)",
                        Some("first_layer(sidebar-accent,accent)"),
                    ),
                    (false, Some("pressed"), "first(sidebar-foreground,foreground)", Some("first_layer(accent)")),
                ]
                .into_iter()
                .map(|(disabled, layer, color, background)| TreeViewRowColorRule {
                    disabled: Some(disabled),
                    layer: layer.map(Into::into),
                    foreground: color.into(),
                    icon_color: color.into(),
                    chevron_color: color.into(),
                    background: background.map(Into::into),
                })
                .collect(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
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

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TreeViewRowColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub foreground: String,
    pub icon_color: String,
    pub chevron_color: String,
    pub background: Option<String>,
}
