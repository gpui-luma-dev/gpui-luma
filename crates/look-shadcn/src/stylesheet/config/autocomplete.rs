use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AutocompleteStylesheet {
    #[serde(default)]
    pub chrome: AutocompleteChromeStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AutocompleteChromeStylesheet {
    #[serde(default)]
    pub color_rules: Vec<AutocompleteChromeColorRule>,
}

impl AutocompleteChromeStylesheet {
    pub fn color_rule(&self) -> Option<&AutocompleteChromeColorRule> {
        self.color_rules.first()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AutocompleteChromeColorRule {
    pub status_color: String,
    pub muted_text_color: String,
    pub clear_icon_color: String,
    pub clear_icon_hover_color: String,
}
