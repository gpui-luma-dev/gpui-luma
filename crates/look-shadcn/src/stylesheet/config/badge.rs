use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct BadgeStylesheet {
    #[serde(default)]
    pub color_rules: Vec<BadgeColorRule>,
}

impl BadgeStylesheet {
    pub fn find_color_rule(&self, style: &str, mode: &str) -> Option<&BadgeColorRule> {
        self.color_rules.iter().find(|rule| rule.matches(style, mode))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct BadgeColorRule {
    pub style: Option<String>,
    pub mode: Option<String>,
    pub background: String,
    pub foreground: String,
    pub border: Option<String>,
}

impl BadgeColorRule {
    pub fn matches(&self, style: &str, mode: &str) -> bool {
        self.style.as_deref().is_none_or(|value| value == "any" || value == style)
            && self.mode.as_deref().is_none_or(|value| value == "any" || value == mode)
    }
}
