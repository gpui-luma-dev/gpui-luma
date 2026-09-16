use serde::Deserialize;

use super::floating_menu::FloatingMenuSurfaceElevationRule;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CardStylesheet {
    #[serde(default)]
    pub elevation_rules: Vec<FloatingMenuSurfaceElevationRule>,
    #[serde(default)]
    pub color_rule: Option<CardColorRule>,
}

impl CardStylesheet {
    pub fn elevation_rule(&self) -> Option<&FloatingMenuSurfaceElevationRule> {
        self.elevation_rules.first()
    }

    pub fn color_rule(&self) -> Option<&CardColorRule> {
        self.color_rule.as_ref()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CardColorRule {
    pub background: String,
    pub foreground: String,
    pub muted_foreground: String,
    pub border: String,
}
