use gpui_luma::theme::{InteractionLayer};
use serde::Deserialize;

use super::{matches_optional_bool, matches_optional_layer};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AccordionStylesheet {
    #[serde(default)]
    pub trigger: AccordionTriggerStylesheet,
    #[serde(default)]
    pub content: AccordionContentStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AccordionTriggerStylesheet {
    #[serde(default)]
    pub color_rules: Vec<AccordionTriggerColorRule>,
}

impl AccordionTriggerStylesheet {
    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&AccordionTriggerColorRule> {
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
pub struct AccordionTriggerColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub foreground: String,
    pub icon_color: String,
    pub chevron_color: String,
    pub border_color: String,
    pub background: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AccordionContentStylesheet {
    #[serde(default)]
    pub color_rules: Vec<AccordionContentColorRule>,
}

impl AccordionContentStylesheet {
    pub fn find_color_rule(&self, expanded: bool) -> Option<&AccordionContentColorRule> {
        self.color_rules.iter().find(|rule| matches_optional_bool(rule.expanded, expanded))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AccordionContentColorRule {
    pub expanded: Option<bool>,
    pub foreground: String,
    pub background: Option<String>,
}
