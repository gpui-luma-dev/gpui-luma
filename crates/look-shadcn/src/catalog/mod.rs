mod parse;

use std::collections::BTreeMap;

use anyhow::{Context as _, Result, anyhow};
use gpui::Hsla;

pub use parse::{CssTokenCatalog, parse_css_catalog};

use crate::color::parse_css_color;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CssTokenMap {
    pub tokens: BTreeMap<String, String>,
}

impl CssTokenMap {
    pub fn from_map(tokens: BTreeMap<String, String>) -> Self {
        Self { tokens }
    }

    pub fn get(&self, token: &str) -> Option<&str> {
        self.tokens.get(token).map(String::as_str)
    }

    pub fn color(&self, token: &str) -> Result<Hsla> {
        let raw = self.get(token).ok_or_else(|| anyhow!("missing css token `--{token}`"))?;
        parse_css_color(raw).with_context(|| format!("parse css color for `--{token}`"))
    }

    pub fn color_first(&self, tokens: &[&str]) -> Result<Hsla> {
        for token in tokens {
            if let Some(raw) = self.get(token) {
                return parse_css_color(raw).with_context(|| format!("parse css color for `--{token}`"));
            }
        }
        Err(anyhow!("missing css token (tried: {})", tokens.join(", ")))
    }

    pub fn optional_color(&self, tokens: &[&str]) -> Result<Option<Hsla>> {
        for token in tokens {
            if let Some(raw) = self.get(token) {
                return parse_css_color(raw).with_context(|| format!("parse css color for `--{token}`")).map(Some);
            }
        }
        Ok(None)
    }
}

impl CssTokenCatalog {
    pub fn light_map(&self) -> CssTokenMap {
        CssTokenMap::from_map(self.light.clone())
    }

    pub fn dark_map(&self) -> CssTokenMap {
        CssTokenMap::from_map(self.dark.clone())
    }
}
