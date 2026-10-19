mod config;
mod find;
mod metadata;
mod resolve;
mod selector;

pub fn embedded_stylesheet() -> &'static StylesheetConfig {
    embedded()
}

pub use config::{ButtonElevationRule, FloatingMenuSurfaceElevationRule, LayeredElevationRule, StylesheetConfig};
pub use find::*;
pub use metadata::*;
pub use resolve::{
    resolve_accordion_content_color_rule, resolve_accordion_trigger_color_rule, resolve_button_color_rule,
    resolve_badge_color_rule, resolve_button_metrics_rule, resolve_card_color_rule, resolve_checkbox_color_rule,
    resolve_control_group_list_color_rule, resolve_floating_menu_surface_color_rule,
    resolve_floating_menu_trigger_color_rule, resolve_table_row_color_rule, resolve_table_surface_color_rule,
    resolve_listbox_list_color_rule, resolve_listbox_row_color_rule, resolve_sidebar_branch_color_rule,
    resolve_sidebar_container_color_rule, resolve_sidebar_item_color_rule, resolve_sidebar_metrics,
    resolve_sidebar_section_color_rule, resolve_autocomplete_chrome_color_rule, resolve_progress_color_rule,
    resolve_progress_metrics, resolve_color_ref, resolve_stepper_metrics, resolve_radio_color_rule,
    resolve_resizable_panels_color_rule, resolve_scrollbar_color_rule, resolve_scrollbar_metrics,
    resolve_slider_color_rule, resolve_slider_metrics, resolve_split_view_color_rule, resolve_stylesheet_shadow_token,
    resolve_layered_elevation_shadow, resolve_switch_color_rule, resolve_switch_metrics, resolve_stylesheet_metric,
    resolve_tabs_item_color_rule, resolve_tabs_list_color_rule, resolve_textfield_color_rule,
    resolve_tree_view_row_color_rule, resolve_typography_rule, ResolvedFields,
};

use std::sync::OnceLock;

fn embedded() -> &'static StylesheetConfig {
    static STYLESHEET: OnceLock<StylesheetConfig> = OnceLock::new();
    STYLESHEET.get_or_init(|| {
        let mut config = StylesheetConfig::native_default();
        define_shadcn_slider(&mut config);
        define_shadcn_switch(&mut config);
        define_shadcn_checkbox(&mut config);
        config
    })
}

/// Construct the complete Shadcn common overrides using native definitions.
pub fn native_shadcn_common_stylesheet() -> gpui_luma::theme::stylesheet::CommonStylesheet {
    use gpui_luma::theme::stylesheet::*;
    use std::collections::HashMap;
    let mut motion = CommonStylesheet::sdk_default().motion.clone();
    motion.tabs.content = Some(TransitionSpec::Timed { duration_ms: 300, easing: gpui_luma::motion::Easing::OutCubic });
    CommonStylesheet {
        motion,
        slider: native_shadcn_common_slider(),
        switch: native_shadcn_common_switch(),
        tabs: TabsStylesheet {
            geometry: TabsGeometryStylesheet { indicator_height: Some(3.0), ..Default::default() },
            ..Default::default()
        },
        button: ButtonStylesheet {
            sizes: [("sm", 12.0, 12.0, 14.0), ("md", 16.0, 14.0, 16.0), ("lg", 20.0, 16.0, 18.0)]
                .into_iter()
                .map(|(key, padding, font, icon)| {
                    (
                        key.into(),
                        ButtonGeometryStylesheet {
                            padding_x: Some(padding),
                            font_size: Some(font),
                            icon_size: Some(icon),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
            ..Default::default()
        },
        toggle: ToggleStylesheet {
            sizes: [("sm", 10.0, 12.0, 14.0, 36.0), ("md", 12.0, 14.0, 16.0, 40.0), ("lg", 20.0, 16.0, 18.0, 44.0)]
                .into_iter()
                .map(|(key, padding, font, icon, height)| {
                    (
                        key.into(),
                        ToggleGeometryStylesheet {
                            padding_x: Some(padding),
                            font_size: Some(font),
                            icon_size: Some(icon),
                            height: Some(height),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
            ..Default::default()
        },
        scrollbar: ScrollbarStylesheet {
            geometry: ScrollbarGeometryStylesheet {
                length_h: Some(260.0),
                length_v: Some(180.0),
                ..Default::default()
            },
            sizes: ["sm", "md", "lg"]
                .into_iter()
                .map(|key| (key.into(), ScrollbarGeometryStylesheet::default()))
                .collect(),
        },
        progress: ProgressStylesheet {
            sizes: ["sm", "md", "lg"]
                .into_iter()
                .map(|key| (key.into(), ProgressGeometryStylesheet::default()))
                .collect(),
            ..Default::default()
        },
        stepper: StepperStylesheet {
            sizes: ["sm", "md", "lg"]
                .into_iter()
                .map(|key| (key.into(), StepperGeometryStylesheet::default()))
                .collect(),
            ..Default::default()
        },
        tooltip: TooltipStylesheet {
            geometry: TooltipGeometryStylesheet {
                padding: Some(8.0),
                padding_y: Some(4.0),
                font_size: Some(12.0),
                line_height: Some(16.0),
                ..Default::default()
            },
            ..Default::default()
        },
        toolbar: ToolbarStylesheet {
            geometry: ToolbarGeometryStylesheet { padding_x: Some(6.0), padding_y: Some(4.0), ..Default::default() },
            ..Default::default()
        },
        sidebar: SidebarStylesheet {
            geometry: SidebarGeometryStylesheet {
                section_height: Some(20.0),
                item_height: Some(30.0),
                item_padding_x: Some(8.0),
                ..Default::default()
            },
            ..Default::default()
        },
        context_menu: ContextMenuStylesheet {
            geometry: ContextMenuGeometryStylesheet { min_width: Some(200.0), ..Default::default() },
            ..Default::default()
        },
        overlay_window: OverlayWindowStylesheet {
            geometry: OverlayWindowGeometryStylesheet { backdrop_opacity: Some(0.42), ..Default::default() },
            sizes: HashMap::from([
                ("sm".into(), OverlayWindowGeometryStylesheet { max_width: Some(420.0), ..Default::default() }),
                ("md".into(), OverlayWindowGeometryStylesheet::default()),
                ("lg".into(), OverlayWindowGeometryStylesheet { max_width: Some(620.0), ..Default::default() }),
            ]),
        },
        pager: PagerStylesheet {
            geometry: PagerGeometryStylesheet {
                padding_x: Some(8.0),
                padding_y: Some(8.0),
                gap: Some(4.0),
                ..Default::default()
            },
            sizes: HashMap::from([
                (
                    "minimal".into(),
                    PagerGeometryStylesheet {
                        button_size: Some(28.0),
                        button_min_width: Some(28.0),
                        group_gap: Some(8.0),
                        ..Default::default()
                    },
                ),
                (
                    "minimal-edge".into(),
                    PagerGeometryStylesheet {
                        button_min_width: Some(32.0),
                        group_gap: Some(16.0),
                        ..Default::default()
                    },
                ),
                (
                    "numeric".into(),
                    PagerGeometryStylesheet {
                        button_min_width: Some(32.0),
                        group_gap: Some(16.0),
                        ..Default::default()
                    },
                ),
            ]),
        },
        ..Default::default()
    }
    .with_sdk_defaults()
}

/// Constructs the native Shadcn common slider sizes in pure Rust without preloading TOML.
pub fn native_shadcn_common_slider() -> gpui_luma::theme::stylesheet::SliderStylesheet {
    let mut slider = gpui_luma::theme::stylesheet::SliderStylesheet::default();
    slider
        .size("sm", |m| {
            m.width(260.0).dimensions(24.0, 4.0, 12.0);
        })
        .size("md", |m| {
            m.width(260.0).dimensions(32.0, 6.0, 16.0);
        })
        .size("lg", |m| {
            m.width(260.0).dimensions(40.0, 8.0, 20.0);
        });
    slider
}

/// Constructs native Shadcn slider elevation and color rules in pure Rust without preloading TOML.
pub fn native_shadcn_slider() -> config::SliderStylesheet {
    let elevation_rules = vec![
        config::LayeredElevationRule { layer: None, shadow: "shadow-sm".into() },
        config::LayeredElevationRule { layer: Some("disabled".into()), shadow: "none".into() },
    ];

    let mut color_rules = Vec::with_capacity(8);
    for (style, default_fill) in [("primary", "@action_default"), ("secondary", "@action_foreground")] {
        color_rules.push(config::SliderColorRule {
            style: Some(style.into()),
            layer: Some("disabled".into()),
            track_background: "border".into(),
            fill_background: "muted-foreground".into(),
            thumb_background: "first(background,card)".into(),
            thumb_border: "muted".into(),
        });
        for (layer, fill) in [
            ("default", default_fill),
            (
                "hover",
                if style == "primary" {
                    "@action_layer"
                } else {
                    default_fill
                },
            ),
            (
                "pressed",
                if style == "primary" {
                    "@action_layer"
                } else {
                    default_fill
                },
            ),
        ] {
            color_rules.push(config::SliderColorRule {
                style: Some(style.into()),
                layer: Some(layer.into()),
                track_background: "border".into(),
                fill_background: fill.into(),
                thumb_background: "first(background,card)".into(),
                thumb_border: default_fill.into(),
            });
        }
    }

    config::SliderStylesheet { metrics: std::collections::HashMap::new(), elevation_rules, color_rules }
}

/// Programmatic definition of all Shadcn slider rules.
pub fn define_shadcn_slider(config: &mut StylesheetConfig) {
    native_shadcn_common_slider().apply_to(&mut config.common.slider);
    config.slider = native_shadcn_slider();
}

/// Construct Shadcn switch size overrides without parsing bundled data.
pub fn native_shadcn_common_switch() -> gpui_luma::theme::stylesheet::SwitchStylesheet {
    let mut switch = gpui_luma::theme::stylesheet::SwitchStylesheet::default();
    switch
        .track_padding(2.0)
        .size("sm", |m| {
            m.dimensions(32.0, 13.0, 9.0);
        })
        .size("md", |m| {
            m.dimensions(40.0, 18.0, 14.0);
        })
        .size("lg", |m| {
            m.dimensions(48.0, 24.0, 20.0);
        });
    switch
}

/// Construct Shadcn switch rules, preserving disabled-first matching order.
pub fn native_shadcn_switch() -> config::SwitchStylesheet {
    config::SwitchStylesheet {
        metrics: Default::default(),
        elevation_rules: vec![
            config::LayeredElevationRule { layer: None, shadow: "shadow-sm".into() },
            config::LayeredElevationRule { layer: Some("disabled".into()), shadow: "none".into() },
        ],
        color_rules: vec![
            config::SwitchColorRule {
                on: None,
                disabled: Some(true),
                track_background: "muted".into(),
                thumb_background: "muted-foreground".into(),
                thumb_border: "muted".into(),
                label_color: "muted-foreground".into(),
            },
            config::SwitchColorRule {
                on: Some(true),
                disabled: Some(false),
                track_background: "@action_default".into(),
                thumb_background: "@action_foreground".into(),
                thumb_border: "@action_foreground".into(),
                label_color: "foreground".into(),
            },
            config::SwitchColorRule {
                on: Some(false),
                disabled: Some(false),
                track_background: "input".into(),
                thumb_background: "background".into(),
                thumb_border: "border".into(),
                label_color: "foreground".into(),
            },
        ],
    }
}

/// Install native Shadcn switch overrides over the SDK baseline.
pub fn define_shadcn_switch(config: &mut StylesheetConfig) {
    native_shadcn_common_switch().apply_to(&mut config.common.switch);
    config.switch = native_shadcn_switch();
}

/// Construct Shadcn checkbox rules without parsing bundled data.
pub fn native_shadcn_checkbox() -> config::CheckboxStylesheet {
    config::CheckboxStylesheet {
        elevation_rules: vec![
            config::LayeredElevationRule { layer: None, shadow: "shadow-xs".into() },
            config::LayeredElevationRule { layer: Some("disabled".into()), shadow: "none".into() },
        ],
        color_rules: vec![
            config::CheckboxColorRule {
                layer: Some("disabled".into()),
                checked: None,
                indicator_background: "muted".into(),
                checkmark_color: "muted-foreground".into(),
                label_color: "muted-foreground".into(),
            },
            config::CheckboxColorRule {
                layer: None,
                checked: Some(false),
                indicator_background: "@outline_layer".into(),
                checkmark_color: "foreground".into(),
                label_color: "foreground".into(),
            },
            config::CheckboxColorRule {
                layer: None,
                checked: Some(true),
                indicator_background: "@action_layer".into(),
                checkmark_color: "@action_foreground".into(),
                label_color: "foreground".into(),
            },
        ],
    }
}

/// Install native Shadcn checkbox overrides over the SDK baseline.
pub fn define_shadcn_checkbox(config: &mut StylesheetConfig) {
    config.checkbox = native_shadcn_checkbox();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::BadgeVariant;
    use gpui_luma::theme::{InteractionLayer, ThemeMode};

    #[test]
    fn embedded_stylesheet_parses() {
        let stylesheet = embedded_stylesheet();
        assert!(stylesheet.typography.scale_rule("xs").is_some());
        assert!(stylesheet.typography.semantic_rule("h1").is_some());
        assert!(!stylesheet.button.color_rules.is_empty());
        assert_eq!(stylesheet.badge.color_rules.len(), 5);
        assert_eq!(stylesheet.checkbox.color_rules.len(), 3);
        assert_eq!(stylesheet.checkbox.elevation_rules.len(), 2);
        assert_eq!(stylesheet.toggle.elevation_rules.len(), 2);
        assert_eq!(stylesheet.radio.color_rules.len(), 5);
        assert_eq!(stylesheet.radio.elevation_rules.len(), 2);
        assert_eq!(stylesheet.switch.color_rules.len(), 3);
        assert_eq!(stylesheet.switch.elevation_rules.len(), 2);
        assert_eq!(stylesheet.slider.color_rules.len(), 8);
        assert_eq!(stylesheet.slider.elevation_rules.len(), 2);
        assert_eq!(stylesheet.card.elevation_rules.len(), 1);
        assert_eq!(stylesheet.scrollbar.color_rules.len(), 8);
        assert_eq!(stylesheet.accordion.trigger.color_rules.len(), 5);
        assert_eq!(stylesheet.resizable_panels.color_rules.len(), 5);
        assert_eq!(stylesheet.listbox.list.color_rules.len(), 2);
        assert_eq!(stylesheet.listbox.row.color_rules.len(), 6);
        assert_eq!(stylesheet.table.row.color_rules.len(), 6);
        assert_eq!(stylesheet.floating_menu.trigger.color_rules.len(), 5);
        assert_eq!(stylesheet.floating_menu.surface.elevation_rules.len(), 1);
        assert_eq!(stylesheet.tabs.item.color_rules.len(), 10);
        assert_eq!(stylesheet.tree_view.row.color_rules.len(), 5);
        assert_eq!(stylesheet.sidebar.item.color_rules.len(), 10);
        assert_eq!(stylesheet.textfield.color_rules.len(), 16);
        assert_eq!(stylesheet.textfield.elevation_rules.len(), 4);
        assert_eq!(stylesheet.autocomplete.chrome.color_rules.len(), 1);
        assert_eq!(stylesheet.progress.color_rules.len(), 2);
        assert_eq!(stylesheet.split_view.color_rules.len(), 2);
        assert_eq!(stylesheet.control_group.list.color_rules.len(), 2);
    }

    #[test]
    fn enabled_controls_find_matching_rules() {
        let stylesheet = embedded_stylesheet();
        assert!(find_badge_color_rule(stylesheet, BadgeVariant::Default, ThemeMode::Dark).is_some());
        assert!(find_progress_color_rule(stylesheet, true).is_some());
        assert!(find_split_view_color_rule(stylesheet, false).is_some());
        assert!(find_control_group_list_color_rule(stylesheet, true).is_some());
        assert!(find_textfield_color_rule(stylesheet, "input", true, false, ThemeMode::Light).is_some());
        assert!(find_textfield_color_rule(stylesheet, "primary", true, false, ThemeMode::Light).is_some());
        assert!(find_textfield_elevation_rule(stylesheet, crate::controls::ShadcnTextFieldStyle::Primary).is_some());
        assert!(find_autocomplete_chrome_color_rule(stylesheet).is_some());
        assert!(find_checkbox_color_rule(stylesheet, true, InteractionLayer::Default).is_some());
        assert!(find_slider_color_rule(stylesheet, "primary", InteractionLayer::Hovered).is_some());
        assert!(find_slider_color_rule(stylesheet, "secondary", InteractionLayer::Default).is_some());
    }

    #[test]
    fn embedded_color_rule_metadata_covers_all_controls() {
        let sections = embedded_color_rule_metadata();
        assert_eq!(sections.len(), 28);
        assert!(sections.iter().all(|section| !section.rules.is_empty()));
        assert_eq!(
            sections.iter().map(|section| section.rules.len()).sum::<usize>(),
            embedded_stylesheet().button.color_rules.len()
                + embedded_stylesheet().badge.color_rules.len()
                + embedded_stylesheet().checkbox.color_rules.len()
                + embedded_stylesheet().radio.color_rules.len()
                + embedded_stylesheet().switch.color_rules.len()
                + embedded_stylesheet().slider.color_rules.len()
                + embedded_stylesheet().scrollbar.color_rules.len()
                + embedded_stylesheet().accordion.trigger.color_rules.len()
                + embedded_stylesheet().accordion.content.color_rules.len()
                + embedded_stylesheet().resizable_panels.color_rules.len()
                + embedded_stylesheet().listbox.list.color_rules.len()
                + embedded_stylesheet().listbox.row.color_rules.len()
                + embedded_stylesheet().table.surface.color_rules.len()
                + embedded_stylesheet().table.row.color_rules.len()
                + embedded_stylesheet().floating_menu.surface.color_rules.len()
                + embedded_stylesheet().floating_menu.trigger.color_rules.len()
                + embedded_stylesheet().tabs.list.color_rules.len()
                + embedded_stylesheet().tabs.item.color_rules.len()
                + embedded_stylesheet().tree_view.row.color_rules.len()
                + embedded_stylesheet().sidebar.container.color_rules.len()
                + embedded_stylesheet().sidebar.section.color_rules.len()
                + embedded_stylesheet().sidebar.branch.color_rules.len()
                + embedded_stylesheet().sidebar.item.color_rules.len()
                + embedded_stylesheet().textfield.color_rules.len()
                + embedded_stylesheet().autocomplete.chrome.color_rules.len()
                + embedded_stylesheet().progress.color_rules.len()
                + embedded_stylesheet().split_view.color_rules.len()
                + embedded_stylesheet().control_group.list.color_rules.len()
        );
    }

    #[test]
    fn independent_control_assembly_matches_bundled_styles() {
        use gpui_luma::theme::stylesheet::{define_sdk_checkbox, define_sdk_slider, define_sdk_switch};
        let original = StylesheetConfig::native_default();
        // Assemble the controls independently from an empty sheet.
        let mut native = StylesheetConfig::default();
        define_sdk_slider(&mut native.common);
        define_sdk_switch(&mut native.common);
        define_sdk_checkbox(&mut native.common);
        define_shadcn_slider(&mut native);
        define_shadcn_switch(&mut native);
        define_shadcn_checkbox(&mut native);
        assert_eq!(native.common.slider, original.common.slider);
        assert_eq!(native.common.switch, original.common.switch);
        assert_eq!(native.common.checkbox, original.common.checkbox);
        assert_eq!(native.slider, original.slider);
        assert_eq!(native.switch, original.switch);
        assert_eq!(native.checkbox, original.checkbox);
        assert_eq!(native.common.motion.slider, original.common.motion.slider);
        assert_eq!(native.common.motion.switch, original.common.motion.switch);
        assert_eq!(native.common.motion.checkbox, original.common.motion.checkbox);
        for family in ["slider", "switch", "checkbox"] {
            assert_eq!(native.common.recipes.0.get(family), original.common.recipes.0.get(family));
        }

        let look = crate::ShadcnLook::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let tokens = look.mode_tokens();
            for style in [crate::controls::ShadcnButtonStyle::Primary, crate::controls::ShadcnButtonStyle::Secondary] {
                for size in [
                    gpui_luma::theme::ControlSize::Sm,
                    gpui_luma::theme::ControlSize::Md,
                    gpui_luma::theme::ControlSize::Lg,
                ] {
                    for bits in 0..32 {
                        let state = gpui_luma::theme::InteractionState {
                            hovered: bits & 1 != 0,
                            pressed: bits & 2 != 0,
                            focused: bits & 4 != 0,
                            disabled: bits & 8 != 0,
                            invalid: bits & 16 != 0,
                        };
                        use gpui_luma::controls::slider::SliderThumbSize;
                        for thumb in
                            [None, Some(SliderThumbSize::Sm), Some(SliderThumbSize::Md), Some(SliderThumbSize::Lg)]
                        {
                            let old = crate::controls::slider::slider_look_with_stylesheet(
                                &tokens, &original, mode, style, size, thumb, state,
                            );
                            let new = crate::controls::slider::slider_look_with_stylesheet(
                                &tokens, &native, mode, style, size, thumb, state,
                            );
                            assert_eq!(
                                format!("{new:?}"),
                                format!("{old:?}"),
                                "{mode:?} {style:?} {size:?} {thumb:?} {state:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}
