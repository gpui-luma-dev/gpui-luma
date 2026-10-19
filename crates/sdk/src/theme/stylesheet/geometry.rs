//! Typed common dimensions for remaining control families.
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::{optional_length, optional_unit};
use crate::theme::provenance::ResolvedMetric;

// Adapt an existing resolved theme value without duplicating its token-derived
// fallback construction. Only field mappings vary between control families.
macro_rules! apply_common_geometry {
    ($sheet:expr, $key:expr, $fallback:ident, $look:ident, [$($field:ident => $($target:ident).+),+ $(,)?] $(, defaults = $defaults:expr)?) => {{
        let geometry = $sheet.resolve_geometry($key, $crate::theme::stylesheet::$fallback {
            $($field: $look.$($target).+,)+
            $(..$defaults)?
        });
        $($look.$($target).+ = geometry.$field.value_px;)+
    }};
}
pub(crate) use apply_common_geometry;

macro_rules! inherit_geometry {
    ($stylesheet:ident, $axis:ident, [$($field:ident),+]) => {
        impl $stylesheet {
            pub(super) fn with_baseline(mut self, baseline: &Self) -> Self {
                let mut inherited = baseline.$axis.clone();
                for value in inherited.values_mut() {
                    $(if self.geometry.$field.is_some() { value.$field = None; })+
                }
                for (key, authored) in self.$axis {
                    let value = inherited.entry(key).or_default();
                    $(value.$field = authored.$field.or(value.$field);)+
                }
                $(self.geometry.$field = self.geometry.$field.or(baseline.geometry.$field);)+
                self.$axis = inherited;
                self
            }
        }
    };
}
pub(super) use inherit_geometry;

macro_rules! geometry_family {
    ($stylesheet:ident, $authored:ident, $fallback:ident, $resolved:ident, $family:literal, [$($field:ident),*] $(, unit [$($unit:ident),+])?) => {
        geometry_family_impl!($stylesheet, $authored, $fallback, $resolved, $family,
            [$(($field, "optional_length"),)* $($(($unit, "optional_unit"),)+)?]);
    };
}

macro_rules! geometry_family_impl {
    ($stylesheet:ident, $authored:ident, $fallback:ident, $resolved:ident, $family:literal, [$(($field:ident, $validator:literal)),+ ,]) => {
        /// Common template dimensions, with semantic size keys supplied by each look.
        #[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct $stylesheet {
            pub geometry: $authored,
            pub sizes: HashMap<String, $authored>,
        }
        /// Optional logical pixel values. Authored zero overrides the fallback.
        #[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct $authored {
            $(#[serde(deserialize_with = $validator)] pub $field: Option<f32>,)+
        }
        impl $authored {
            /// Fill missing fields without replacing explicit zero values.
            pub fn with_fallback(self, fallback: &Self) -> Self {
                Self { $($field: self.$field.or(fallback.$field),)+ }
            }
        }
        /// Dimensions supplied by the SDK or look tokens when configuration is unset.
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $fallback { $(pub $field: f32,)+ }
        /// Resolved common dimensions with the source of each value.
        #[derive(Clone, Debug)]
        pub struct $resolved { $(pub $field: ResolvedMetric,)+ }
        inherit_geometry!($stylesheet, sizes, [$($field),+]);
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
            /// Preserve optional derived inputs while resolving size over general fields.
            pub fn authored_geometry(&self, key: &str) -> $authored {
                self.sizes.get(key).cloned().unwrap_or_default().with_fallback(&self.geometry)
            }
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
    ShellStylesheet,
    ShellGeometryStylesheet,
    ShellGeometry,
    ResolvedShellGeometry,
    "shell",
    [
        height,
        macos_left_padding,
        left_padding,
        traffic_light_inset,
        icon_size,
        border_width,
        fullscreen_left_padding
    ]
);
geometry_family!(
    SharedStylesheet,
    SharedGeometryStylesheet,
    SharedGeometry,
    ResolvedSharedGeometry,
    "shared",
    [
        popup_viewport_height,
        popup_row_height,
        menu_row_height,
        menu_row_gap,
        compact_textfield_height,
        compact_textfield_padding_y,
        textarea_resize_icon_size,
        textarea_resize_inset,
        textarea_resize_hit_size,
        textarea_scrollbar_width,
        textarea_scrollbar_inset,
        scrollbar_width,
        composition_underline_width,
        label_baseline_sm,
        label_baseline_md,
        label_baseline_lg,
        drag_offset,
        drag_preview_width,
        drag_padding_x,
        drag_padding_y,
        drag_border_width,
        drag_marker_width,
        split_sidebar_width,
        split_sidebar_min_width,
        split_sidebar_max_width,
        selection_scrollbar_width,
        wide_middle_gap,
        wide_side_min_width,
        wide_side_preferred_width,
        wide_middle_min_width,
        wide_middle_max_width,
        form_field_gap,
        page_size_trigger_width,
        popup_icon_size
    ],
    unit[textarea_resize_opacity]
);
geometry_family!(
    TreeViewStylesheet,
    TreeViewGeometryStylesheet,
    TreeViewGeometry,
    ResolvedTreeViewGeometry,
    "tree_view",
    [
        row_height,
        row_height_scale,
        base_padding_x,
        indentation_width,
        inner_gap,
        radius,
        icon_size,
        chevron_size,
        chevron_inset,
        preview_padding_x,
        preview_padding_y, drag_marker_width, drag_insert_width, scrollbar_width
    ],
    unit[disabled_opacity, drag_opacity]
);
geometry_family!(
    AccordionStylesheet,
    AccordionGeometryStylesheet,
    AccordionGeometry,
    ResolvedAccordionGeometry,
    "accordion",
    [
        trigger_height,
        padding_x,
        padding_y,
        content_padding_y,
        content_padding_scale,
        radius,
        item_gap,
        inner_gap,
        icon_size,
        chevron_size,
        drag_handle_padding
    ],
    unit[disabled_opacity]
);
geometry_family!(
    SplitViewStylesheet,
    SplitViewGeometryStylesheet,
    SplitViewGeometry,
    ResolvedSplitViewGeometry,
    "split_view",
    [hitbox_width, expand_inset_y, expand_cue_left, cue_inset_y, cue_radius, cue_width, cue_hovered_width]
);
geometry_family!(
    DockSplitterStylesheet,
    DockSplitterGeometryStylesheet,
    DockSplitterGeometry,
    ResolvedDockSplitterGeometry,
    "dock_splitter",
    [hit_target, visible_line, thumb_width, thumb_height, thumb_radius],
    unit[thumb_rest_opacity]
);
geometry_family!(
    ResizablePanelsStylesheet,
    ResizablePanelsGeometryStylesheet,
    ResizablePanelsGeometry,
    ResolvedResizablePanelsGeometry,
    "resizable_panels",
    [lane_width, hit_target, grip_cross_axis, grip_main_axis, grip_radius, border_width, divider_width],
    unit[disabled_opacity]
);
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
    [
        indicator_size,
        indicator_scale,
        indicator_radius,
        glyph_size,
        glyph_inset,
        control_padding_x,
        control_padding_y,
        gap
    ],
    unit[disabled_opacity]
);
geometry_family!(
    RadioStylesheet,
    RadioGeometryStylesheet,
    RadioGeometry,
    ResolvedRadioGeometry,
    "radio",
    [indicator_size, indicator_scale, dot_size, dot_scale, control_padding_x, control_padding_y, gap],
    unit[disabled_opacity]
);
geometry_family!(
    TextFieldStylesheet,
    TextFieldGeometryStylesheet,
    TextFieldGeometry,
    ResolvedTextFieldGeometry,
    "textfield",
    [
        min_height,
        padding_x,
        padding_y,
        gap,
        icon_size,
        font_size,
        line_height,
        caret_width,
        caret_height_extra
    ],
    unit[disabled_opacity]
);
geometry_family!(
    TextAreaStylesheet,
    TextAreaGeometryStylesheet,
    TextAreaGeometry,
    ResolvedTextAreaGeometry,
    "textarea",
    [min_height, padding_x, padding_y, font_size, line_height, caret_width, caret_height_extra],
    unit[disabled_opacity]
);
geometry_family!(
    ProgressStylesheet,
    ProgressGeometryStylesheet,
    ProgressGeometry,
    ResolvedProgressGeometry,
    "progress",
    [track_height, size, stroke_width, thumb_size],
    unit[indeterminate_fraction, shimmer_soft_opacity, shimmer_mid_opacity, shimmer_mid_offset, shimmer_mid_width, shimmer_core_offset, shimmer_core_width]
);
geometry_family!(
    FloatingMenuStylesheet,
    FloatingMenuGeometryStylesheet,
    FloatingMenuGeometry,
    ResolvedFloatingMenuGeometry,
    "floating_menu",
    [
        border_width,
        padding,
        min_width,
        item_height,
        item_padding_x,
        item_gap,
        item_icon_size,
        font_size,
        line_height,
        submenu_offset_x,
        padding_scale,
        item_height_scale,
        item_padding_scale,
        submenu_offset_scale,
        separator_thickness,
        separator_spacing,
        separator_inset
    ],
    unit[disabled_opacity]
);
geometry_family!(
    SelectionPanelStylesheet,
    SelectionPanelGeometryStylesheet,
    SelectionPanelGeometry,
    ResolvedSelectionPanelGeometry,
    "selection_panel",
    [
        padding,
        min_width,
        item_height,
        item_padding_x,
        item_gap,
        item_icon_size,
        font_size,
        line_height,
        border_width,
        radius,
        item_radius
    ],
    unit[disabled_opacity]
);
geometry_family!(
    TooltipStylesheet,
    TooltipGeometryStylesheet,
    TooltipGeometry,
    ResolvedTooltipGeometry,
    "tooltip",
    [
        target_gap,
        window_margin,
        pointer_size,
        pointer_inset,
        padding,
        padding_y,
        max_width,
        font_size,
        line_height,
        shortcut_gap,
        shortcut_font_size
    ],
    unit[shortcut_opacity]
);
geometry_family!(
    ToolbarStylesheet,
    ToolbarGeometryStylesheet,
    ToolbarGeometry,
    ResolvedToolbarGeometry,
    "toolbar",
    [padding_x, padding_y, gap, border_width, separator_width]
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
    [
        viewport_padding,
        window_margin,
        drag_header_height,
        padding,
        min_width,
        max_width,
        estimated_height,
        font_size,
        line_height,
        border_width,
        radius
    ],
    unit[backdrop_opacity]
);

geometry_family!(
    PopupMenuStylesheet,
    PopupMenuGeometryStylesheet,
    PopupMenuGeometry,
    ResolvedPopupMenuGeometry,
    "popup_menu",
    [
        height,
        padding_x,
        padding_y,
        gap,
        font_size,
        line_height,
        icon_size,
        radius,
        menu_offset_y,
        menu_offset_scale
    ]
);
geometry_family!(
    ContextMenuStylesheet,
    ContextMenuGeometryStylesheet,
    ContextMenuGeometry,
    ResolvedContextMenuGeometry,
    "context_menu",
    [min_width, padding_x, padding_y, font_size, line_height, border_width, window_margin],
    unit[disabled_opacity]
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
    [thickness, track_thickness, thumb_thickness, min_thumb_length, length_h, length_v],
    unit[disabled_opacity, thumb_fraction]
);

geometry_family!(
    StepperStylesheet,
    StepperGeometryStylesheet,
    StepperGeometry,
    ResolvedStepperGeometry,
    "stepper",
    [
        step_badge_size,
        track_thickness,
        side_label_width,
        label_width,
        content_gap,
        label_gap,
        side_label_gap,
        content_height,
        font_size,
        line_height,
        glyph_scale,
        number_scale,
        border_width
    ]
);

geometry_family!(
    SidebarStylesheet,
    SidebarGeometryStylesheet,
    SidebarGeometry,
    ResolvedSidebarGeometry,
    "sidebar",
    [
        section_height,
        item_height,
        item_padding_x,
        item_gap,
        item_icon_size,
        width_expanded,
        width_icon_rail,
        width_mobile,
        rail_hit_width,
        popover_offset,
        container_gap,
        region_gap,
        header_padding_bottom,
        footer_padding_top,
        title_gap,
        title_padding_bottom,
        section_padding_top,
        disclosure_icon_size,
        rail_submenu_offset_x,
        window_margin,
        engine_item_height,
        engine_section_height,
        engine_item_padding_x, rail_branch_indicator_size, rail_branch_overhang
    ],
    unit[subtitle_opacity, disabled_opacity]
);

geometry_family!(
    ControlGroupStylesheet,
    ControlGroupGeometryStylesheet,
    ControlGroupGeometry,
    ResolvedControlGroupGeometry,
    "control_group",
    [padding_x, padding_y, gap, radius, vertical_gap, border_width],
    unit[disabled_opacity]
);

geometry_family!(
    TableStylesheet,
    TableGeometryStylesheet,
    TableGeometry,
    ResolvedTableGeometry,
    "table",
    [
        row_divider_width,
        padding_x,
        padding_y,
        padding_y_scale,
        row_radius,
        border_width,
        header_divider_width,
        header_padding_bottom_scale,
        drag_offset,
        drag_width,
        drag_padding_x,
        drag_padding_y,
        drag_border_width,
        drag_marker_width,
        column_padding_x,
        column_resize_hit_width,
        column_resize_line_width,
        column_resize_vertical_extension
    ],
    unit[row_hover_opacity, row_disabled_opacity, column_muted_opacity]
);

geometry_family!(
    PagerStylesheet,
    PagerGeometryStylesheet,
    PagerGeometry,
    ResolvedPagerGeometry,
    "pager",
    [
        gap_icon_size,
        button_size,
        button_min_width,
        control_height,
        padding_x,
        padding_y,
        gap,
        group_gap,
        font_size,
        line_height,
        radius,
        padding_x_inset_scale,
        padding_y_gap_scale,
        button_extra_width_scale,
        border_width,
        page_size_label_gap,
        popup_offset_y,
        popup_gap,
        popup_padding,
        popup_item_padding_y,
        popup_item_radius_inset
    ],
    unit[disabled_opacity]
);

geometry_family!(
    SelectorStylesheet,
    SelectorGeometryStylesheet,
    SelectorGeometry,
    ResolvedSelectorGeometry,
    "selector",
    [
        height,
        padding_x,
        padding_y,
        gap,
        icon_size,
        font_size,
        line_height,
        radius,
        menu_offset_y,
        menu_offset_scale
    ]
);

geometry_family!(
    SelectorListStylesheet,
    SelectorListGeometryStylesheet,
    SelectorListGeometry,
    ResolvedSelectorListGeometry,
    "selector_list",
    [
        padding,
        min_width,
        item_height,
        item_padding_x,
        item_gap,
        item_icon_size,
        font_size,
        line_height,
        radius,
        item_radius,
        border_width,
        popup_offset_y,
        window_margin
    ],
    unit[disabled_opacity]
);

geometry_family!(
    PopoverStylesheet,
    PopoverGeometryStylesheet,
    PopoverGeometry,
    ResolvedPopoverGeometry,
    "popover",
    [offset_y, window_margin]
);
geometry_family!(
    SlidePanelStylesheet,
    SlidePanelGeometryStylesheet,
    SlidePanelGeometry,
    ResolvedSlidePanelGeometry,
    "slide_panel",
    [side_width, edge_height],
    unit[edge_max_viewport_ratio]
);

geometry_family!(
    RadioGroupStylesheet,
    RadioGroupGeometryStylesheet,
    RadioGroupGeometry,
    ResolvedRadioGroupGeometry,
    "radio_group",
    [gap, item_gap],
    unit[disabled_opacity]
);
geometry_family!(
    IconGroupStylesheet,
    IconGroupGeometryStylesheet,
    IconGroupGeometry,
    ResolvedIconGroupGeometry,
    "icon_group",
    [padding_x, padding_y, gap, radius]
);

geometry_family!(
    AutocompleteStylesheet,
    AutocompleteGeometryStylesheet,
    AutocompleteGeometry,
    ResolvedAutocompleteGeometry,
    "autocomplete",
    [
        gap,
        status_gap,
        clear_inset,
        clear_width,
        clear_icon_size,
        status_font_size,
        status_line_height,
        trigger_width,
        popup_extra_width
    ]
);

geometry_family!(
    ComboBoxStylesheet,
    ComboBoxGeometryStylesheet,
    ComboBoxGeometry,
    ResolvedComboBoxGeometry,
    "combobox",
    [affordance_inset, affordance_width, affordance_icon_size, affordance_gap, trigger_extra_gap]
);

geometry_family!(
    SearchSelectorStylesheet,
    SearchSelectorGeometryStylesheet,
    SearchSelectorGeometry,
    ResolvedSearchSelectorGeometry,
    "search_selector",
    [
        border_width,
        search_icon_width,
        search_icon_size,
        trigger_icon_extra_width,
        trigger_extra_gap,
        search_padding,
        search_separator_height,
        search_padding_reduction,
        search_input_height_floor,
        search_chrome_height,
        trailing_icon_size
    ]
);

impl TableStylesheet {
    /// Fills absent table fields from SDK defaults without copying unrelated families.
    pub fn with_sdk_defaults(&self) -> Self {
        self.clone().with_baseline(&super::CommonStylesheet::sdk_default().table)
    }
}

impl RadioStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: RadioGeometryStylesheet {
                disabled_opacity: Some(0.56),
                control_padding_x: Some(0.0),
                control_padding_y: Some(0.0),
                ..Default::default()
            },
            sizes: [("sm", 0.45), ("md", 0.5), ("lg", 0.55)]
                .into_iter()
                .map(|(key, scale)| {
                    (
                        key.into(),
                        RadioGeometryStylesheet {
                            indicator_scale: Some(scale),
                            dot_scale: Some(0.24),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        }
    }
}

impl ProgressStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ProgressGeometryStylesheet {
                indeterminate_fraction: Some(0.28),
                shimmer_soft_opacity: Some(0.35),
                shimmer_mid_opacity: Some(0.85),
                shimmer_mid_offset: Some(0.2),
                shimmer_mid_width: Some(0.6),
                shimmer_core_offset: Some(0.35),
                shimmer_core_width: Some(0.3),
                ..Default::default()
            },
            sizes: [("sm", 4.0, 48.0, 5.0, 12.0), ("md", 6.0, 64.0, 6.0, 16.0), ("lg", 8.0, 80.0, 8.0, 20.0)]
                .into_iter()
                .map(|(key, track, size, stroke, thumb)| {
                    (
                        key.into(),
                        ProgressGeometryStylesheet {
                            track_height: Some(track),
                            size: Some(size),
                            stroke_width: Some(stroke),
                            thumb_size: Some(thumb),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        }
    }
}

impl StepperStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: StepperGeometryStylesheet {
                side_label_width: Some(120.0),
                label_width: Some(48.0),
                content_gap: Some(16.0),
                label_gap: Some(6.0),
                side_label_gap: Some(10.0),
                content_height: Some(120.0),
                font_size: Some(12.0),
                line_height: Some(16.0),
                glyph_scale: Some(0.45),
                number_scale: Some(0.38),
                border_width: Some(1.0),
                ..Default::default()
            },
            sizes: [("sm", 24.0, 2.0), ("md", 32.0, 2.0), ("lg", 40.0, 3.0)]
                .into_iter()
                .map(|(key, badge, track)| {
                    (
                        key.into(),
                        StepperGeometryStylesheet {
                            step_badge_size: Some(badge),
                            track_thickness: Some(track),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        }
    }
}

impl ScrollbarStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ScrollbarGeometryStylesheet {
                thumb_fraction: Some(0.25),
                disabled_opacity: Some(0.56),
                ..Default::default()
            },
            sizes: [("sm", 10.0, 3.0, 6.0, 24.0), ("md", 12.0, 4.0, 8.0, 28.0), ("lg", 14.0, 5.0, 10.0, 32.0)]
                .into_iter()
                .map(|(key, thickness, track, thumb, min)| {
                    (
                        key.into(),
                        ScrollbarGeometryStylesheet {
                            thickness: Some(thickness),
                            track_thickness: Some(track),
                            thumb_thickness: Some(thumb),
                            min_thumb_length: Some(min),
                            length_h: Some(260.0),
                            length_v: Some(180.0),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        }
    }
}

impl TooltipStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: TooltipGeometryStylesheet {
                target_gap: Some(10.0),
                window_margin: Some(8.0),
                pointer_size: Some(6.0),
                pointer_inset: Some(12.0),
                shortcut_gap: Some(16.0),
                shortcut_font_size: Some(12.0),
                shortcut_opacity: Some(0.65),
                padding: Some(12.0),
                padding_y: Some(8.0),
                max_width: Some(260.0),
                font_size: Some(14.0),
                line_height: Some(20.0),
            },
            sizes: HashMap::from([(
                "compact".into(),
                TooltipGeometryStylesheet {
                    padding: Some(8.0),
                    padding_y: Some(4.0),
                    font_size: Some(12.0),
                    line_height: Some(16.0),
                    ..Default::default()
                },
            )]),
        }
    }
}

impl FloatingMenuStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: FloatingMenuGeometryStylesheet {
                min_width: Some(180.0),
                padding_scale: Some(0.5),
                item_height_scale: Some(0.9),
                item_padding_scale: Some(0.75),
                submenu_offset_scale: Some(0.5),
                disabled_opacity: Some(0.56),
                separator_thickness: Some(1.0),
                separator_inset: Some(0.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl ContextMenuStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ContextMenuGeometryStylesheet {
                border_width: Some(1.0),
                window_margin: Some(8.0),
                disabled_opacity: Some(0.56),
                min_width: Some(200.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl ControlGroupStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ControlGroupGeometryStylesheet {
                vertical_gap: Some(8.0),
                border_width: Some(1.0),
                disabled_opacity: Some(0.56),
                padding_x: Some(6.0),
                padding_y: Some(4.0),
                gap: Some(6.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl DockSplitterStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: DockSplitterGeometryStylesheet {
                thumb_width: Some(4.0),
                thumb_height: Some(36.0),
                thumb_radius: Some(8.0),
                thumb_rest_opacity: Some(0.4),
                hit_target: Some(8.0),
                visible_line: Some(1.0),
            },
            ..Default::default()
        }
    }
}

impl ResizablePanelsStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ResizablePanelsGeometryStylesheet {
                border_width: Some(1.0),
                divider_width: Some(1.0),
                grip_radius: Some(8.0),
                disabled_opacity: Some(0.45),
                ..Default::default()
            },
            sizes: [("sm", 8.0, 12.0, 4.0, 36.0), ("md", 12.0, 14.0, 6.0, 40.0), ("lg", 18.0, 20.0, 8.0, 44.0)]
                .into_iter()
                .map(|(key, lane, hit, cross, main)| {
                    (
                        key.into(),
                        ResizablePanelsGeometryStylesheet {
                            lane_width: Some(lane),
                            hit_target: Some(hit),
                            grip_cross_axis: Some(cross),
                            grip_main_axis: Some(main),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        }
    }
}

impl OverlayWindowStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: OverlayWindowGeometryStylesheet {
                viewport_padding: Some(24.0),
                window_margin: Some(12.0),
                drag_header_height: Some(56.0),
                border_width: Some(1.0),
                backdrop_opacity: Some(0.44),
                ..Default::default()
            },
            sizes: [("sm", 320.0, 440.0, 196.0), ("md", 400.0, 520.0, 228.0), ("lg", 480.0, 600.0, 264.0)]
                .into_iter()
                .map(|(key, min, max, height)| {
                    (
                        key.into(),
                        OverlayWindowGeometryStylesheet {
                            min_width: Some(min),
                            max_width: Some(max),
                            estimated_height: Some(height),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        }
    }
}

impl SidebarStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: SidebarGeometryStylesheet {
                rail_branch_indicator_size: Some(18.0),
                rail_branch_overhang: Some(10.0),
                container_gap: Some(8.0),
                region_gap: Some(4.0),
                header_padding_bottom: Some(8.0),
                footer_padding_top: Some(8.0),
                title_gap: Some(2.0),
                title_padding_bottom: Some(4.0),
                section_padding_top: Some(8.0),
                disclosure_icon_size: Some(14.0),
                rail_submenu_offset_x: Some(12.0),
                window_margin: Some(8.0),
                engine_item_height: Some(30.0),
                engine_section_height: Some(20.0),
                engine_item_padding_x: Some(8.0),
                subtitle_opacity: Some(0.72),
                disabled_opacity: Some(0.56),
                width_expanded: Some(256.0),
                width_icon_rail: Some(48.0),
                width_mobile: Some(288.0),
                item_height: Some(32.0),
                item_icon_size: Some(16.0),
                rail_hit_width: Some(6.0),
                popover_offset: Some(8.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl TreeViewStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: TreeViewGeometryStylesheet {
                drag_marker_width: Some(1.0),
                drag_insert_width: Some(2.0),
                drag_opacity: Some(0.12),
                scrollbar_width: Some(12.0),
                disabled_opacity: Some(0.4),
                preview_padding_x: Some(12.0),
                preview_padding_y: Some(4.0),
                row_height_scale: Some(0.85),
                indentation_width: Some(16.0),
                chevron_inset: Some(2.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl AccordionStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: AccordionGeometryStylesheet {
                drag_handle_padding: Some(2.0),
                disabled_opacity: Some(0.56),
                content_padding_scale: Some(1.5),
                item_gap: Some(0.0),
                icon_size: Some(16.0),
                chevron_size: Some(14.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl TableStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: TableGeometryStylesheet {
                row_divider_width: Some(1.0),
                column_padding_x: Some(12.0),
                column_muted_opacity: Some(0.65),
                column_resize_hit_width: Some(10.0),
                column_resize_line_width: Some(1.0),
                column_resize_vertical_extension: Some(4.0),
                drag_offset: Some(12.0),
                drag_width: Some(220.0),
                drag_padding_x: Some(12.0),
                drag_padding_y: Some(8.0),
                drag_border_width: Some(1.0),
                drag_marker_width: Some(2.0),
                border_width: Some(1.0),
                header_divider_width: Some(1.0),
                header_padding_bottom_scale: Some(3.0 / 4.0),
                padding_x: Some(0.0),
                padding_y_scale: Some(1.0 / 2.0),
                row_radius: Some(0.0),
                row_hover_opacity: Some(0.4),
                row_disabled_opacity: Some(0.56),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl SelectorStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: SelectorGeometryStylesheet { menu_offset_scale: Some(1.0 / 2.0), ..Default::default() },
            ..Default::default()
        }
    }
}

impl PagerStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: PagerGeometryStylesheet {
                gap_icon_size: Some(16.0),
                border_width: Some(1.0),
                page_size_label_gap: Some(8.0),
                popup_offset_y: Some(4.0),
                popup_gap: Some(2.0),
                popup_padding: Some(4.0),
                popup_item_padding_y: Some(4.0),
                popup_item_radius_inset: Some(2.0),
                padding_x_inset_scale: Some(1.0 / 2.0),
                padding_y_gap_scale: Some(1.0 / 2.0),
                button_extra_width_scale: Some(1.0),
                disabled_opacity: Some(0.56),
                ..Default::default()
            },
            sizes: HashMap::from([(
                "minimal".into(),
                PagerGeometryStylesheet { button_extra_width_scale: Some(0.0), ..Default::default() },
            )]),
        }
    }
}

impl PopupMenuStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: PopupMenuGeometryStylesheet { menu_offset_scale: Some(1.0 / 2.0), ..Default::default() },
            ..Default::default()
        }
    }
}

impl PopoverStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: PopoverGeometryStylesheet { window_margin: Some(8.0), ..Default::default() },
            sizes: [("button", 0.0), ("panel", 4.0)]
                .into_iter()
                .map(|(key, offset)| {
                    (key.into(), PopoverGeometryStylesheet { offset_y: Some(offset), ..Default::default() })
                })
                .collect(),
        }
    }
}

impl SlidePanelStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: SlidePanelGeometryStylesheet {
                side_width: Some(360.0),
                edge_height: Some(420.0),
                edge_max_viewport_ratio: Some(0.72),
            },
            ..Default::default()
        }
    }
}

impl RadioGroupStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: RadioGroupGeometryStylesheet {
                item_gap: Some(8.0),
                disabled_opacity: Some(0.56),
                ..Default::default()
            },
            sizes: [("vertical", 8.0), ("horizontal", 12.0)]
                .into_iter()
                .map(|(key, gap)| (key.into(), RadioGroupGeometryStylesheet { gap: Some(gap), ..Default::default() }))
                .collect(),
        }
    }
}

impl AutocompleteStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: AutocompleteGeometryStylesheet {
                gap: Some(10.0),
                status_gap: Some(10.0),
                clear_inset: Some(10.0),
                clear_width: Some(18.0),
                clear_icon_size: Some(12.0),
                trigger_width: Some(240.0),
                popup_extra_width: Some(24.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl ComboBoxStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ComboBoxGeometryStylesheet {
                affordance_inset: Some(10.0),
                affordance_width: Some(18.0),
                affordance_icon_size: Some(12.0),
                affordance_gap: Some(2.0),
                trigger_extra_gap: Some(8.0),
            },
            ..Default::default()
        }
    }
}

impl SearchSelectorStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: SearchSelectorGeometryStylesheet {
                border_width: Some(1.0),
                search_icon_width: Some(18.0),
                search_icon_size: Some(12.0),
                trigger_icon_extra_width: Some(10.0),
                trigger_extra_gap: Some(8.0),
                search_padding: Some(8.0),
                search_separator_height: Some(1.0),
                search_padding_reduction: Some(3.0),
                search_input_height_floor: Some(18.0),
                search_chrome_height: Some(57.0),
                trailing_icon_size: Some(14.0),
            },
            ..Default::default()
        }
    }
}

impl TextFieldStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: TextFieldGeometryStylesheet {
                caret_width: Some(1.5),
                caret_height_extra: Some(2.0),
                disabled_opacity: Some(0.6),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl TextAreaStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: TextAreaGeometryStylesheet {
                caret_width: Some(1.5),
                caret_height_extra: Some(2.0),
                disabled_opacity: Some(0.6),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl SplitViewStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: SplitViewGeometryStylesheet {
                hitbox_width: Some(20.0),
                expand_inset_y: Some(10.0),
                expand_cue_left: Some(2.0),
                cue_inset_y: Some(16.0),
                cue_radius: Some(4.0),
                cue_width: Some(4.0),
                cue_hovered_width: Some(8.0),
            },
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_family_contracts_validate_every_field_and_preserve_precedence() {
        ScrollbarStylesheet::validate_contract();
        ShellStylesheet::validate_contract();
        SharedStylesheet::validate_contract();
        TreeViewStylesheet::validate_contract();
        AccordionStylesheet::validate_contract();
        DockSplitterStylesheet::validate_contract();
        SplitViewStylesheet::validate_contract();
        ResizablePanelsStylesheet::validate_contract();
        StepperStylesheet::validate_contract();
        SidebarStylesheet::validate_contract();
        ControlGroupStylesheet::validate_contract();
        RadioGroupStylesheet::validate_contract();
        AutocompleteStylesheet::validate_contract();
        ComboBoxStylesheet::validate_contract();
        SearchSelectorStylesheet::validate_contract();
        IconGroupStylesheet::validate_contract();
        TableStylesheet::validate_contract();
        PagerStylesheet::validate_contract();
        PopoverStylesheet::validate_contract();
        SlidePanelStylesheet::validate_contract();
        SelectorStylesheet::validate_contract();
        SelectorListStylesheet::validate_contract();
        ButtonStylesheet::validate_contract();
        ToggleStylesheet::validate_contract();
        CheckboxStylesheet::validate_contract();
        RadioStylesheet::validate_contract();
        TextFieldStylesheet::validate_contract();
        TextAreaStylesheet::validate_contract();
        ProgressStylesheet::validate_contract();
        FloatingMenuStylesheet::validate_contract();
        TooltipStylesheet::validate_contract();
        SelectionPanelStylesheet::validate_contract();
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
