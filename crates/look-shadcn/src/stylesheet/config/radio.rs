use gpui_luma::theme::{InteractionLayer};
use serde::Deserialize;

use crate::controls::ShadcnButtonStyle;

use super::{LayeredElevationRule, matches_optional_bool, matches_optional_str};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct RadioIndicatorDefaults {
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub outline: Option<String>,
    pub ghost: Option<String>,
}

impl RadioIndicatorDefaults {
    pub fn for_style(&self, style: ShadcnButtonStyle) -> &str {
        match style {
            ShadcnButtonStyle::Primary => self.primary.as_deref().unwrap_or("filled"),
            ShadcnButtonStyle::Secondary => self.secondary.as_deref().unwrap_or("ring"),
            ShadcnButtonStyle::Outline => self.outline.as_deref().unwrap_or("ring"),
            ShadcnButtonStyle::Ghost => self.ghost.as_deref().unwrap_or("ring"),
            ShadcnButtonStyle::ContentOnly => self.primary.as_deref().unwrap_or("filled"),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct RadioStylesheet {
    #[serde(default)]
    pub indicator_defaults: RadioIndicatorDefaults,
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<RadioColorRule>,
}

impl RadioStylesheet {
    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn find_color_rule(
        &self,
        style: ShadcnButtonStyle,
        selected: bool,
        layer: InteractionLayer,
    ) -> Option<&RadioColorRule> {
        let indicator = self.indicator_defaults.for_style(style);
        self.color_rules.iter().find(|rule| {
            if rule.layer.as_deref() == Some("disabled") {
                return layer == InteractionLayer::Disabled;
            }
            if layer == InteractionLayer::Disabled {
                return false;
            }
            if rule.layer.is_some() {
                return false;
            }
            if !matches_optional_bool(rule.selected, selected) {
                return false;
            }
            matches_optional_str(rule.indicator.as_deref(), indicator)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RadioColorRule {
    pub layer: Option<String>,
    pub selected: Option<bool>,
    pub indicator: Option<String>,
    pub indicator_background: String,
    pub selection_ring: String,
    pub dot_color: String,
    pub label_color: String,
}
