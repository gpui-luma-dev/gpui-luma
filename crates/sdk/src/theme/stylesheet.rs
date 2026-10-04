//! Common look configuration consumed by SDK controls.
//!
//! Looks embed this contract under `[common]` in their own `style.toml`.
//! Other root sections remain look-owned. Unknown common fields are rejected;
//! missing fields use SDK fallbacks, and explicit zero remains an override.

use std::time::Duration;
use std::collections::HashMap;
use crate::theme::provenance::ResolvedMetric;
use serde::Deserialize;

mod geometry;
pub use geometry::*;

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct CommonStylesheet {
    pub tabs: TabsStylesheet,
    pub switch: SwitchStylesheet,
    pub slider: SliderStylesheet,
    pub button: ButtonStylesheet,
    pub toggle: ToggleStylesheet,
    pub checkbox: CheckboxStylesheet,
    pub radio: RadioStylesheet,
    pub textfield: TextFieldStylesheet,
    pub textarea: TextAreaStylesheet,
    pub progress: ProgressStylesheet,
    pub floating_menu: FloatingMenuStylesheet,
    pub tooltip: TooltipStylesheet,
    pub toolbar: ToolbarStylesheet,
    pub segmented: SegmentedStylesheet,
    pub avatar: AvatarStylesheet,
    pub badge: BadgeStylesheet,
    pub card: CardStylesheet,
    pub callout: CalloutStylesheet,
    pub overlay_window: OverlayWindowStylesheet,
    pub selector: SelectorStylesheet,
    pub popup_menu: PopupMenuStylesheet,
    pub scrollbar: ScrollbarStylesheet,
    pub stepper: StepperStylesheet,
    pub sidebar: SidebarStylesheet,
    pub control_group: ControlGroupStylesheet,
    pub table: TableStylesheet,
    pub pager: PagerStylesheet,
    pub context_menu: ContextMenuStylesheet,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct TabsStylesheet {
    pub content: TabsContentStylesheet,
    pub geometry: TabsGeometryStylesheet,
    /// Variant names belong to each look; fields use the same common contract.
    pub variants: HashMap<String, TabsGeometryStylesheet>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct TabsContentStylesheet {
    /// Incoming-panel fade in milliseconds. Missing uses the SDK's zero fallback.
    pub fade_in_ms: Option<u32>,
}

/// Common tab geometry in logical pixels. Missing fields retain the supplied
/// token/recipe fallback; variant values take precedence over general values.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct TabsGeometryStylesheet {
    #[serde(deserialize_with = "optional_length")]
    pub list_gap: Option<f32>,
    #[serde(deserialize_with = "optional_length")]
    pub list_padding: Option<f32>,
    #[serde(deserialize_with = "optional_length")]
    pub indicator_height: Option<f32>,
}

fn optional_length<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<f32>, D::Error> {
    let value = Option::<f32>::deserialize(deserializer)?;
    if value.is_some_and(|value| !value.is_finite() || value < 0.0) {
        return Err(serde::de::Error::custom("geometry must be finite and nonnegative"));
    }
    Ok(value)
}

/// Shared switch dimensions. Size fields override general geometry field by field.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SwitchStylesheet {
    pub geometry: SwitchGeometryStylesheet,
    /// Semantic size keys belong to the look (for example Radix "1"/"2"/"3").
    pub sizes: HashMap<String, SwitchGeometryStylesheet>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SwitchGeometryStylesheet {
    #[serde(deserialize_with = "optional_length")]
    pub track_width: Option<f32>,
    #[serde(deserialize_with = "optional_length")]
    pub track_height: Option<f32>,
    #[serde(deserialize_with = "optional_length")]
    pub track_padding: Option<f32>,
    #[serde(deserialize_with = "optional_length")]
    pub thumb_size: Option<f32>,
    #[serde(deserialize_with = "optional_length")]
    pub gap: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct ResolvedSwitchGeometry {
    pub track_width: ResolvedMetric,
    pub track_height: ResolvedMetric,
    pub track_padding: ResolvedMetric,
    pub thumb_size: ResolvedMetric,
    pub gap: ResolvedMetric,
}

impl SwitchStylesheet {
    /// Resolve per-size, general, then supplied SDK/legacy fallback geometry.
    /// Authored values are snapped once at the requested display scale factor.
    pub fn resolve_geometry(
        &self,
        key: &str,
        fallback: crate::controls::switch::SwitchScale,
        scale_factor: f32,
    ) -> ResolvedSwitchGeometry {
        use crate::theme::snap_to_pixel;
        let specific = self.sizes.get(key).cloned().unwrap_or_default();
        let resolve = |field: &str, specific: Option<f32>, general: Option<f32>, fallback: f32| {
            if let Some(value) = specific {
                ResolvedMetric::authored(
                    snap_to_pixel(value, scale_factor),
                    format!("common.switch.sizes.{key}.{field}"),
                )
            } else if let Some(value) = general {
                ResolvedMetric::authored(snap_to_pixel(value, scale_factor), format!("common.switch.geometry.{field}"))
            } else {
                ResolvedMetric::constant(fallback, format!("switch {field} fallback"))
            }
        };
        ResolvedSwitchGeometry {
            track_width: resolve("track_width", specific.track_width, self.geometry.track_width, fallback.track_width),
            track_height: resolve(
                "track_height",
                specific.track_height,
                self.geometry.track_height,
                fallback.track_height,
            ),
            track_padding: resolve(
                "track_padding",
                specific.track_padding,
                self.geometry.track_padding,
                fallback.track_padding,
            ),
            thumb_size: resolve("thumb_size", specific.thumb_size, self.geometry.thumb_size, fallback.thumb_size),
            gap: resolve("gap", specific.gap, self.geometry.gap, fallback.gap),
        }
    }
}

impl ResolvedSwitchGeometry {
    /// Preserve the look-owned radius and baseline shift while supplying common inputs.
    pub fn apply_to(&self, mut scale: crate::controls::switch::SwitchScale) -> crate::controls::switch::SwitchScale {
        scale.track_width = self.track_width.value_px;
        scale.track_height = self.track_height.value_px;
        scale.track_padding = self.track_padding.value_px;
        scale.thumb_size = self.thumb_size.value_px;
        scale.gap = self.gap.value_px;
        scale
    }
}

/// Shared slider dimensions; palettes, shadows and radius recipes remain look-owned.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SliderStylesheet {
    pub geometry: SliderGeometryStylesheet,
    /// Semantic size keys belong to the look; the SDK does not normalize them.
    pub sizes: HashMap<String, SliderGeometryStylesheet>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SliderGeometryStylesheet {
    #[serde(deserialize_with = "optional_length")]
    pub width: Option<f32>,
    #[serde(deserialize_with = "optional_length")]
    pub height: Option<f32>,
    #[serde(deserialize_with = "optional_length")]
    pub track_height: Option<f32>,
    #[serde(deserialize_with = "optional_length")]
    pub thumb_size: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct ResolvedSliderGeometry {
    pub width: ResolvedMetric,
    pub height: ResolvedMetric,
    pub track_height: ResolvedMetric,
    pub thumb_size: ResolvedMetric,
}

impl SliderStylesheet {
    /// Resolve per-size, general, then SDK/legacy dimensions. Explicit thumb-size
    /// selections use the corresponding look size, independently of control size.
    /// Slider dimensions retain logical pixels without introducing display snapping.
    pub fn resolve_geometry(
        &self,
        key: &str,
        thumb_key: Option<&str>,
        fallback: &crate::controls::slider::SliderLook,
    ) -> ResolvedSliderGeometry {
        let specific = self.sizes.get(key).cloned().unwrap_or_default();
        let resolve = |key: &str, field: &str, specific: Option<f32>, general: Option<f32>, fallback: f32| {
            if let Some(value) = specific {
                ResolvedMetric::authored(value, format!("common.slider.sizes.{key}.{field}"))
            } else if let Some(value) = general {
                ResolvedMetric::authored(value, format!("common.slider.geometry.{field}"))
            } else {
                ResolvedMetric::constant(fallback, format!("slider {field} fallback"))
            }
        };
        let selected_thumb_key = thumb_key.unwrap_or(key);
        let thumb_geometry = self.sizes.get(selected_thumb_key).cloned().unwrap_or_default();
        let mut thumb = resolve(
            selected_thumb_key,
            "thumb_size",
            thumb_geometry.thumb_size,
            self.geometry.thumb_size,
            fallback.thumb_size,
        );
        if thumb_key.is_some() {
            let source = match &thumb.source {
                crate::theme::provenance::MetricSource::Authored { key } => key.clone(),
                _ => "SDK/legacy thumb fallback".into(),
            };
            thumb.source = crate::theme::provenance::MetricSource::Derived {
                note: format!("instance thumb_size {selected_thumb_key} → {source}"),
            };
        }
        ResolvedSliderGeometry {
            width: resolve(key, "width", specific.width, self.geometry.width, fallback.width),
            height: resolve(key, "height", specific.height, self.geometry.height, fallback.height),
            track_height: resolve(
                key,
                "track_height",
                specific.track_height,
                self.geometry.track_height,
                fallback.track_height,
            ),
            thumb_size: thumb,
        }
    }
}

/// The same resolved geometry is consumed by paint paths and inspectors.
#[derive(Clone, Debug)]
pub struct ResolvedTabsGeometry {
    pub list_gap: ResolvedMetric,
    pub list_padding: ResolvedMetric,
    pub indicator_height: ResolvedMetric,
}

impl TabsStylesheet {
    /// Resolve variant, general, then supplied look-token/SDK fallback values.
    pub fn resolve_geometry(&self, variant: &str, fallback: ResolvedTabsGeometry) -> ResolvedTabsGeometry {
        let specific = self.variants.get(variant);
        let resolve = |field: &str, general: Option<f32>, specific: Option<f32>, fallback: ResolvedMetric| {
            if let Some(value) = specific {
                ResolvedMetric::authored(value, format!("common.tabs.variants.{variant}.{field}"))
            } else if let Some(value) = general {
                ResolvedMetric::authored(value, format!("common.tabs.geometry.{field}"))
            } else {
                fallback
            }
        };
        ResolvedTabsGeometry {
            list_gap: resolve("list_gap", self.geometry.list_gap, specific.and_then(|v| v.list_gap), fallback.list_gap),
            list_padding: resolve(
                "list_padding",
                self.geometry.list_padding,
                specific.and_then(|v| v.list_padding),
                fallback.list_padding,
            ),
            indicator_height: resolve(
                "indicator_height",
                self.geometry.indicator_height,
                specific.and_then(|v| v.indicator_height),
                fallback.indicator_height,
            ),
        }
    }
}

/// Effective motion value and its origin, suitable for inspection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedMotion {
    pub duration: Duration,
    pub source: MotionSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MotionSource {
    SdkFallback,
    Look { name: String, field: &'static str },
    InstanceOverride,
    AnimationDisabled,
}

impl Default for ResolvedMotion {
    fn default() -> Self {
        Self { duration: Duration::ZERO, source: MotionSource::SdkFallback }
    }
}

impl CommonStylesheet {
    /// Parse a complete look stylesheet. Look-owned root sections are ignored.
    /// File access and input-size limits belong to the application.
    pub fn parse(source: &str) -> anyhow::Result<Self> {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Document {
            common: CommonStylesheet,
        }
        toml::from_str::<Document>(source)
            .map(|document| document.common)
            .map_err(|err| anyhow::anyhow!("parse common style.toml: {err}"))
    }

    pub fn tabs_content_motion(&self, look_name: &str) -> ResolvedMotion {
        match self.tabs.content.fade_in_ms {
            Some(ms) => ResolvedMotion {
                duration: Duration::from_millis(u64::from(ms)),
                source: MotionSource::Look { name: look_name.into(), field: "common.tabs.content.fade_in_ms" },
            },
            None => ResolvedMotion::default(),
        }
    }
}

impl ResolvedMotion {
    /// Control-wide disable takes precedence over both inherited and explicit motion.
    pub fn with_override(self, duration: Option<Duration>, animated: bool) -> Self {
        if !animated {
            Self { duration: Duration::ZERO, source: MotionSource::AnimationDisabled }
        } else if let Some(duration) = duration {
            Self { duration, source: MotionSource::InstanceOverride }
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_size_keys_remain_independent_and_look_owned() {
        use crate::controls::slider::default_slider_theme;
        use crate::controls::switch::SwitchScale;
        use crate::theme::ControlSize;
        let stylesheet = CommonStylesheet::parse(
            r#"
[common.slider.sizes."1"]
height = 16
[common.slider.sizes.sm]
height = 19
[common.slider.sizes."4"]
height = 28
[common.switch.sizes."1"]
gap = 5
[common.switch.sizes.compact]
gap = 7
"#,
        )
        .unwrap();
        let fallback = default_slider_theme().resolve(ControlSize::Sm, None, Default::default());
        for (key, height) in [("1", 16.0), ("sm", 19.0), ("4", 28.0)] {
            let geometry = stylesheet.slider.resolve_geometry(key, None, &fallback);
            assert_eq!(geometry.height.value_px, height);
            assert!(matches!(geometry.height.source,
                crate::theme::provenance::MetricSource::Authored { key: source }
                if source == format!("common.slider.sizes.{key}.height")));
        }
        let fallback = SwitchScale::compute(ControlSize::Sm, &Default::default(), 1.0);
        assert_eq!(stylesheet.switch.resolve_geometry("1", fallback, 1.0).gap.value_px, 5.0);
        assert_eq!(stylesheet.switch.resolve_geometry("compact", fallback, 1.0).gap.value_px, 7.0);
        assert_eq!(stylesheet.switch.resolve_geometry("sm", fallback, 1.0).gap.value_px, fallback.gap);
    }

    #[test]
    fn slider_geometry_validates_values_and_preserves_zero_and_thumb_selection() {
        use crate::controls::slider::{default_slider_theme, SliderThumbSize};
        use crate::theme::ControlSize;
        for size in ["geometry", "sizes.1", "sizes.md", "sizes.custom"] {
            for field in ["width", "height", "track_height", "thumb_size"] {
                for value in ["-1", "nan", "inf", "1e100", "\"4px\""] {
                    assert!(CommonStylesheet::parse(&format!("[common.slider.{size}]\n{field} = {value}")).is_err());
                }
            }
        }
        assert!(CommonStylesheet::parse("[common.slider.xl]\nwidth = 12").is_err());
        let stylesheet = CommonStylesheet::parse("[common.slider.geometry]\nwidth = 100\n[common.slider.sizes.md]\nwidth = 0\nthumb_size = 17\n[common.slider.sizes.lg]\nthumb_size = 23").unwrap();
        let fallback = default_slider_theme().resolve(ControlSize::Md, Some(SliderThumbSize::Lg), Default::default());
        let value = stylesheet.slider.resolve_geometry("md", Some("lg"), &fallback);
        assert_eq!(value.width.value_px, 0.0);
        assert_eq!(value.height.value_px, fallback.height);
        assert_eq!(value.thumb_size.value_px, 23.0);
        assert!(
            matches!(value.thumb_size.source, crate::theme::provenance::MetricSource::Derived { note } if note.contains("instance thumb_size lg") && note.contains("common.slider.sizes.lg.thumb_size"))
        );
        let value = stylesheet.slider.resolve_geometry("md", None, &fallback);
        assert_eq!(value.thumb_size.value_px, 17.0);
        assert_eq!(stylesheet.slider.resolve_geometry("sm", None, &fallback).width.value_px, 100.0);
    }

    #[test]
    fn switch_geometry_rejects_invalid_lengths_and_sizes() {
        for size in ["geometry", "sizes.1", "sizes.md", "sizes.custom"] {
            for field in ["track_width", "track_height", "track_padding", "thumb_size", "gap"] {
                for value in ["-1", "nan", "inf", "1e100", "\"4px\""] {
                    assert!(CommonStylesheet::parse(&format!("[common.switch.{size}]\n{field} = {value}")).is_err());
                }
            }
        }
        assert!(CommonStylesheet::parse("[common.switch.xl]\ngap = 2").is_err());
    }

    #[test]
    fn switch_size_precedence_zero_fallback_and_pixel_snapping() {
        use crate::theme::ControlSize;
        use crate::controls::switch::SwitchScale;
        let stylesheet = CommonStylesheet::parse("[common.switch.geometry]\ngap = 9.0\ntrack_padding = 1.25\n[common.switch.sizes.md]\ngap = 0\ntrack_width = 41.25").unwrap();
        let fallback = SwitchScale::compute(ControlSize::Md, &Default::default(), 2.0);
        let geometry = stylesheet.switch.resolve_geometry("md", fallback, 2.0);
        assert_eq!(geometry.gap.value_px, 0.0);
        assert_eq!(geometry.track_width.value_px, 41.5);
        assert_eq!(geometry.track_padding.value_px, 1.5);
        assert_eq!(geometry.thumb_size.value_px, fallback.thumb_size);
        assert!(
            matches!(geometry.gap.source, crate::theme::provenance::MetricSource::Authored { key } if key == "common.switch.sizes.md.gap")
        );
        assert_eq!(stylesheet.switch.resolve_geometry("sm", fallback, 2.0).gap.value_px, 9.0);
    }

    #[test]
    fn geometry_rejects_invalid_lengths_and_unknown_fields() {
        for section in ["geometry", "variants.line"] {
            for field in ["list_gap", "list_padding", "indicator_height"] {
                for value in ["-1.0", "nan", "inf", "1e100", "\"4px\""] {
                    assert!(CommonStylesheet::parse(&format!("[common.tabs.{section}]\n{field} = {value}")).is_err());
                }
            }
        }
        assert!(CommonStylesheet::parse("[common.tabs.geometry]\nindictor_height = 3").is_err());
    }

    #[test]
    fn geometry_preserves_fallback_and_variant_precedence() {
        let config = CommonStylesheet::parse("[common.tabs.geometry]\nlist_gap = 7.0\nindicator_height = 3.0\n[common.tabs.variants.line]\nlist_gap = 0.0").unwrap();
        let fallback = ResolvedTabsGeometry {
            list_gap: ResolvedMetric::constant(4.0, "fallback gap"),
            list_padding: ResolvedMetric::constant(2.0, "fallback padding"),
            indicator_height: ResolvedMetric::constant(2.0, "fallback indicator"),
        };
        let line = config.tabs.resolve_geometry("line", fallback.clone());
        assert_eq!(line.list_gap.value_px, 0.0);
        assert!(
            matches!(line.list_gap.source, crate::theme::provenance::MetricSource::Authored { key } if key == "common.tabs.variants.line.list_gap")
        );
        assert_eq!(line.list_padding.value_px, 2.0);
        assert!(matches!(line.list_padding.source, crate::theme::provenance::MetricSource::Constant { .. }));
        let other = config.tabs.resolve_geometry("other", fallback);
        assert_eq!(other.list_gap.value_px, 7.0);
        assert_eq!(other.indicator_height.value_px, 3.0);
    }

    #[test]
    fn missing_and_explicit_zero_have_distinct_sources() {
        let fallback = CommonStylesheet::parse("[button]\nheight = 40").unwrap().tabs_content_motion("test");
        let zero = CommonStylesheet::parse("[common.tabs.content]\nfade_in_ms = 0")
            .unwrap()
            .tabs_content_motion("test");
        assert_eq!(fallback.duration, zero.duration);
        assert_eq!(fallback.source, MotionSource::SdkFallback);
        assert!(matches!(zero.source, MotionSource::Look { .. }));
    }

    #[test]
    fn invalid_common_values_are_rejected() {
        for value in ["-1", "1.5", "\"300\"", "4294967296"] {
            assert!(CommonStylesheet::parse(&format!("[common.tabs.content]\nfade_in_ms = {value}")).is_err());
        }
        assert!(CommonStylesheet::parse("[common.tabs.content]\nfade_ms = 300").is_err());
    }

    #[test]
    fn precedence_keeps_zero_and_disable() {
        let inherited = CommonStylesheet::parse("[common.tabs.content]\nfade_in_ms = 300")
            .unwrap()
            .tabs_content_motion("test");
        assert_eq!(inherited.clone().with_override(None, true).duration, Duration::from_millis(300));
        let zero = inherited.clone().with_override(Some(Duration::ZERO), true);
        assert_eq!(zero.duration, Duration::ZERO);
        assert_eq!(zero.source, MotionSource::InstanceOverride);
        let disabled = inherited.with_override(Some(Duration::from_millis(900)), false);
        assert_eq!(disabled.duration, Duration::ZERO);
        assert_eq!(disabled.source, MotionSource::AnimationDisabled);
    }
}
