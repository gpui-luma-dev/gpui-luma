use std::collections::HashMap;

use gpui_luma::theme::{InteractionLayer};
use serde::Deserialize;

use super::matches_optional_layer;

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct FloatingMenuStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, FloatingMenuMetricsRule>,
    #[serde(default)]
    pub surface: FloatingMenuSurfaceStylesheet,
    #[serde(default)]
    pub trigger: FloatingMenuTriggerStylesheet,
}

impl FloatingMenuStylesheet {
    /// Native bundled menu surface and trigger rules.
    pub fn native_default() -> Self {
        Self {
            surface: FloatingMenuSurfaceStylesheet {
                elevation_rules: vec![FloatingMenuSurfaceElevationRule { shadow: "shadow-md".into() }],
                color_rules: vec![FloatingMenuSurfaceColorRule {
                    background: "first(popover,card)".into(),
                    foreground: "popover-foreground".into(),
                    border: "border".into(),
                    item_hover_background: "accent".into(),
                    item_hover_foreground: "accent-foreground".into(),
                    item_disabled_foreground: "muted-foreground".into(),
                }],
            },
            trigger: FloatingMenuTriggerStylesheet {
                color_rules: [
                    (true, None, "muted", "muted-foreground"),
                    (false, Some("hover"), "transparent", "accent-foreground"),
                    (false, Some("pressed"), "transparent", "accent-foreground"),
                    (false, Some("default"), "transparent", "foreground"),
                    (false, Some("disabled"), "transparent", "foreground"),
                ]
                .into_iter()
                .map(|(disabled, layer, background, foreground)| FloatingMenuTriggerColorRule {
                    disabled: Some(disabled),
                    layer: layer.map(Into::into),
                    background: background.into(),
                    foreground: foreground.into(),
                })
                .collect(),
            },
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct FloatingMenuMetricsRule {
    pub min_width: f32,
    pub radius: String,
    pub item_height_factor: f32,
    pub item_padding_x_factor: f32,
    pub item_radius: String,
    pub submenu_offset_x_factor: f32,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct FloatingMenuSurfaceStylesheet {
    #[serde(default)]
    pub elevation_rules: Vec<FloatingMenuSurfaceElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<FloatingMenuSurfaceColorRule>,
}

impl FloatingMenuSurfaceStylesheet {
    pub fn elevation_rule(&self) -> Option<&FloatingMenuSurfaceElevationRule> {
        self.elevation_rules.first()
    }

    pub fn color_rule(&self) -> Option<&FloatingMenuSurfaceColorRule> {
        self.color_rules.first()
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct FloatingMenuSurfaceElevationRule {
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct FloatingMenuSurfaceColorRule {
    pub background: String,
    pub foreground: String,
    pub border: String,
    pub item_hover_background: String,
    pub item_hover_foreground: String,
    pub item_disabled_foreground: String,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct FloatingMenuTriggerStylesheet {
    #[serde(default)]
    pub color_rules: Vec<FloatingMenuTriggerColorRule>,
}

impl FloatingMenuTriggerStylesheet {
    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&FloatingMenuTriggerColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct FloatingMenuTriggerColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub background: String,
    pub foreground: String,
}
