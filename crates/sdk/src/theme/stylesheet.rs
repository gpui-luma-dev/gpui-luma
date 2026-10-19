//! Common look configuration consumed by SDK controls.
//!
//! Looks author this contract in Rust; external documents use `[common]`.
//! Other root sections remain look-owned. Unknown common fields are rejected;
//! missing fields inherit SDK data, and explicit zero remains an override.

use std::collections::HashMap;
use crate::theme::provenance::ResolvedMetric;
use serde::{Deserialize, Serialize};

mod recipes;
pub use recipes::*;
mod icons;
pub use icons::*;
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

macro_rules! common_stylesheet {
    ($($field:ident: $kind:ty),+ $(,)?) => {
        #[derive(Clone, Debug, Default, PartialEq, Serialize)]
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

            /// Apply an in-place mutation or decorator closure.
            pub fn with(mut self, decorator: impl FnOnce(&mut Self)) -> Self {
                decorator(&mut self);
                self
            }
        }
    };
}
common_stylesheet! {
    shell: ShellStylesheet,
    shared: SharedStylesheet,
    row_limits: RowLimitsStylesheet,
    recipes: PaletteRecipes,
    font_features: FontFeaturesStylesheet,
    icons: IconsStylesheet,
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
    split_view: SplitViewStylesheet,
    resizable_panels: ResizablePanelsStylesheet,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct TabsStylesheet {
    pub geometry: TabsGeometryStylesheet,
    /// Variant names belong to each look; fields use the same common contract.
    pub variants: HashMap<String, TabsGeometryStylesheet>,
}

/// Common tab geometry in logical pixels. Missing fields retain the supplied
/// token/recipe fallback; variant values take precedence over general values.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct TabsGeometryStylesheet {
    #[serde(deserialize_with = "optional_length")]
    pub accessory_size: Option<f32>,
    #[serde(deserialize_with = "optional_length")]
    pub accessory_gap: Option<f32>,
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

pub(crate) fn required_unit<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    let value = f32::deserialize(deserializer)?;
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
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
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SwitchStylesheet {
    pub geometry: SwitchGeometryStylesheet,
    /// Semantic size keys belong to the look (for example Radix "1"/"2"/"3").
    pub sizes: HashMap<String, SwitchGeometryStylesheet>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SwitchGeometryStylesheet {
    #[serde(deserialize_with = "optional_unit")]
    pub disabled_opacity: Option<f32>,
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
    pub disabled_opacity: ResolvedMetric,
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
        thumb_scale,
        disabled_opacity
    ]
);

impl SwitchStylesheet {
    /// Construct the SDK switch geometry without parsing bundled data.
    pub fn native_default() -> Self {
        let mut switch = Self::default();
        switch
            .disabled_opacity(0.56)
            .track_padding(2.0)
            .size("sm", |m| {
                m.scales(32.0 / 28.0, 13.0 / 28.0, 9.0 / 28.0);
            })
            .size("md", |m| {
                m.scales(40.0 / 36.0, 18.0 / 36.0, 14.0 / 36.0);
            })
            .size("lg", |m| {
                m.scales(48.0 / 44.0, 24.0 / 44.0, 20.0 / 44.0);
            });
        switch
    }

    /// Configure general switch disabled opacity (0.0 to 1.0).
    pub fn disabled_opacity(&mut self, opacity: f32) -> &mut Self {
        self.geometry.disabled_opacity = Some(opacity);
        self
    }

    /// Configure general switch track width in logical pixels.
    pub fn track_width(&mut self, width: f32) -> &mut Self {
        self.geometry.track_width = Some(width);
        self
    }

    /// Configure general switch track height in logical pixels.
    pub fn track_height(&mut self, height: f32) -> &mut Self {
        self.geometry.track_height = Some(height);
        self
    }

    /// Configure general switch track padding in logical pixels.
    pub fn track_padding(&mut self, padding: f32) -> &mut Self {
        self.geometry.track_padding = Some(padding);
        self
    }

    /// Configure general switch thumb size in logical pixels.
    pub fn thumb_size(&mut self, thumb: f32) -> &mut Self {
        self.geometry.thumb_size = Some(thumb);
        self
    }

    /// Configure general switch label gap in logical pixels.
    pub fn gap(&mut self, gap: f32) -> &mut Self {
        self.geometry.gap = Some(gap);
        self
    }

    /// Configure general switch track width scale relative to control height.
    pub fn track_width_scale(&mut self, scale: f32) -> &mut Self {
        self.geometry.track_width_scale = Some(scale);
        self
    }

    /// Configure general switch track height scale relative to control height.
    pub fn track_height_scale(&mut self, scale: f32) -> &mut Self {
        self.geometry.track_height_scale = Some(scale);
        self
    }

    /// Configure general switch thumb scale relative to control height.
    pub fn thumb_scale(&mut self, scale: f32) -> &mut Self {
        self.geometry.thumb_scale = Some(scale);
        self
    }

    /// Configure or override a size key without manual map manipulation.
    pub fn size(&mut self, key: impl Into<String>, configure: impl FnOnce(&mut SwitchGeometryStylesheet)) -> &mut Self {
        let entry = self.sizes.entry(key.into()).or_default();
        configure(entry);
        self
    }

    /// Apply authored switch fields with the existing stylesheet precedence.
    pub fn apply_to(&self, target: &mut Self) {
        *target = self.clone().with_baseline(target);
    }

    /// Resolve optional dimensions and proportional recipes over general fields.
    pub fn authored_geometry(&self, key: &str) -> SwitchGeometryStylesheet {
        let specific = self.sizes.get(key).cloned().unwrap_or_default();
        macro_rules! field {
            ($name:ident) => {
                specific.$name.or(self.geometry.$name)
            };
        }
        SwitchGeometryStylesheet {
            disabled_opacity: field!(disabled_opacity),
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
        let disabled_opacity = if let Some(value) = specific.disabled_opacity {
            ResolvedMetric::authored(value, format!("common.switch.sizes.{key}.disabled_opacity"))
        } else if let Some(value) = self.geometry.disabled_opacity {
            ResolvedMetric::authored(value, "common.switch.geometry.disabled_opacity")
        } else {
            ResolvedMetric::constant(fallback.disabled_opacity, "switch disabled opacity fallback")
        };
        ResolvedSwitchGeometry {
            disabled_opacity,
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

impl SwitchGeometryStylesheet {
    pub fn disabled_opacity(&mut self, opacity: f32) -> &mut Self {
        self.disabled_opacity = Some(opacity);
        self
    }

    pub fn track_width(&mut self, width: f32) -> &mut Self {
        self.track_width = Some(width);
        self
    }

    pub fn track_height(&mut self, height: f32) -> &mut Self {
        self.track_height = Some(height);
        self
    }

    pub fn track_padding(&mut self, padding: f32) -> &mut Self {
        self.track_padding = Some(padding);
        self
    }

    pub fn thumb_size(&mut self, thumb: f32) -> &mut Self {
        self.thumb_size = Some(thumb);
        self
    }

    pub fn gap(&mut self, gap: f32) -> &mut Self {
        self.gap = Some(gap);
        self
    }

    pub fn track_width_scale(&mut self, scale: f32) -> &mut Self {
        self.track_width_scale = Some(scale);
        self
    }

    pub fn track_height_scale(&mut self, scale: f32) -> &mut Self {
        self.track_height_scale = Some(scale);
        self
    }

    pub fn thumb_scale(&mut self, scale: f32) -> &mut Self {
        self.thumb_scale = Some(scale);
        self
    }

    /// Convenience setter for track width, track height, and thumb size in logical pixels.
    pub fn dimensions(&mut self, track_width: f32, track_height: f32, thumb_size: f32) -> &mut Self {
        self.track_width(track_width);
        self.track_height(track_height);
        self.thumb_size(thumb_size);
        self
    }

    /// Convenience setter for track width scale, track height scale, and thumb scale.
    pub fn scales(&mut self, track_width_scale: f32, track_height_scale: f32, thumb_scale: f32) -> &mut Self {
        self.track_width_scale(track_width_scale);
        self.track_height_scale(track_height_scale);
        self.thumb_scale(thumb_scale);
        self
    }
}

impl ResolvedSwitchGeometry {
    /// Preserve the look-owned radius and baseline shift while supplying common inputs.
    pub fn apply_to(&self, mut scale: crate::controls::switch::SwitchScale) -> crate::controls::switch::SwitchScale {
        scale.disabled_opacity = self.disabled_opacity.value_px;
        scale.track_width = self.track_width.value_px;
        scale.track_height = self.track_height.value_px;
        scale.track_padding = self.track_padding.value_px;
        scale.thumb_size = self.thumb_size.value_px;
        scale.gap = self.gap.value_px;
        scale
    }
}

/// Shared slider dimensions; palettes, shadows and radius recipes remain look-owned.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SliderStylesheet {
    pub geometry: SliderGeometryStylesheet,
    /// Semantic size keys belong to the look; the SDK does not normalize them.
    pub sizes: HashMap<String, SliderGeometryStylesheet>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SliderGeometryStylesheet {
    #[serde(deserialize_with = "optional_length")]
    pub dial_size: Option<f32>,
    #[serde(deserialize_with = "optional_length")]
    pub track_radius: Option<f32>,
    #[serde(deserialize_with = "optional_unit")]
    pub disabled_opacity: Option<f32>,
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
    [
        width,
        height,
        track_height,
        thumb_size,
        height_floor_scale,
        thumb_scale,
        dial_size,
        track_radius,
        disabled_opacity
    ]
);

/// Nonlinear slider presentation metrics, in logical pixels and unit opacity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderDecoration {
    pub dial_size: f32,
    pub track_radius: f32,
    pub disabled_opacity: f32,
}

impl SliderStylesheet {
    /// Resolve radial paint dimensions and disabled opacity for the selected size.
    pub fn decoration(&self, key: &str) -> SliderDecoration {
        let recipe = self.authored_geometry(key);
        let baseline = CommonStylesheet::sdk_default().slider.authored_geometry(key);
        SliderDecoration {
            dial_size: recipe.dial_size.or(baseline.dial_size).unwrap_or_default(),
            track_radius: recipe.track_radius.or(baseline.track_radius).unwrap_or_default(),
            disabled_opacity: recipe.disabled_opacity.or(baseline.disabled_opacity).unwrap_or_default(),
        }
    }
    /// Optional size/general inputs, including token-derived recipe coefficients.
    pub fn authored_geometry(&self, key: &str) -> SliderGeometryStylesheet {
        let specific = self.sizes.get(key).cloned().unwrap_or_default();
        macro_rules! field {
            ($name:ident) => {
                specific.$name.or(self.geometry.$name)
            };
        }
        SliderGeometryStylesheet {
            dial_size: field!(dial_size),
            track_radius: field!(track_radius),
            disabled_opacity: field!(disabled_opacity),
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

    /// Configure general slider width in logical pixels.
    pub fn width(&mut self, width: f32) -> &mut Self {
        self.geometry.width = Some(width);
        self
    }

    /// Configure general slider height in logical pixels.
    pub fn height(&mut self, height: f32) -> &mut Self {
        self.geometry.height = Some(height);
        self
    }

    /// Configure general slider track height in logical pixels.
    pub fn track_height(&mut self, track_height: f32) -> &mut Self {
        self.geometry.track_height = Some(track_height);
        self
    }

    /// Configure general slider thumb size in logical pixels.
    pub fn thumb_size(&mut self, thumb_size: f32) -> &mut Self {
        self.geometry.thumb_size = Some(thumb_size);
        self
    }

    /// Configure general slider height floor scale relative to control height.
    pub fn height_floor_scale(&mut self, scale: f32) -> &mut Self {
        self.geometry.height_floor_scale = Some(scale);
        self
    }

    /// Configure general slider thumb scale relative to resolved height.
    pub fn thumb_scale(&mut self, scale: f32) -> &mut Self {
        self.geometry.thumb_scale = Some(scale);
        self
    }

    /// Configure general slider radial dial size in logical pixels.
    pub fn dial_size(&mut self, dial_size: f32) -> &mut Self {
        self.geometry.dial_size = Some(dial_size);
        self
    }

    /// Configure general slider radial track radius in logical pixels.
    pub fn track_radius(&mut self, track_radius: f32) -> &mut Self {
        self.geometry.track_radius = Some(track_radius);
        self
    }

    /// Configure general slider disabled opacity (0.0 to 1.0).
    pub fn disabled_opacity(&mut self, opacity: f32) -> &mut Self {
        self.geometry.disabled_opacity = Some(opacity);
        self
    }

    /// Configure or override a size key without manual map manipulation.
    pub fn size(&mut self, key: impl Into<String>, configure: impl FnOnce(&mut SliderGeometryStylesheet)) -> &mut Self {
        let entry = self.sizes.entry(key.into()).or_default();
        configure(entry);
        self
    }

    /// Construct the native SDK slider stylesheet in pure Rust without preloading TOML.
    pub fn native_default() -> Self {
        let mut slider = Self::default();
        slider
            .width(260.0)
            .height_floor_scale(0.5)
            .thumb_scale(0.5)
            .dial_size(200.0)
            .track_radius(72.0)
            .disabled_opacity(0.56);
        slider
    }

    /// Apply authored slider fields with the existing stylesheet precedence.
    pub fn apply_to(&self, target: &mut Self) {
        *target = self.clone().with_baseline(target);
    }
}

impl SliderGeometryStylesheet {
    pub fn width(&mut self, width: f32) -> &mut Self {
        self.width = Some(width);
        self
    }

    pub fn height(&mut self, height: f32) -> &mut Self {
        self.height = Some(height);
        self
    }

    pub fn track_height(&mut self, track_height: f32) -> &mut Self {
        self.track_height = Some(track_height);
        self
    }

    pub fn thumb_size(&mut self, thumb_size: f32) -> &mut Self {
        self.thumb_size = Some(thumb_size);
        self
    }

    pub fn height_floor_scale(&mut self, scale: f32) -> &mut Self {
        self.height_floor_scale = Some(scale);
        self
    }

    pub fn thumb_scale(&mut self, scale: f32) -> &mut Self {
        self.thumb_scale = Some(scale);
        self
    }

    pub fn dial_size(&mut self, dial_size: f32) -> &mut Self {
        self.dial_size = Some(dial_size);
        self
    }

    pub fn track_radius(&mut self, track_radius: f32) -> &mut Self {
        self.track_radius = Some(track_radius);
        self
    }

    pub fn disabled_opacity(&mut self, opacity: f32) -> &mut Self {
        self.disabled_opacity = Some(opacity);
        self
    }

    /// Convenience setter for height, track height, and thumb size in logical pixels.
    pub fn dimensions(&mut self, height: f32, track_height: f32, thumb_size: f32) -> &mut Self {
        self.height(height);
        self.track_height(track_height);
        self.thumb_size(thumb_size);
        self
    }
}

impl SliderMotionStylesheet {
    /// Construct native SDK slider motion specs in pure Rust without preloading TOML.
    pub fn native_default() -> Self {
        Self { thumb: Some(TransitionSpec::Timed { duration_ms: 200, easing: crate::motion::Easing::OutCubic }) }
    }
}

/// Construct native SDK slider palette recipe mappings in pure Rust without preloading TOML.
pub fn native_sdk_slider_recipes() -> HashMap<PaletteRole, PaletteRole> {
    use PaletteRole::*;
    HashMap::from([
        (StateDisabledBackground, StateDisabledBackground),
        (StateDisabledForeground, StateDisabledForeground),
        (StatePressedBackground, StatePressedBackground),
        (StateSelectedBackground, StateSelectedBackground),
        (StateSelectedForeground, StateSelectedForeground),
        (SurfacePanelBackground, SurfacePanelBackground),
        (SurfaceSubtleBackground, SurfaceSubtleBackground),
    ])
}

/// Native SDK slider defaults.
pub fn define_sdk_slider(sheet: &mut CommonStylesheet) {
    sheet.slider = SliderStylesheet::native_default();
    sheet.motion.slider = SliderMotionStylesheet::native_default();
    sheet.recipes.0.insert("slider".into(), native_sdk_slider_recipes());
}

impl SwitchMotionStylesheet {
    /// Construct the SDK switch motion without parsing bundled data.
    pub fn native_default() -> Self {
        Self { state: Some(TransitionSpec::Timed { duration_ms: 200, easing: crate::motion::Easing::OutCubic }) }
    }
}

/// Construct SDK switch palette references without parsing bundled data.
pub fn native_sdk_switch_recipes() -> HashMap<PaletteRole, PaletteRole> {
    use PaletteRole::*;
    [
        AppBackground,
        AppForeground,
        BorderDefault,
        FocusRing,
        FormInputBackground,
        StateDisabledBackground,
        StateDisabledForeground,
        StateSelectedBackground,
        StateSelectedForeground,
    ]
    .into_iter()
    .map(|role| (role, role))
    .collect()
}

/// Install the native SDK switch baseline.
pub fn define_sdk_switch(sheet: &mut CommonStylesheet) {
    sheet.switch = SwitchStylesheet::native_default();
    sheet.motion.switch = SwitchMotionStylesheet::native_default();
    sheet.recipes.0.insert("switch".into(), native_sdk_switch_recipes());
}

impl CheckboxStylesheet {
    /// Construct the SDK checkbox geometry without parsing bundled data.
    pub fn native_default() -> Self {
        let mut checkbox = Self::default();
        checkbox
            .disabled_opacity(0.56)
            .control_padding_x(0.0)
            .control_padding_y(0.0)
            .size("sm", |m| {
                m.indicator(0.45, 2.0, 2.0);
            })
            .size("md", |m| {
                m.indicator(0.5, 4.0, 3.0);
            })
            .size("lg", |m| {
                m.indicator(0.55, 6.0, 4.0);
            });
        checkbox
    }

    /// Apply authored checkbox fields with the existing stylesheet precedence.
    pub fn apply_to(&self, target: &mut Self) {
        *target = self.clone().with_baseline(target);
    }

    pub fn disabled_opacity(&mut self, opacity: f32) -> &mut Self {
        self.geometry.disabled_opacity = Some(opacity);
        self
    }

    pub fn indicator_size(&mut self, size: f32) -> &mut Self {
        self.geometry.indicator_size = Some(size);
        self
    }

    pub fn indicator_scale(&mut self, scale: f32) -> &mut Self {
        self.geometry.indicator_scale = Some(scale);
        self
    }

    pub fn indicator_radius(&mut self, radius: f32) -> &mut Self {
        self.geometry.indicator_radius = Some(radius);
        self
    }

    pub fn glyph_size(&mut self, size: f32) -> &mut Self {
        self.geometry.glyph_size = Some(size);
        self
    }

    pub fn glyph_inset(&mut self, inset: f32) -> &mut Self {
        self.geometry.glyph_inset = Some(inset);
        self
    }

    pub fn control_padding_x(&mut self, padding: f32) -> &mut Self {
        self.geometry.control_padding_x = Some(padding);
        self
    }

    pub fn control_padding_y(&mut self, padding: f32) -> &mut Self {
        self.geometry.control_padding_y = Some(padding);
        self
    }

    pub fn gap(&mut self, gap: f32) -> &mut Self {
        self.geometry.gap = Some(gap);
        self
    }

    pub fn size(
        &mut self,
        key: impl Into<String>,
        configure: impl FnOnce(&mut CheckboxGeometryStylesheet),
    ) -> &mut Self {
        let entry = self.sizes.entry(key.into()).or_default();
        configure(entry);
        self
    }
}

impl CheckboxGeometryStylesheet {
    pub fn disabled_opacity(&mut self, opacity: f32) -> &mut Self {
        self.disabled_opacity = Some(opacity);
        self
    }

    pub fn indicator_size(&mut self, size: f32) -> &mut Self {
        self.indicator_size = Some(size);
        self
    }

    pub fn indicator_scale(&mut self, scale: f32) -> &mut Self {
        self.indicator_scale = Some(scale);
        self
    }

    pub fn indicator_radius(&mut self, radius: f32) -> &mut Self {
        self.indicator_radius = Some(radius);
        self
    }

    pub fn glyph_size(&mut self, size: f32) -> &mut Self {
        self.glyph_size = Some(size);
        self
    }

    pub fn glyph_inset(&mut self, inset: f32) -> &mut Self {
        self.glyph_inset = Some(inset);
        self
    }

    pub fn control_padding_x(&mut self, padding: f32) -> &mut Self {
        self.control_padding_x = Some(padding);
        self
    }

    pub fn control_padding_y(&mut self, padding: f32) -> &mut Self {
        self.control_padding_y = Some(padding);
        self
    }

    pub fn gap(&mut self, gap: f32) -> &mut Self {
        self.gap = Some(gap);
        self
    }

    pub fn indicator(&mut self, scale: f32, radius: f32, glyph_inset: f32) -> &mut Self {
        self.indicator_scale(scale);
        self.indicator_radius(radius);
        self.glyph_inset(glyph_inset);
        self
    }
}

impl CheckboxMotionStylesheet {
    /// Construct the SDK checkbox motion without parsing bundled data.
    pub fn native_default() -> Self {
        Self { state: Some(TransitionSpec::Timed { duration_ms: 200, easing: crate::motion::Easing::OutCubic }) }
    }
}

/// Construct SDK checkbox palette references without parsing bundled data.
pub fn native_sdk_checkbox_recipes() -> HashMap<PaletteRole, PaletteRole> {
    use PaletteRole::*;
    [
        AppForeground,
        FocusRing,
        FormInputBackground,
        FormInputBorder,
        StateDisabledBackground,
        StateDisabledForeground,
        StateHoverBackground,
        StatePressedBackground,
        StateSelectedBackground,
        StateSelectedForeground,
    ]
    .into_iter()
    .map(|role| (role, role))
    .collect()
}

/// Install the native SDK checkbox baseline.
pub fn define_sdk_checkbox(sheet: &mut CommonStylesheet) {
    sheet.checkbox = CheckboxStylesheet::native_default();
    sheet.motion.checkbox = CheckboxMotionStylesheet::native_default();
    sheet.recipes.0.insert("checkbox".into(), native_sdk_checkbox_recipes());
}

/// The same resolved geometry is consumed by paint paths and inspectors.
#[derive(Clone, Debug)]
pub struct ResolvedTabsGeometry {
    pub list_gap: ResolvedMetric,
    pub list_padding: ResolvedMetric,
    pub indicator_height: ResolvedMetric,
}

geometry::inherit_geometry!(
    TabsStylesheet,
    variants,
    [list_gap, list_padding, indicator_height, accessory_size, accessory_gap]
);

impl TabsStylesheet {
    /// Accessory glyph size and gap in pixels, with variant and SDK inheritance.
    pub fn accessory_metrics(&self, variant: &str) -> (f32, f32) {
        let baseline = &CommonStylesheet::sdk_default().tabs.geometry;
        let specific = self.variants.get(variant);
        (
            specific
                .and_then(|v| v.accessory_size)
                .or(self.geometry.accessory_size)
                .or(baseline.accessory_size)
                .unwrap_or_default(),
            specific
                .and_then(|v| v.accessory_gap)
                .or(self.geometry.accessory_gap)
                .or(baseline.accessory_gap)
                .unwrap_or_default(),
        )
    }
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
    /// Construct the bundled SDK stylesheet using native control definitions.
    pub fn native_default() -> Self {
        let mut sheet = Self {
            motion: MotionStylesheet::native_default(),
            radio: RadioStylesheet::native_default(),
            progress: ProgressStylesheet::native_default(),
            stepper: StepperStylesheet::native_default(),
            scrollbar: ScrollbarStylesheet::native_default(),
            tooltip: TooltipStylesheet::native_default(),
            floating_menu: FloatingMenuStylesheet::native_default(),
            context_menu: ContextMenuStylesheet::native_default(),
            control_group: ControlGroupStylesheet::native_default(),
            dock_splitter: DockSplitterStylesheet::native_default(),
            resizable_panels: ResizablePanelsStylesheet::native_default(),
            overlay_window: OverlayWindowStylesheet::native_default(),
            sidebar: SidebarStylesheet::native_default(),
            tree_view: TreeViewStylesheet::native_default(),
            accordion: AccordionStylesheet::native_default(),
            table: TableStylesheet::native_default(),
            selector: SelectorStylesheet::native_default(),
            pager: PagerStylesheet::native_default(),
            popup_menu: PopupMenuStylesheet::native_default(),
            popover: PopoverStylesheet::native_default(),
            slide_panel: SlidePanelStylesheet::native_default(),
            radio_group: RadioGroupStylesheet::native_default(),
            autocomplete: AutocompleteStylesheet::native_default(),
            combobox: ComboBoxStylesheet::native_default(),
            search_selector: SearchSelectorStylesheet::native_default(),
            textfield: TextFieldStylesheet::native_default(),
            textarea: TextAreaStylesheet::native_default(),
            split_view: SplitViewStylesheet::native_default(),
            tabs: TabsStylesheet {
                geometry: TabsGeometryStylesheet {
                    accessory_size: Some(12.0),
                    accessory_gap: Some(4.0),
                    list_padding: Some(0.0),
                    indicator_height: Some(2.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            shell: ShellStylesheet {
                geometry: ShellGeometryStylesheet {
                    fullscreen_left_padding: Some(12.0),
                    height: Some(34.0),
                    macos_left_padding: Some(80.0),
                    left_padding: Some(12.0),
                    traffic_light_inset: Some(9.0),
                    icon_size: Some(16.0),
                    border_width: Some(1.0),
                },
                ..Default::default()
            },
            shared: SharedStylesheet {
                geometry: SharedGeometryStylesheet {
                    popup_viewport_height: Some(220.0),
                    popup_row_height: Some(28.0),
                    menu_row_height: Some(27.0),
                    menu_row_gap: Some(4.0),
                    compact_textfield_height: Some(22.0),
                    compact_textfield_padding_y: Some(2.0),
                    textarea_resize_icon_size: Some(10.0),
                    textarea_resize_inset: Some(3.0),
                    textarea_resize_hit_size: Some(12.0),
                    textarea_scrollbar_width: Some(12.0),
                    textarea_scrollbar_inset: Some(2.0),
                    scrollbar_width: Some(12.0),
                    composition_underline_width: Some(1.0),
                    label_baseline_sm: Some(0.5),
                    label_baseline_md: Some(1.0),
                    label_baseline_lg: Some(1.5),
                    drag_offset: Some(12.0),
                    drag_preview_width: Some(220.0),
                    drag_padding_x: Some(12.0),
                    drag_padding_y: Some(8.0),
                    drag_border_width: Some(1.0),
                    drag_marker_width: Some(2.0),
                    split_sidebar_width: Some(280.0),
                    split_sidebar_min_width: Some(220.0),
                    split_sidebar_max_width: Some(420.0),
                    selection_scrollbar_width: Some(16.0),
                    textarea_resize_opacity: Some(0.75),
                    wide_middle_gap: Some(24.0),
                    wide_side_min_width: Some(240.0),
                    wide_side_preferred_width: Some(280.0),
                    wide_middle_min_width: Some(435.0),
                    wide_middle_max_width: Some(435.0),
                    form_field_gap: Some(4.0),
                    page_size_trigger_width: Some(64.0),
                    popup_icon_size: Some(16.0),
                },
                ..Default::default()
            },
            row_limits: RowLimitsStylesheet(HashMap::from([
                ("autocomplete".into(), RowLimits { min: Some(7), max: Some(14) }),
                ("combobox".into(), RowLimits { min: Some(1), max: Some(7) }),
                ("search_selector".into(), RowLimits { min: Some(1), max: Some(7) }),
                ("selection_panel".into(), RowLimits { min: Some(1), max: Some(7) }),
                ("pager".into(), RowLimits { min: Some(5), max: Some(7) }),
            ])),
            font_features: FontFeaturesStylesheet(HashMap::from([(
                "sidebar_section".into(),
                vec![("smcp".into(), 1)],
            )])),
            icons: IconsStylesheet::native_default(),
            ..Default::default()
        };
        define_sdk_slider(&mut sheet);
        define_sdk_switch(&mut sheet);
        define_sdk_checkbox(&mut sheet);
        use PaletteRole::*;
        for (family, roles) in [
            (
                "button",
                &[
                    AppForeground,
                    AppMutedForeground,
                    BorderDefault,
                    FocusRing,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StatePressedBackground,
                    StateSelectedBackground,
                    StateSelectedForeground,
                    SurfaceSubtleBackground,
                ][..],
            ),
            (
                "control_group",
                &[AppMutedForeground, BorderDefault, StateDisabledBackground, SurfaceSubtleBackground][..],
            ),
            (
                "radio_button",
                &[
                    AppForeground,
                    FocusRing,
                    FormInputBackground,
                    FormInputBorder,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StatePressedBackground,
                    StateSelectedBackground,
                    StateSelectedForeground,
                ][..],
            ),
            (
                "table",
                &[
                    AppForeground,
                    AppMutedForeground,
                    DataAccent1,
                    FocusRing,
                    FormInputBackground,
                    FormInputBorder,
                    FormInputInvalidBorder,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StatePressedBackground,
                    SurfaceFloatingBackground,
                    SurfaceFloatingForeground,
                    SurfaceSubtleBackground,
                    SurfaceSubtleForeground,
                ][..],
            ),
            (
                "tree_view",
                &[
                    AppForeground,
                    NavigationHoverBackground,
                    NavigationMutedForeground,
                    NavigationSelectedBackground,
                    NavigationSelectedForeground,
                    StateDisabledForeground,
                    StatePressedBackground,
                ][..],
            ),
            ("dock_splitter", &[BorderDefault, StateDisabledForeground, StateSelectedBackground][..]),
            (
                "resizable_panels",
                &[BorderDefault, BorderStrong, StateDisabledBackground, StateDisabledForeground, StateHoverBackground]
                    [..],
            ),
            (
                "scrollbar",
                &[
                    BorderDefault,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StatePressedBackground,
                    SurfaceSubtleBackground,
                ][..],
            ),
            ("split_view", &[BorderDefault, BorderStrong, StateDisabledBackground, StateDisabledForeground][..]),
            (
                "accordion",
                &[
                    AppForeground,
                    NavigationBorder,
                    NavigationHoverBackground,
                    NavigationMutedForeground,
                    StateDisabledForeground,
                    StatePressedBackground,
                ][..],
            ),
            (
                "pager",
                &[
                    AppForeground,
                    AppMutedForeground,
                    FormInputBorder,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StateSelectedBackground,
                    StateSelectedForeground,
                    SurfacePanelBackground,
                ][..],
            ),
            (
                "sidebar",
                &[
                    FocusRing,
                    NavigationForeground,
                    NavigationHoverBackground,
                    NavigationMutedForeground,
                    NavigationSelectedBackground,
                    NavigationSelectedForeground,
                    StateDisabledForeground,
                    StatePressedBackground,
                ][..],
            ),
            (
                "stepper",
                &[
                    BorderDefault,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateSelectedBackground,
                    SurfaceSubtleBackground,
                    SurfaceSubtleForeground,
                ][..],
            ),
            (
                "tabs",
                &[
                    AppForeground,
                    FocusRing,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StatePressedBackground,
                    StateSelectedBackground,
                ][..],
            ),
            (
                "context_menu",
                &[
                    AppBackground,
                    AppForeground,
                    BorderDefault,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StatePressedBackground,
                ][..],
            ),
            (
                "floating_menu",
                &[
                    DataPositive,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StateHoverForeground,
                    SurfaceFloatingBackground,
                    SurfaceFloatingBorder,
                    SurfaceFloatingForeground,
                ][..],
            ),
            ("overlay_window", &[SurfaceFloatingBackground, SurfaceFloatingBorder, SurfaceFloatingForeground][..]),
            (
                "popup_menu",
                &[
                    AppBackground,
                    AppForeground,
                    BorderDefault,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StatePressedBackground,
                    SurfaceSubtleBackground,
                ][..],
            ),
            (
                "progress",
                &[StateDisabledBackground, StateDisabledForeground, StateSelectedBackground, SurfaceSubtleBackground][..],
            ),
            (
                "selector",
                &[
                    AppBackground,
                    AppForeground,
                    AppMutedForeground,
                    BorderDefault,
                    FocusRing,
                    FormInputInvalidBorder,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateHoverBackground,
                    StatePressedBackground,
                ][..],
            ),
            (
                "textarea",
                &[
                    FormInputBackground,
                    FormInputBorder,
                    FormInputForeground,
                    FormInputInvalidBorder,
                    FormInputPlaceholder,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateSelectedBackground,
                    StateSelectedForeground,
                ][..],
            ),
            (
                "textfield",
                &[
                    FormInputBackground,
                    FormInputBorder,
                    FormInputForeground,
                    FormInputInvalidBorder,
                    FormInputPlaceholder,
                    StateDisabledBackground,
                    StateDisabledForeground,
                    StateSelectedBackground,
                    StateSelectedForeground,
                ][..],
            ),
            ("tooltip", &[SurfaceFloatingBackground, SurfaceFloatingForeground][..]),
            ("autocomplete", &[FormInputForeground, StateSelectedBackground, SurfaceFloatingForeground][..]),
            ("toolbar", &[BorderDefault, StateDisabledBackground, SurfaceSubtleBackground][..]),
            (
                "shell",
                &[
                    ShellBackground,
                    ShellForeground,
                    ShellBorder,
                    ShellHoverBackground,
                    ShellActiveBackground,
                    ShellCloseHoverBackground,
                    ShellCloseActiveBackground,
                    ShellCloseForeground,
                ][..],
            ),
            (
                "drag_drop",
                &[SurfaceFloatingBackground, SurfaceFloatingForeground, FocusRing, FormInputInvalidBorder][..],
            ),
        ] {
            sheet.recipes.0.insert(family.into(), roles.iter().map(|role| (*role, *role)).collect());
        }
        sheet
    }

    /// Resolve validated layers over the SDK baseline, in order. Look-specific
    /// roots are ignored. Programmatic edits precede explicit control overrides.
    pub fn resolve_layers(layers: &[&str]) -> anyhow::Result<Self> {
        let mut document = toml::Value::Table(toml::map::Map::from_iter([(
            "common".into(),
            toml::Value::try_from(Self::sdk_default())?,
        )]));
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
    /// let sheet = CommonStylesheet::sdk_default().clone().with(|sheet| {
    ///     sheet.progress.sizes.entry("md".into()).or_default().track_height = Some(9.0);
    /// });
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
        DEFAULT.get_or_init(Self::native_default)
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
            .map_err(|err| anyhow::anyhow!("parse common stylesheet: {err}"))
    }
}

impl CommonStylesheet {
    /// Shared helper dimensions resolved from this layer over the native SDK baseline.
    pub fn shared_geometry(&self) -> ResolvedSharedGeometry {
        self.shared
            .clone()
            .with_baseline(&Self::sdk_default().shared)
            .resolve_geometry("", SharedGeometry::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_adapter_preserves_the_complete_native_baseline() {
        assert_eq!(&CommonStylesheet::resolve_layers(&[]).unwrap(), CommonStylesheet::sdk_default());
        let native = CommonStylesheet::sdk_default().clone().with(|sheet| {
            sheet.slider.geometry.width = Some(0.0);
            sheet.font_features.0.insert("sidebar_section".into(), Vec::new());
            sheet.tokens.elevation.light.menu.layers.clear();
        });
        let source = toml::to_string(&native).unwrap();
        let restored: CommonStylesheet = toml::from_str(&source).unwrap();
        assert_eq!(restored, native);
    }

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

    #[test]
    fn slider_overlay_general_geometry_overrides_inherited_sizes() {
        for width in [120.0, 0.0] {
            let mut target = SliderStylesheet::native_default();
            target.size("sm", |size| {
                size.width(260.0).dimensions(24.0, 4.0, 12.0);
            });
            target.size("md", |size| {
                size.width(260.0).dimensions(32.0, 6.0, 16.0);
            });
            let mut layer = SliderStylesheet::default();
            layer.width(width).size("md", |size| {
                size.width(180.0);
            });

            layer.apply_to(&mut target);

            assert_eq!(target.authored_geometry("sm").width, Some(width));
            assert_eq!(target.authored_geometry("md").width, Some(180.0));
            assert_eq!(target.authored_geometry("sm").height, Some(24.0));
            assert_eq!(target.authored_geometry("md").thumb_size, Some(16.0));
            assert_eq!(target.geometry.dial_size, Some(200.0));
        }
    }

    #[test]
    fn slider_builder_methods_configure_geometry_and_sizes_fluently() {
        let mut sheet = CommonStylesheet::default();
        sheet
            .slider
            .width(120.0)
            .height_floor_scale(0.5)
            .thumb_scale(0.5)
            .size("1", |m| {
                m.dimensions(16.0, 4.0, 12.0);
            })
            .size("2", |m| {
                m.dimensions(20.0, 6.0, 16.0);
            });

        assert_eq!(sheet.slider.geometry.width, Some(120.0));
        assert_eq!(sheet.slider.geometry.height_floor_scale, Some(0.5));
        let size1 = sheet.slider.sizes.get("1").unwrap();
        assert_eq!(size1.height, Some(16.0));
        assert_eq!(size1.track_height, Some(4.0));
        assert_eq!(size1.thumb_size, Some(12.0));

        let sheet = CommonStylesheet::default().with(|s| {
            s.slider.size("3", |m| {
                m.dimensions(24.0, 8.0, 20.0);
            });
        });
        assert_eq!(sheet.slider.sizes.get("3").unwrap().height, Some(24.0));
    }

    #[test]
    fn switch_builder_methods_configure_geometry_and_sizes_fluently() {
        let mut sheet = CommonStylesheet::default();
        sheet.switch.track_padding(1.0).gap(8.0).size("1", |m| {
            m.dimensions(28.0, 16.0, 14.0);
        });

        assert_eq!(sheet.switch.geometry.track_padding, Some(1.0));
        assert_eq!(sheet.switch.geometry.gap, Some(8.0));
        let size1 = sheet.switch.sizes.get("1").unwrap();
        assert_eq!(size1.track_width, Some(28.0));
        assert_eq!(size1.track_height, Some(16.0));
        assert_eq!(size1.thumb_size, Some(14.0));

        let sheet = CommonStylesheet::default().with(|s| {
            s.switch.size("sm", |m| {
                m.scales(32.0 / 28.0, 13.0 / 28.0, 9.0 / 28.0);
            });
        });
        assert_eq!(sheet.switch.sizes.get("sm").unwrap().track_width_scale, Some(32.0 / 28.0));
    }

    #[test]
    fn checkbox_builder_methods_configure_geometry_and_sizes_fluently() {
        let mut sheet = CommonStylesheet::default();
        sheet.checkbox.disabled_opacity(0.56).gap(8.0).size("1", |m| {
            m.indicator_size(14.0).glyph_size(9.0);
        });

        assert_eq!(sheet.checkbox.geometry.disabled_opacity, Some(0.56));
        assert_eq!(sheet.checkbox.geometry.gap, Some(8.0));
        let size1 = sheet.checkbox.sizes.get("1").unwrap();
        assert_eq!(size1.indicator_size, Some(14.0));
        assert_eq!(size1.glyph_size, Some(9.0));
    }
}
