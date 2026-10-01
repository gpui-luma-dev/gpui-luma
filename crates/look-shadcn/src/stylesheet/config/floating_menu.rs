use std::collections::HashMap;

use gpui_luma::theme::{InteractionLayer};
use serde::Deserialize;

use super::matches_optional_layer;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FloatingMenuStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, FloatingMenuMetricsRule>,
    #[serde(default)]
    pub surface: FloatingMenuSurfaceStylesheet,
    #[serde(default)]
    pub trigger: FloatingMenuTriggerStylesheet,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FloatingMenuMetricsRule {
    pub min_width: f32,
    pub radius: String,
    pub item_height_factor: f32,
    pub item_padding_x_factor: f32,
    pub item_radius: String,
    pub submenu_offset_x_factor: f32,
}

#[derive(Debug, Deserialize, Clone, Default)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct FloatingMenuSurfaceElevationRule {
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FloatingMenuSurfaceColorRule {
    pub background: String,
    pub foreground: String,
    pub border: String,
    pub item_hover_background: String,
    pub item_hover_foreground: String,
    pub item_disabled_foreground: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct FloatingMenuTriggerColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub background: String,
    pub foreground: String,
}
