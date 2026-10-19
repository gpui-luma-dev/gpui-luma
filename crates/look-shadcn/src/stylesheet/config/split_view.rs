use serde::Deserialize;

use super::{EnabledColorRule, find_enabled_color_rule};

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct SplitViewStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SplitViewColorRule>,
}

impl SplitViewStylesheet {
    /// Native bundled separator colors.
    pub fn native_default() -> Self {
        Self {
            color_rules: [(true, "border", "border-hover"), (false, "muted-foreground", "muted-foreground")]
                .into_iter()
                .map(|(enabled, separator, hover)| SplitViewColorRule {
                    enabled: Some(enabled),
                    separator: separator.into(),
                    separator_hover: hover.into(),
                })
                .collect(),
        }
    }

    pub fn find_color_rule(&self, enabled: bool) -> Option<&SplitViewColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
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
