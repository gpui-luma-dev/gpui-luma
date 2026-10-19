use std::collections::HashMap;

use gpui_luma::theme::{ControlSize, InteractionLayer};
use serde::Deserialize;

use super::LayeredElevationRule;

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ButtonStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ButtonMetricsRule>,
    /// Look-owned metric-token references; literals live in the common contract.
    #[serde(default)]
    pub tokens: HashMap<String, ButtonMetricTokensRule>,
    #[serde(default)]
    pub elevation_rules: Vec<ButtonElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<ButtonColorRule>,
}

impl ButtonStylesheet {
    /// Native bundled rules in their original priority order.
    pub fn native_default() -> Self {
        let mut color_rules = Vec::new();
        for (style, foreground, base, hover, pressed) in [
            ("primary", "primary-foreground", "primary", "primary-hover", "primary-pressed"),
            ("secondary", "secondary-foreground", "secondary", "secondary-hover", "secondary-pressed"),
        ] {
            for (layer, background, label) in [
                ("default", base, foreground),
                ("hover", hover, foreground),
                ("pressed", pressed, foreground),
                ("disabled", "muted", "muted-foreground"),
            ] {
                color_rules.push(ButtonColorRule {
                    style: Some(style.into()),
                    layer: Some(layer.into()),
                    mode: None,
                    selected: None,
                    background: background.into(),
                    foreground: label.into(),
                    border: Some("@background".into()),
                });
            }
        }
        for selected in [true, false] {
            for layer in ["default", "hover", "pressed"] {
                for mode in ["light", "dark"] {
                    let background = match (selected, layer, mode) {
                        (true, "default", _) | (_, _, "light") if selected || layer != "default" => "accent",
                        (false, "default", "light") => "background",
                        (false, "default", _) => "input/30",
                        _ => "input/50",
                    };
                    color_rules.push(ButtonColorRule {
                        style: Some("outline".into()),
                        layer: Some(layer.into()),
                        mode: Some(mode.into()),
                        selected: Some(selected),
                        background: background.into(),
                        foreground: if selected || layer != "default" {
                            "accent-foreground"
                        } else {
                            "foreground"
                        }
                        .into(),
                        border: Some(if mode == "light" { "border" } else { "input" }.into()),
                    });
                }
            }
        }
        color_rules.push(ButtonColorRule {
            style: Some("outline".into()),
            layer: Some("disabled".into()),
            mode: None,
            selected: None,
            background: "transparent".into(),
            foreground: "muted-foreground".into(),
            border: Some("border".into()),
        });
        for selected in [true, false] {
            color_rules.push(ButtonColorRule {
                style: Some("ghost".into()),
                layer: Some("default".into()),
                mode: None,
                selected: Some(selected),
                background: if selected { "accent" } else { "transparent" }.into(),
                foreground: if selected { "accent-foreground" } else { "foreground" }.into(),
                border: None,
            });
            for layer in ["hover", "pressed"] {
                for mode in ["light", "dark"] {
                    color_rules.push(ButtonColorRule {
                        style: Some("ghost".into()),
                        layer: Some(layer.into()),
                        mode: Some(mode.into()),
                        selected: Some(selected),
                        background: if mode == "light" { "accent" } else { "accent/50" }.into(),
                        foreground: "accent-foreground".into(),
                        border: None,
                    });
                }
            }
        }
        color_rules.push(ButtonColorRule {
            style: Some("ghost".into()),
            layer: Some("disabled".into()),
            mode: None,
            selected: None,
            background: "transparent".into(),
            foreground: "muted-foreground".into(),
            border: None,
        });
        for layer in ["default", "hover", "pressed", "disabled"] {
            color_rules.push(ButtonColorRule {
                style: Some("content-only".into()),
                layer: Some(layer.into()),
                mode: None,
                selected: None,
                background: "transparent".into(),
                foreground: "foreground".into(),
                border: Some("transparent".into()),
            });
        }
        color_rules.push(ButtonColorRule {
            style: None,
            layer: None,
            mode: None,
            selected: None,
            background: "transparent".into(),
            foreground: "foreground".into(),
            border: None,
        });
        Self {
            tokens: [("sm", "metrics.control.sm"), ("md", "metrics.control.md"), ("lg", "metrics.control.lg")]
                .into_iter()
                .map(|(key, height)| {
                    (
                        key.into(),
                        ButtonMetricTokensRule { height: Some(height.into()), corner_radius: Some("radius".into()) },
                    )
                })
                .collect(),
            elevation_rules: [
                ("primary", "none"),
                ("secondary", "none"),
                ("ghost", "none"),
                ("outline", "shadow-xs"),
                ("content-only", "none"),
            ]
            .into_iter()
            .map(|(style, shadow)| ButtonElevationRule { style: style.into(), shadow: shadow.into() })
            .collect(),
            color_rules,
            ..Default::default()
        }
    }

    pub fn find_color_rule(&self, style: &str, layer: &str, mode: &str, selected: bool) -> Option<&ButtonColorRule> {
        self.color_rules.iter().find(|rule| rule.matches(style, layer, mode, selected))
    }

    pub fn elevation_rule_for_style(&self, style: &str) -> Option<&ButtonElevationRule> {
        self.elevation_rules.iter().find(|rule| rule.style == style)
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ButtonMetricsRule> {
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ToggleStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ButtonMetricsRule>,
    /// Look-owned metric-token references; literals live in the common contract.
    #[serde(default)]
    pub tokens: HashMap<String, ButtonMetricTokensRule>,
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
}

impl ToggleStylesheet {
    /// Native bundled toggle token references and elevation.
    pub fn native_default() -> Self {
        Self {
            tokens: ["sm", "md", "lg"]
                .into_iter()
                .map(|key| (key.into(), ButtonMetricTokensRule { height: None, corner_radius: Some("radius".into()) }))
                .collect(),
            elevation_rules: vec![
                LayeredElevationRule { layer: None, shadow: "shadow-sm".into() },
                LayeredElevationRule { layer: Some("disabled".into()), shadow: "none".into() },
            ],
            ..Default::default()
        }
    }

    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ButtonMetricsRule> {
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ButtonMetricsRule {
    pub height: String,
    pub padding_horizontal: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub corner_radius: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ButtonElevationRule {
    pub style: String,
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ButtonColorRule {
    pub style: Option<String>,
    pub layer: Option<String>,
    pub mode: Option<String>,
    pub selected: Option<bool>,

    pub background: String,
    pub foreground: String,
    pub border: Option<String>,
}

impl ButtonColorRule {
    pub fn matches(&self, style: &str, layer: &str, mode: &str, selected: bool) -> bool {
        self.style.as_deref().is_none_or(|value| value == "any" || value == style)
            && self.layer.as_deref().is_none_or(|value| value == "any" || value == layer)
            && self.mode.as_deref().is_none_or(|value| value == "any" || value == mode)
            && self.selected.is_none_or(|value| value == selected)
    }
}

/// Shadcn token references retained independently of common pixel dimensions.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ButtonMetricTokensRule {
    pub height: Option<String>,
    pub corner_radius: Option<String>,
}
