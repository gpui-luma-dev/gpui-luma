use serde::Deserialize;

use super::{EnabledColorRule, find_enabled_color_rule};

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ControlGroupStylesheet {
    #[serde(default)]
    pub list: ControlGroupListStylesheet,
}

impl ControlGroupStylesheet {
    /// Native bundled list rules.
    pub fn native_default() -> Self {
        Self {
            list: ControlGroupListStylesheet {
                color_rules: [(true, "muted"), (false, "muted-foreground")]
                    .into_iter()
                    .map(|(enabled, background)| ControlGroupListColorRule {
                        enabled: Some(enabled),
                        background: background.into(),
                        border: "border".into(),
                    })
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ControlGroupListStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ControlGroupListColorRule>,
}

impl ControlGroupListStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&ControlGroupListColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
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
