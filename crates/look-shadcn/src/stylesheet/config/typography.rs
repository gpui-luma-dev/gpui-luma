use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TypographyStylesheet {
    #[serde(default)]
    pub semantic: HashMap<String, TypographyRule>,
    #[serde(default)]
    pub scale: HashMap<String, TypographyRule>,
}

impl TypographyStylesheet {
    pub fn semantic_rule(&self, key: &str) -> Option<&TypographyRule> {
        self.semantic.get(key)
    }

    pub fn scale_rule(&self, key: &str) -> Option<&TypographyRule> {
        self.scale.get(key)
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct TypographyRule {
    pub size: f32,
    pub line_height: f32,
    pub weight: f32,
}
