mod parse;

use std::collections::BTreeMap;

pub use parse::parse_css_catalog;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenCatalog {
    pub light: BTreeMap<String, String>,
    pub dark: BTreeMap<String, String>,
}

impl TokenCatalog {
    pub fn get(&self, mode: crate::mode::ThemeMode, token: &str) -> Option<&str> {
        let map = match mode {
            crate::mode::ThemeMode::Light => &self.light,
            crate::mode::ThemeMode::Dark => &self.dark,
        };
        map.get(token).map(String::as_str)
    }
}
