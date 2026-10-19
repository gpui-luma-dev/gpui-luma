use gpui_luma::theme::{InteractionLayer};
use serde::Deserialize;

use super::{EnabledColorRule, find_enabled_color_rule, matches_optional_bool, matches_optional_layer};

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TabsStylesheet {
    #[serde(default)]
    pub list: TabsListStylesheet,
    #[serde(default)]
    pub item: TabsItemStylesheet,
}

impl TabsStylesheet {
    /// Native bundled tab rules, preserving disabled and active matching order.
    pub fn native_default() -> Self {
        Self {
            list: TabsListStylesheet {
                color_rules: [(true, "transparent"), (false, "muted")]
                    .into_iter()
                    .map(|(enabled, color)| TabsListColorRule {
                        enabled: Some(enabled),
                        disabled_background: color.into(),
                    })
                    .collect(),
            },
            item: TabsItemStylesheet {
                color_rules: [
                    (None, "disabled", None, "muted-foreground", None),
                    (Some(false), "default", None, "foreground", None),
                    (Some(false), "hover", None, "foreground", None),
                    (Some(false), "pressed", None, "foreground", None),
                    (Some(true), "default", Some(false), "foreground", Some("@primary_default")),
                    (Some(true), "hover", Some(false), "foreground", Some("@primary_layer")),
                    (Some(true), "pressed", Some(false), "foreground", Some("@primary_layer")),
                    (Some(true), "default", Some(true), "foreground", Some("@primary_default")),
                    (Some(true), "hover", Some(true), "foreground", Some("@primary_layer")),
                    (Some(true), "pressed", Some(true), "foreground", Some("@primary_layer")),
                ]
                .into_iter()
                .map(|(active, layer, focused, color, indicator)| TabsItemColorRule {
                    active,
                    layer: Some(layer.into()),
                    focused,
                    label_color: color.into(),
                    indicator: indicator.map(Into::into),
                })
                .collect(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TabsListStylesheet {
    #[serde(default)]
    pub color_rules: Vec<TabsListColorRule>,
}

impl TabsListStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&TabsListColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TabsListColorRule {
    pub enabled: Option<bool>,
    pub disabled_background: String,
}

impl EnabledColorRule for TabsListColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
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

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TabsItemColorRule {
    pub active: Option<bool>,
    pub layer: Option<String>,
    pub focused: Option<bool>,
    pub label_color: String,
    pub indicator: Option<String>,
}
