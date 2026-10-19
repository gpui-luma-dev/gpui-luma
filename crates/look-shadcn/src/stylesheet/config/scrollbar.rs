use std::collections::HashMap;

use gpui_luma::theme::{ControlSize, InteractionLayer};
use serde::Deserialize;

use super::{control_size_key, matches_optional_bool, matches_optional_layer, matches_optional_str};

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ScrollbarStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ScrollbarMetricsRule>,
    #[serde(default)]
    pub color_rules: Vec<ScrollbarColorRule>,
}

impl ScrollbarStylesheet {
    /// Native bundled rules, retaining disabled-first matching.
    pub fn native_default() -> Self {
        Self {
            color_rules: [
                (None, true, None, "muted", "muted-foreground"),
                (None, false, Some("disabled"), "transparent", "muted-foreground"),
                (Some("ghost"), false, Some("default"), "transparent", "border"),
                (Some("ghost"), false, Some("hover"), "transparent", "border"),
                (Some("ghost"), false, Some("pressed"), "transparent", "@darken_border"),
                (Some("soft"), false, Some("default"), "muted", "border"),
                (Some("soft"), false, Some("hover"), "muted", "border"),
                (Some("soft"), false, Some("pressed"), "muted", "@darken_border"),
            ]
            .into_iter()
            .map(|(style, disabled, layer, track, thumb)| ScrollbarColorRule {
                style: style.map(Into::into),
                disabled: Some(disabled),
                layer: layer.map(Into::into),
                track_background: track.into(),
                thumb_background: thumb.into(),
            })
            .collect(),
            ..Default::default()
        }
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ScrollbarMetricsRule> {
        self.metrics.get(control_size_key(size))
    }

    pub fn find_color_rule(&self, style: &str, disabled: bool, layer: InteractionLayer) -> Option<&ScrollbarColorRule> {
        self.color_rules.iter().find(|rule| {
            matches_optional_str(rule.style.as_deref(), style)
                && matches_optional_bool(rule.disabled, disabled)
                && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
pub struct ScrollbarMetricsRule {
    pub thickness: f32,
    pub track_thickness: f32,
    pub thumb_thickness: f32,
    pub min_thumb_length: f32,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ScrollbarColorRule {
    pub style: Option<String>,
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub track_background: String,
    pub thumb_background: String,
}
