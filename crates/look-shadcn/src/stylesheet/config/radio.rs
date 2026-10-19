use gpui_luma::theme::{InteractionLayer};
use serde::Deserialize;

use crate::controls::ShadcnButtonStyle;

use super::{LayeredElevationRule, matches_optional_bool, matches_optional_str};

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
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

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct RadioStylesheet {
    #[serde(default)]
    pub indicator_defaults: RadioIndicatorDefaults,
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<RadioColorRule>,
}

impl RadioStylesheet {
    /// Native bundled radio rules in matching order.
    pub fn native_default() -> Self {
        Self {
            indicator_defaults: RadioIndicatorDefaults {
                primary: Some("filled".into()),
                secondary: Some("ring".into()),
                outline: Some("ring".into()),
                ghost: Some("ring".into()),
            },
            elevation_rules: vec![
                LayeredElevationRule { layer: None, shadow: "shadow-xs".into() },
                LayeredElevationRule { layer: Some("disabled".into()), shadow: "none".into() },
            ],
            color_rules: [
                (Some("disabled"), None, None, "muted", "muted-foreground", "muted-foreground", "muted-foreground"),
                (None, Some(false), None, "@outline_layer", "border", "foreground", "foreground"),
                (
                    None,
                    Some(true),
                    Some("filled"),
                    "@action_layer",
                    "@action_layer",
                    "@action_foreground",
                    "foreground",
                ),
                (
                    None,
                    Some(true),
                    Some("ring"),
                    "@outline_layer",
                    "@action_layer",
                    "@action_foreground",
                    "foreground",
                ),
                (None, Some(true), Some("dot"), "@outline_layer", "border", "@action_layer", "foreground"),
            ]
            .into_iter()
            .map(|(layer, selected, indicator, background, ring, dot, label)| RadioColorRule {
                layer: layer.map(Into::into),
                selected,
                indicator: indicator.map(Into::into),
                indicator_background: background.into(),
                selection_ring: ring.into(),
                dot_color: dot.into(),
                label_color: label.into(),
            })
            .collect(),
        }
    }

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

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct RadioColorRule {
    pub layer: Option<String>,
    pub selected: Option<bool>,
    pub indicator: Option<String>,
    pub indicator_background: String,
    pub selection_ring: String,
    pub dot_color: String,
    pub label_color: String,
}
