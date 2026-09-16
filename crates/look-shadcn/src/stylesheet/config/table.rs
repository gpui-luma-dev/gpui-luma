use std::collections::HashMap;

use luma::theme::{ControlSize, InteractionLayer};
use serde::Deserialize;

use super::{EnabledColorRule, find_enabled_color_rule, matches_optional_layer};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TableStylesheet {
    #[serde(default)]
    pub surface: TableSurfaceStylesheet,
    #[serde(default)]
    pub row: TableRowStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TableSurfaceStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, TableSurfaceMetricsRule>,
    #[serde(default)]
    pub color_rules: Vec<TableSurfaceColorRule>,
}

impl TableSurfaceStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&TableSurfaceColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&TableSurfaceMetricsRule> {
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TableSurfaceMetricsRule {
    pub padding_y_factor: f32,
    pub radius: String,
    #[serde(default)]
    pub max_radius: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TableSurfaceColorRule {
    pub enabled: Option<bool>,
    pub background: String,
    pub border: String,
    pub header_background: String,
    pub header_label_color: String,
}

impl EnabledColorRule for TableSurfaceColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TableRowStylesheet {
    #[serde(default)]
    pub color_rules: Vec<TableRowColorRule>,
}

impl TableRowStylesheet {
    pub fn find_color_rule(
        &self,
        selected: bool,
        focused: bool,
        disabled: bool,
        layer: InteractionLayer,
    ) -> Option<&TableRowColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            if rule.selected == Some(true) {
                return selected;
            }
            if rule.focused == Some(true) {
                return focused;
            }
            if selected || focused {
                return false;
            }
            matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TableRowColorRule {
    pub disabled: Option<bool>,
    pub selected: Option<bool>,
    pub focused: Option<bool>,
    pub layer: Option<String>,
    pub background: String,
    pub label_color: String,
    pub divider: String,
}
