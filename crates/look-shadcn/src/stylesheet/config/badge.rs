use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct BadgeStylesheet {
    #[serde(default)]
    pub color_rules: Vec<BadgeColorRule>,
}

impl BadgeStylesheet {
    /// Native bundled rules in matching order.
    pub fn native_default() -> Self {
        Self {
            color_rules: [
                ("default", None, "primary", "primary-foreground", None),
                ("secondary", None, "secondary", "secondary-foreground", None),
                ("outline", Some("light"), "background", "foreground", Some("border")),
                ("outline", Some("dark"), "input/30", "foreground", Some("input")),
                ("ghost", None, "transparent", "foreground", None),
            ]
            .into_iter()
            .map(|(style, mode, background, foreground, border)| BadgeColorRule {
                style: Some(style.into()),
                mode: mode.map(Into::into),
                background: background.into(),
                foreground: foreground.into(),
                border: border.map(Into::into),
            })
            .collect(),
        }
    }

    pub fn find_color_rule(&self, style: &str, mode: &str) -> Option<&BadgeColorRule> {
        self.color_rules.iter().find(|rule| rule.matches(style, mode))
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
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
