use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct AutocompleteStylesheet {
    #[serde(default)]
    pub chrome: AutocompleteChromeStylesheet,
}

impl AutocompleteStylesheet {
    /// Native bundled chrome colors.
    pub fn native_default() -> Self {
        Self {
            chrome: AutocompleteChromeStylesheet {
                color_rules: vec![AutocompleteChromeColorRule {
                    status_color: "primary".into(),
                    muted_text_color: "muted-foreground".into(),
                    clear_icon_color: "muted-foreground".into(),
                    clear_icon_hover_color: "foreground".into(),
                }],
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct AutocompleteChromeStylesheet {
    #[serde(default)]
    pub color_rules: Vec<AutocompleteChromeColorRule>,
}

impl AutocompleteChromeStylesheet {
    pub fn color_rule(&self) -> Option<&AutocompleteChromeColorRule> {
        self.color_rules.first()
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AutocompleteChromeColorRule {
    pub status_color: String,
    pub muted_text_color: String,
    pub clear_icon_color: String,
    pub clear_icon_hover_color: String,
}
