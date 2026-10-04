//! Typed common dimensions for remaining control families.
use std::collections::HashMap;
use serde::Deserialize;
use super::optional_length;
use crate::theme::provenance::ResolvedMetric;

macro_rules! geometry_family {
    ($stylesheet:ident, $authored:ident, $fallback:ident, $resolved:ident, $family:literal, [$($field:ident),+]) => {
        /// Common template dimensions, with semantic size keys supplied by each look.
        #[derive(Clone, Debug, Default, Deserialize, PartialEq)]
        #[serde(default, deny_unknown_fields)]
        pub struct $stylesheet {
            pub geometry: $authored,
            pub sizes: HashMap<String, $authored>,
        }
        /// Optional logical pixel values. Authored zero overrides the fallback.
        #[derive(Clone, Debug, Default, Deserialize, PartialEq)]
        #[serde(default, deny_unknown_fields)]
        pub struct $authored {
            $(#[serde(deserialize_with = "optional_length")] pub $field: Option<f32>,)+
        }
        /// Dimensions supplied by the SDK or look tokens when configuration is unset.
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $fallback { $(pub $field: f32,)+ }
        /// Resolved common dimensions with the source of each value.
        #[derive(Clone, Debug)]
        pub struct $resolved { $(pub $field: ResolvedMetric,)+ }
        #[cfg(test)]
        impl $stylesheet {
            fn validate_contract() {
                $(for value in ["-1", "nan", "inf", "1e100", "\"4px\""] {
                    assert!(crate::theme::stylesheet::CommonStylesheet::parse(&format!(
                        "[common.{}.sizes.\"custom\"]\n{} = {value}", $family, stringify!($field))).is_err());
                })+
                let authored = $authored { $($field: Some(0.0),)+ };
                let config = $stylesheet { geometry: $authored { $($field: Some(9.0),)+ }, sizes: HashMap::from([("4".into(), authored)]) };
                let selected = config.resolve_geometry("4", $fallback { $($field: 7.0,)+ });
                let general = config.resolve_geometry("other", $fallback { $($field: 7.0,)+ });
                let missing = Self::default().resolve_geometry("4", $fallback { $($field: 7.0,)+ });
                $(assert_eq!(selected.$field.value_px, 0.0);
                assert_eq!(general.$field.value_px, 9.0);
                assert_eq!(missing.$field.value_px, 7.0);
                assert!(matches!(selected.$field.source, crate::theme::provenance::MetricSource::Authored { key }
                    if key == format!("common.{}.sizes.4.{}", $family, stringify!($field))));)+
                assert!(crate::theme::stylesheet::CommonStylesheet::parse(&format!("[common.{}.geometry]\nunknown = 1", $family)).is_err());
            }
        }
        impl $stylesheet {
            /// Per-size values override general geometry, then supplied fallbacks.
            pub fn resolve_geometry(&self, key: &str, fallback: $fallback) -> $resolved {
                let specific = self.sizes.get(key);
                $resolved {
                    $($field: if let Some(value) = specific.and_then(|size| size.$field) {
                        ResolvedMetric::authored(value, format!("common.{}.sizes.{key}.{}", $family, stringify!($field)))
                    } else if let Some(value) = self.geometry.$field {
                        ResolvedMetric::authored(value, concat!("common.", $family, ".geometry.", stringify!($field)))
                    } else {
                        ResolvedMetric::constant(fallback.$field, concat!($family, " ", stringify!($field), " fallback"))
                    },)+
                }
            }
        }
    };
}
geometry_family!(
    ButtonStylesheet,
    ButtonGeometryStylesheet,
    ButtonGeometry,
    ResolvedButtonGeometry,
    "button",
    [height, padding_x, padding_y, gap, icon_size, font_size, line_height]
);
geometry_family!(
    CheckboxStylesheet,
    CheckboxGeometryStylesheet,
    CheckboxGeometry,
    ResolvedCheckboxGeometry,
    "checkbox",
    [indicator_size, glyph_size, gap]
);
geometry_family!(
    RadioStylesheet,
    RadioGeometryStylesheet,
    RadioGeometry,
    ResolvedRadioGeometry,
    "radio",
    [indicator_size, dot_size, gap]
);
geometry_family!(
    TextFieldStylesheet,
    TextFieldGeometryStylesheet,
    TextFieldGeometry,
    ResolvedTextFieldGeometry,
    "textfield",
    [min_height, padding_x, padding_y, gap, icon_size, font_size, line_height]
);
geometry_family!(
    TextAreaStylesheet,
    TextAreaGeometryStylesheet,
    TextAreaGeometry,
    ResolvedTextAreaGeometry,
    "textarea",
    [min_height, padding_x, padding_y, font_size, line_height]
);
geometry_family!(
    ProgressStylesheet,
    ProgressGeometryStylesheet,
    ProgressGeometry,
    ResolvedProgressGeometry,
    "progress",
    [track_height, size, stroke_width, thumb_size]
);
geometry_family!(
    FloatingMenuStylesheet,
    FloatingMenuGeometryStylesheet,
    FloatingMenuGeometry,
    ResolvedFloatingMenuGeometry,
    "floating_menu",
    [
        padding,
        min_width,
        item_height,
        item_padding_x,
        item_gap,
        item_icon_size,
        font_size,
        line_height,
        submenu_offset_x
    ]
);
geometry_family!(
    TooltipStylesheet,
    TooltipGeometryStylesheet,
    TooltipGeometry,
    ResolvedTooltipGeometry,
    "tooltip",
    [padding, padding_y, max_width, font_size, line_height]
);
geometry_family!(
    ToolbarStylesheet,
    ToolbarGeometryStylesheet,
    ToolbarGeometry,
    ResolvedToolbarGeometry,
    "toolbar",
    [padding_x, padding_y, gap]
);
geometry_family!(
    SegmentedStylesheet,
    SegmentedGeometryStylesheet,
    SegmentedGeometry,
    ResolvedSegmentedGeometry,
    "segmented",
    [height, padding_x, font_size, line_height]
);
geometry_family!(
    AvatarStylesheet,
    AvatarGeometryStylesheet,
    AvatarGeometry,
    ResolvedAvatarGeometry,
    "avatar",
    [diameter, icon_size, font_size, two_letter_font_size]
);
geometry_family!(
    BadgeStylesheet,
    BadgeGeometryStylesheet,
    BadgeGeometry,
    ResolvedBadgeGeometry,
    "badge",
    [padding_x, padding_y, font_size, line_height, min_height, gap, icon_size]
);
geometry_family!(
    CardStylesheet,
    CardGeometryStylesheet,
    CardGeometry,
    ResolvedCardGeometry,
    "card",
    [padding, section_gap, header_gap, body_gap]
);
geometry_family!(
    CalloutStylesheet,
    CalloutGeometryStylesheet,
    CalloutGeometry,
    ResolvedCalloutGeometry,
    "callout",
    [padding, gap, icon_size, font_size]
);

geometry_family!(
    OverlayWindowStylesheet,
    OverlayWindowGeometryStylesheet,
    OverlayWindowGeometry,
    ResolvedOverlayWindowGeometry,
    "overlay_window",
    [padding, min_width, max_width, estimated_height, font_size, line_height]
);

geometry_family!(
    PopupMenuStylesheet,
    PopupMenuGeometryStylesheet,
    PopupMenuGeometry,
    ResolvedPopupMenuGeometry,
    "popup_menu",
    [height, padding_x, padding_y, gap, font_size, line_height, icon_size]
);
geometry_family!(
    ContextMenuStylesheet,
    ContextMenuGeometryStylesheet,
    ContextMenuGeometry,
    ResolvedContextMenuGeometry,
    "context_menu",
    [min_width, padding_x, padding_y, font_size, line_height]
);

geometry_family!(
    ToggleStylesheet,
    ToggleGeometryStylesheet,
    ToggleGeometry,
    ResolvedToggleGeometry,
    "toggle",
    [height, padding_x, padding_y, gap, icon_size, font_size, line_height]
);

geometry_family!(
    ScrollbarStylesheet,
    ScrollbarGeometryStylesheet,
    ScrollbarGeometry,
    ResolvedScrollbarGeometry,
    "scrollbar",
    [thickness, track_thickness, thumb_thickness, min_thumb_length, length_h, length_v]
);

geometry_family!(
    StepperStylesheet,
    StepperGeometryStylesheet,
    StepperGeometry,
    ResolvedStepperGeometry,
    "stepper",
    [step_badge_size, track_thickness]
);

geometry_family!(
    SidebarStylesheet,
    SidebarGeometryStylesheet,
    SidebarGeometry,
    ResolvedSidebarGeometry,
    "sidebar",
    [section_height, item_height, item_padding_x, item_gap, item_icon_size]
);

geometry_family!(
    ControlGroupStylesheet,
    ControlGroupGeometryStylesheet,
    ControlGroupGeometry,
    ResolvedControlGroupGeometry,
    "control_group",
    [padding_x, padding_y, gap]
);

geometry_family!(
    TableStylesheet,
    TableGeometryStylesheet,
    TableGeometry,
    ResolvedTableGeometry,
    "table",
    [padding_x, padding_y]
);

geometry_family!(
    PagerStylesheet,
    PagerGeometryStylesheet,
    PagerGeometry,
    ResolvedPagerGeometry,
    "pager",
    [
        button_size,
        button_min_width,
        control_height,
        padding_x,
        padding_y,
        gap,
        group_gap,
        font_size,
        line_height
    ]
);

geometry_family!(
    SelectorStylesheet,
    SelectorGeometryStylesheet,
    SelectorGeometry,
    ResolvedSelectorGeometry,
    "selector",
    [height, padding_x, padding_y, gap, icon_size, font_size, line_height]
);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_family_contracts_validate_every_field_and_preserve_precedence() {
        ScrollbarStylesheet::validate_contract();
        StepperStylesheet::validate_contract();
        SidebarStylesheet::validate_contract();
        ControlGroupStylesheet::validate_contract();
        TableStylesheet::validate_contract();
        PagerStylesheet::validate_contract();
        SelectorStylesheet::validate_contract();
        ButtonStylesheet::validate_contract();
        ToggleStylesheet::validate_contract();
        CheckboxStylesheet::validate_contract();
        RadioStylesheet::validate_contract();
        TextFieldStylesheet::validate_contract();
        TextAreaStylesheet::validate_contract();
        ProgressStylesheet::validate_contract();
        FloatingMenuStylesheet::validate_contract();
        TooltipStylesheet::validate_contract();
        ToolbarStylesheet::validate_contract();
        SegmentedStylesheet::validate_contract();
        AvatarStylesheet::validate_contract();
        BadgeStylesheet::validate_contract();
        CardStylesheet::validate_contract();
        CalloutStylesheet::validate_contract();
        OverlayWindowStylesheet::validate_contract();
        PopupMenuStylesheet::validate_contract();
        ContextMenuStylesheet::validate_contract();
    }
}
