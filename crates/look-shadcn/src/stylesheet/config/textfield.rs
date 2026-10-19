use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TextfieldStylesheet {
    #[serde(default)]
    pub elevation_rules: Vec<TextfieldElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<TextfieldColorRule>,
}

impl TextfieldStylesheet {
    /// Native bundled field rules, preserving the surface-invalid exception.
    pub fn native_default() -> Self {
        Self {
            elevation_rules: [("primary", "shadow-xs"), ("outline", "none"), ("surface", "none"), ("input", "none")]
                .into_iter()
                .map(|(style, shadow)| TextfieldElevationRule { style: style.into(), shadow: shadow.into() })
                .collect(),
            color_rules: [
                ("input", false, None, None, "muted", "border"),
                ("input", true, Some(false), Some("light"), "transparent", "border"),
                ("input", true, Some(false), Some("dark"), "input/30", "border"),
                ("input", true, Some(true), Some("light"), "transparent", "destructive"),
                ("input", true, Some(true), Some("dark"), "input/30", "destructive"),
                ("primary", false, None, None, "muted", "border"),
                ("primary", true, Some(false), None, "background", "border"),
                ("primary", true, Some(true), None, "background", "destructive"),
                ("outline", false, None, None, "muted", "border"),
                ("surface", false, None, None, "muted", "transparent"),
                ("surface", true, Some(true), None, "muted", "destructive"),
                ("surface", true, None, None, "muted", "transparent"),
                ("outline", true, Some(false), Some("light"), "transparent", "border"),
                ("outline", true, Some(false), Some("dark"), "input/30", "border"),
                ("outline", true, Some(true), Some("light"), "transparent", "destructive"),
                ("outline", true, Some(true), Some("dark"), "input/30", "destructive"),
            ]
            .into_iter()
            .map(|(style, enabled, invalid, mode, background, border)| TextfieldColorRule {
                style: Some(style.into()),
                enabled: Some(enabled),
                invalid,
                mode: mode.map(Into::into),
                background: background.into(),
                foreground: if enabled { "foreground" } else { "muted-foreground" }.into(),
                border: border.into(),
                placeholder: "muted-foreground".into(),
                icon: "muted-foreground".into(),
                selection_background: if enabled { "primary" } else { "muted-foreground" }.into(),
                selection_foreground: if enabled {
                    "primary-foreground"
                } else {
                    "muted-foreground"
                }
                .into(),
                caret: if enabled { "foreground" } else { "muted-foreground" }.into(),
            })
            .collect(),
        }
    }

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

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TextfieldElevationRule {
    pub style: String,
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
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
