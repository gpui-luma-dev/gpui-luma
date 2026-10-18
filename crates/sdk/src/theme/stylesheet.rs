//! Common look configuration consumed by SDK controls.
//!
//! Looks embed this contract under `[common]` in their own `style.toml`.
//! Other root sections remain look-owned. Unknown common fields are rejected;
//! missing fields inherit SDK data, and explicit zero remains an override.

use std::collections::HashMap;
use crate::theme::provenance::ResolvedMetric;
use serde::Deserialize;

mod geometry;
pub use geometry::*;
mod motion;
pub use motion::*;
mod colors;
mod math;
mod tokens;
pub use tokens::*;

#[cfg(test)]
mod control_defaults;

/// Embedded vanilla SDK configuration. Looks author only their overrides.
pub const DEFAULT_STYLE_TOML: &str = include_str!("../../assets/style.toml");

macro_rules! common_stylesheet {
    ($($field:ident: $kind:ty),+ $(,)?) => {
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct CommonStylesheet {
            pub tokens: TokenStylesheet,
            /// Typed role overrides, shared by SDK-only apps and all looks.
            pub motion: MotionStylesheet,
            $(pub $field: $kind,)+
        }
        impl<'de> serde::Deserialize<'de> for CommonStylesheet {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                #[derive(Default, Deserialize)]
                #[serde(default, deny_unknown_fields)]
                struct Fields {
                    tokens: TokenStylesheet,
                    motion: MotionStylesheet,
                    $($field: $kind,)+
                }
                let mut value = toml::Value::deserialize(deserializer)?;
                math::normalize(&mut value).map_err(serde::de::Error::custom)?;
                let fields: Fields = value.try_into().map_err(serde::de::Error::custom)?;
                Ok(Self { tokens: fields.tokens, motion: fields.motion, $($field: fields.$field,)+ })
            }
        }
        impl CommonStylesheet {
            /// Fill missing fields from the SDK baseline before capturing a style snapshot.
            /// Explicit zero/false values and this layer's geometry retain precedence.
            pub fn with_sdk_defaults(&self) -> Self {
                let baseline = Self::sdk_default();
                Self {
                    tokens: self.tokens.clone(), motion: self.motion.clone(),
                    $($field: self.$field.clone().with_baseline(&baseline.$field),)+
                }
            }
        }
    };
}
common_stylesheet! {
    tree_view: TreeViewStylesheet,
    accordion: AccordionStylesheet,
    tabs: TabsStylesheet,
    switch: SwitchStylesheet,
    slider: SliderStylesheet,
    button: ButtonStylesheet,
    toggle: ToggleStylesheet,
    checkbox: CheckboxStylesheet,
    radio: RadioStylesheet,
    textfield: TextFieldStylesheet,
    textarea: TextAreaStylesheet,
    progress: ProgressStylesheet,
    floating_menu: FloatingMenuStylesheet,
    selection_panel: SelectionPanelStylesheet,
    tooltip: TooltipStylesheet,
    toolbar: ToolbarStylesheet,
    segmented: SegmentedStylesheet,
    avatar: AvatarStylesheet,
    badge: BadgeStylesheet,
    card: CardStylesheet,
    callout: CalloutStylesheet,
    overlay_window: OverlayWindowStylesheet,
    selector: SelectorStylesheet,
    selector_list: SelectorListStylesheet,
    autocomplete: AutocompleteStylesheet,
    combobox: ComboBoxStylesheet,
    search_selector: SearchSelectorStylesheet,
    popup_menu: PopupMenuStylesheet,
    popover: PopoverStylesheet,
    slide_panel: SlidePanelStylesheet,
    scrollbar: ScrollbarStylesheet,
    stepper: StepperStylesheet,
    sidebar: SidebarStylesheet,
    control_group: ControlGroupStylesheet,
    radio_group: RadioGroupStylesheet,
    icon_group: IconGroupStylesheet,
    table: TableStylesheet,
    pager: PagerStylesheet,
    context_menu: ContextMenuStylesheet,
    dock_splitter: DockSplitterStylesheet,
    resizable_panels: ResizablePanelsStylesheet,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct TabsStylesheet {
    pub geometry: TabsGeometryStylesheet,
    /// Variant names belong to each look; fields use the same common contract.
    pub variants: HashMap<String, TabsGeometryStylesheet>,
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

fn optional_unit<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<f32>, D::Error> {
    let value = Option::<f32>::deserialize(deserializer)?;
    if value.is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value)) {
        return Err(serde::de::Error::custom("opacity must be finite and in 0..=1"));
    }
    Ok(value)
}

pub(crate) fn required_length<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    let value = f32::deserialize(deserializer)?;
    if !value.is_finite() || value < 0.0 {
        return Err(serde::de::Error::custom("length must be finite and nonnegative"));
    }
    Ok(value)
}

pub(crate) fn required_finite<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    let value = f32::deserialize(deserializer)?;
    if !value.is_finite() {
        return Err(serde::de::Error::custom("value must be finite"));
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
    /// Track width divided by control height; applied before pixel snapping.
    #[serde(deserialize_with = "optional_length")]
    pub track_width_scale: Option<f32>,
    /// Track height divided by control height; applied before pixel snapping.
    #[serde(deserialize_with = "optional_length")]
    pub track_height_scale: Option<f32>,
    /// Thumb size divided by control height; applied before pixel snapping.
    #[serde(deserialize_with = "optional_length")]
    pub thumb_scale: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct ResolvedSwitchGeometry {
    pub track_width: ResolvedMetric,
    pub track_height: ResolvedMetric,
    pub track_padding: ResolvedMetric,
    pub thumb_size: ResolvedMetric,
    pub gap: ResolvedMetric,
}

geometry::inherit_geometry!(
    SwitchStylesheet,
    sizes,
    [
        track_width,
        track_height,
        track_padding,
        thumb_size,
        gap,
        track_width_scale,
        track_height_scale,
        thumb_scale
    ]
);

impl SwitchStylesheet {
    /// Resolve optional dimensions and proportional recipes over general fields.
    pub fn authored_geometry(&self, key: &str) -> SwitchGeometryStylesheet {
        let specific = self.sizes.get(key).cloned().unwrap_or_default();
        macro_rules! field {
            ($name:ident) => {
                specific.$name.or(self.geometry.$name)
            };
        }
        SwitchGeometryStylesheet {
            track_width: field!(track_width),
            track_height: field!(track_height),
            track_padding: field!(track_padding),
            thumb_size: field!(thumb_size),
            gap: field!(gap),
            track_width_scale: field!(track_width_scale),
            track_height_scale: field!(track_height_scale),
            thumb_scale: field!(thumb_scale),
        }
    }

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
    /// Minimum target height relative to the shared control height.
    #[serde(deserialize_with = "optional_length")]
    pub height_floor_scale: Option<f32>,
    /// Default thumb diameter relative to the selected target height.
    #[serde(deserialize_with = "optional_length")]
    pub thumb_scale: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct ResolvedSliderGeometry {
    pub width: ResolvedMetric,
    pub height: ResolvedMetric,
    pub track_height: ResolvedMetric,
    pub thumb_size: ResolvedMetric,
}

geometry::inherit_geometry!(
    SliderStylesheet,
    sizes,
    [width, height, track_height, thumb_size, height_floor_scale, thumb_scale]
);

impl SliderStylesheet {
    /// Optional size/general inputs, including token-derived recipe coefficients.
    pub fn authored_geometry(&self, key: &str) -> SliderGeometryStylesheet {
        let specific = self.sizes.get(key).cloned().unwrap_or_default();
        macro_rules! field {
            ($name:ident) => {
                specific.$name.or(self.geometry.$name)
            };
        }
        SliderGeometryStylesheet {
            width: field!(width),
            height: field!(height),
            track_height: field!(track_height),
            thumb_size: field!(thumb_size),
            height_floor_scale: field!(height_floor_scale),
            thumb_scale: field!(thumb_scale),
        }
    }

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

geometry::inherit_geometry!(TabsStylesheet, variants, [list_gap, list_padding, indicator_height]);

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

impl CommonStylesheet {
    /// Resolve validated layers over the SDK baseline, in order. Look-specific
    /// roots are ignored. Programmatic edits precede explicit control overrides.
    pub fn resolve_layers(layers: &[&str]) -> anyhow::Result<Self> {
        let mut document: toml::Value = toml::from_str(DEFAULT_STYLE_TOML)?;
        for source in layers {
            Self::parse(source)?;
            let authored: toml::Value = toml::from_str(source)?;
            tokens::overlay(&mut document, authored);
        }
        Self::parse(&toml::to_string(&document)?)
    }

    /// Prepare an immutable SDK-only or custom-look snapshot. Rebuild tokens
    /// after editing the sheet; existing themes retain their captured styling.
    ///
    /// ```
    /// use gpui_luma::{
    ///     color::GamutMapping,
    ///     controls::progress::{DefaultProgressTheme, ProgressTheme},
    ///     theme::{ControlSize, ThemeMode, stylesheet::CommonStylesheet},
    /// };
    /// let sheet = CommonStylesheet::resolve_layers(&[
    ///     "[common.progress.sizes.md]\ntrack_height = 9",
    /// ])?;
    /// let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde)?;
    /// let theme = DefaultProgressTheme::new(tokens);
    /// assert_eq!(theme.resolve(true, ControlSize::Md).track_height, 9.0);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn theme_tokens(
        &self,
        mode: crate::theme::ThemeMode,
        mapping: crate::color::GamutMapping,
    ) -> anyhow::Result<crate::theme::ThemeTokens> {
        let sheet = self.with_sdk_defaults();
        let mut tokens = crate::theme::ThemeTokens::from_source(sheet.tokens.source(mode), mapping)?;
        tokens.stylesheet = std::sync::Arc::new(sheet);
        Ok(tokens)
    }

    /// The SDK-owned baseline, independent of any look crate. `Default` remains
    /// an empty geometry/motion overlay with inherited token data.
    pub fn sdk_default() -> &'static Self {
        static DEFAULT: std::sync::OnceLock<CommonStylesheet> = std::sync::OnceLock::new();
        // A corrupt packaged asset settles motion safely. Tests require the
        // embedded file to parse and define every role, preventing silent drift.
        DEFAULT.get_or_init(|| Self::parse(DEFAULT_STYLE_TOML).unwrap_or_default())
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obsolete_motion_keys_are_rejected() {
        for source in ["[common.tabs.content]\nfade_in_ms = 300", "[common.drag_drop]\ndrop_motion_ms = 200"] {
            assert!(CommonStylesheet::parse(source).is_err(), "accepted {source}");
        }
    }

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
}
