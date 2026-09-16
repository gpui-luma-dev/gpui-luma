use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TextfieldStylesheet {
    #[serde(default)]
    pub elevation_rules: Vec<TextfieldElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<TextfieldColorRule>,
}

impl TextfieldStylesheet {
    pub fn elevation_rule_for_style(&self, style: &str) -> Option<&TextfieldElevationRule> {
        self.elevation_rules.iter().find(|rule| rule.style == style)
    }

    pub fn find_color_rule(
        &self,
        style: &str,
        enabled: bool,
        invalid: bool,
        mode: &str,
    ) -> Option<&TextfieldColorRule> {
        self.color_rules.iter().find(|rule| {
            rule.style.as_deref().is_none_or(|value| value == style)
                && rule.enabled.is_none_or(|value| value == enabled)
                && rule.invalid.is_none_or(|value| value == invalid)
                && rule.mode.as_deref().is_none_or(|value| value == mode)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TextfieldElevationRule {
    pub style: String,
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TextfieldColorRule {
    pub style: Option<String>,
    pub enabled: Option<bool>,
    pub invalid: Option<bool>,
    pub mode: Option<String>,
    pub background: String,
    pub foreground: String,
    pub border: String,
    pub placeholder: String,
    pub icon: String,
    pub selection_background: String,
    pub selection_foreground: String,
    pub caret: String,
}
