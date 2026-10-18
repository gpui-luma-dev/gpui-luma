use super::ControlSize;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MetricTokens {
    /// Default whole-control disabled opacity; family recipes may override it.
    #[serde(deserialize_with = "crate::theme::stylesheet::required_unit")]
    pub disabled_opacity: f32,
    pub spacing: SpacingTokens,
    pub radius: RadiusTokens,
    pub border_width: BorderWidthTokens,
    pub focus: FocusMetricTokens,
    pub control: ControlMetricScale,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SpacingTokens {
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s0: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s1: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s2: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s3: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s4: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s5: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s6: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RadiusTokens {
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub none: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub sm: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub md: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub lg: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub xl: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub pill: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BorderWidthTokens {
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub hairline: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub default: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub strong: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FocusMetricTokens {
    /// Visual gap between a control and its focus ring, in pixels.
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub gap: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub width: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub offset: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ControlMetricScale {
    pub sm: ControlMetricTokens,
    pub md: ControlMetricTokens,
    pub lg: ControlMetricTokens,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ControlMetricTokens {
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub radius: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub height: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub control_height: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub padding_x: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub padding_y: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub gap: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub icon_size: f32,
}

impl Default for MetricTokens {
    fn default() -> Self {
        include!(concat!(env!("OUT_DIR"), "/metrics.rs"))
    }
}

impl ControlMetricTokens {
    pub fn new(height: f32, padding_x: f32, padding_y: f32, gap: f32, radius: f32, icon_size: f32) -> Self {
        Self { radius, height, control_height: height, padding_x, padding_y, gap, icon_size }
    }
}

impl MetricTokens {
    pub fn for_size(&self, size: ControlSize) -> ControlMetricTokens {
        match size {
            ControlSize::Sm => self.control.sm,
            ControlSize::Md => self.control.md,
            ControlSize::Lg => self.control.lg,
        }
    }

    pub fn radius(&self, size: ControlSize) -> f32 {
        self.for_size(size).radius
    }

    pub fn control_height(&self, size: ControlSize) -> f32 {
        self.for_size(size).control_height
    }

    pub fn padding_x(&self, size: ControlSize) -> f32 {
        self.for_size(size).padding_x
    }

    pub fn padding_y(&self, size: ControlSize) -> f32 {
        self.for_size(size).padding_y
    }

    pub fn gap(&self, size: ControlSize) -> f32 {
        self.for_size(size).gap
    }

    pub fn icon_size(&self, size: ControlSize) -> f32 {
        self.for_size(size).icon_size
    }
}
