use gpui_luma::theme::{InteractionLayer};
use serde::Deserialize;

use super::{matches_optional_bool, matches_optional_layer};

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ResizablePanelsStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ResizablePanelsColorRule>,
}

impl ResizablePanelsStylesheet {
    /// Native bundled border and grip rules.
    pub fn native_default() -> Self {
        Self {
            color_rules: [
                (true, None, "muted-foreground", "muted-foreground", "muted-foreground"),
                (false, Some("default"), "border", "border", "accent"),
                (false, Some("hover"), "border", "border", "first_layer(accent)"),
                (false, Some("pressed"), "border", "border", "first_layer(accent)"),
                (false, Some("disabled"), "border", "border", "accent"),
            ]
            .into_iter()
            .map(|(disabled, layer, divider, grip, emphasis)| ResizablePanelsColorRule {
                disabled: Some(disabled),
                layer: layer.map(Into::into),
                border: "border".into(),
                divider: divider.into(),
                grip: grip.into(),
                grip_emphasis: emphasis.into(),
            })
            .collect(),
        }
    }

    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&ResizablePanelsColorRule> {
        self.color_rules.iter().find(|rule| {
            matches_optional_bool(rule.disabled, disabled) && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ResizablePanelsColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub border: String,
    pub divider: String,
    pub grip: String,
    pub grip_emphasis: String,
}
