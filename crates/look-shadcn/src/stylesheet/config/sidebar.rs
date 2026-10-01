use gpui_luma::theme::{InteractionLayer};
use serde::Deserialize;

use super::{matches_optional_bool, matches_optional_layer};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SidebarStylesheet {
    #[serde(default)]
    pub metrics: SidebarMetricsRule,
    #[serde(default)]
    pub container: SidebarContainerStylesheet,
    #[serde(default)]
    pub section: SidebarSectionStylesheet,
    #[serde(default)]
    pub branch: SidebarBranchStylesheet,
    #[serde(default)]
    pub item: SidebarItemStylesheet,
}

/// Layout metrics for [`gpui_luma::controls::sidebar::SidebarMetricScale`].
///
/// Values accept bare numbers, `px`, or `rem` (1rem = 16px).
#[derive(Debug, Deserialize, Clone)]
pub struct SidebarMetricsRule {
    #[serde(default = "sidebar_metric_width_expanded")]
    pub width_expanded: String,
    #[serde(default = "sidebar_metric_width_icon_rail")]
    pub width_icon_rail: String,
    #[serde(default = "sidebar_metric_width_mobile")]
    pub width_mobile: String,
    #[serde(default = "sidebar_metric_item_height")]
    pub item_height: String,
    #[serde(default = "sidebar_metric_icon_size")]
    pub icon_size: String,
    #[serde(default = "sidebar_metric_rail_hit_width")]
    pub rail_hit_width: String,
    #[serde(default = "sidebar_metric_popover_offset")]
    pub popover_offset: String,
}

impl Default for SidebarMetricsRule {
    fn default() -> Self {
        Self {
            width_expanded: sidebar_metric_width_expanded(),
            width_icon_rail: sidebar_metric_width_icon_rail(),
            width_mobile: sidebar_metric_width_mobile(),
            item_height: sidebar_metric_item_height(),
            icon_size: sidebar_metric_icon_size(),
            rail_hit_width: sidebar_metric_rail_hit_width(),
            popover_offset: sidebar_metric_popover_offset(),
        }
    }
}

fn sidebar_metric_width_expanded() -> String {
    "16rem".into()
}
fn sidebar_metric_width_icon_rail() -> String {
    "3rem".into()
}
fn sidebar_metric_width_mobile() -> String {
    "18rem".into()
}
fn sidebar_metric_item_height() -> String {
    "2rem".into()
}
fn sidebar_metric_icon_size() -> String {
    "1rem".into()
}
fn sidebar_metric_rail_hit_width() -> String {
    "0.375rem".into()
}
fn sidebar_metric_popover_offset() -> String {
    "0.5rem".into()
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SidebarContainerStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SidebarContainerColorRule>,
}

impl SidebarContainerStylesheet {
    pub fn color_rule(&self) -> Option<&SidebarContainerColorRule> {
        self.color_rules.first()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SidebarContainerColorRule {
    pub background: String,
    pub foreground: String,
    pub border: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SidebarSectionStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SidebarSectionColorRule>,
}

impl SidebarSectionStylesheet {
    pub fn color_rule(&self) -> Option<&SidebarSectionColorRule> {
        self.color_rules.first()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SidebarSectionColorRule {
    pub label_color: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SidebarBranchStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SidebarBranchColorRule>,
}

impl SidebarBranchStylesheet {
    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&SidebarBranchColorRule> {
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
pub struct SidebarBranchColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub foreground: String,
    pub icon_color: String,
    pub background: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SidebarItemStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SidebarItemColorRule>,
}

impl SidebarItemStylesheet {
    pub fn find_color_rule(
        &self,
        selected: bool,
        disabled: bool,
        layer: InteractionLayer,
    ) -> Option<&SidebarItemColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled && matches_optional_bool(rule.selected, selected);
            }
            if disabled {
                return false;
            }
            matches_optional_bool(rule.selected, selected) && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SidebarItemColorRule {
    pub selected: Option<bool>,
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub foreground: String,
    pub icon_color: String,
    pub background: Option<String>,
}
