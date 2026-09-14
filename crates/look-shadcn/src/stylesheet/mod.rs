mod config;
mod resolve;
mod selector;

pub fn embedded_stylesheet() -> &'static StylesheetConfig {
    embedded()
}

pub use config::{ButtonElevationRule, FloatingMenuSurfaceElevationRule, LayeredElevationRule, StylesheetConfig};
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

use luma::theme::{InteractionLayer, ThemeMode};

use crate::controls::ShadcnButtonStyle;
use crate::elements::BadgeVariant;
use crate::provenance::TableRuleMetadata;
use crate::tokens::{ShadcnTextRole, ShadcnTextSize};

use config::{
    AccordionContentColorRule, AccordionTriggerColorRule, AutocompleteChromeColorRule, BadgeColorRule, ButtonColorRule,
    CardColorRule, CheckboxColorRule, ControlGroupListColorRule, FloatingMenuSurfaceColorRule,
    FloatingMenuTriggerColorRule, ListboxListColorRule, ListboxRowColorRule, TableRowColorRule, TableSurfaceColorRule,
    SidebarBranchColorRule, SidebarContainerColorRule, SidebarItemColorRule, SidebarSectionColorRule,
    ProgressColorRule, RadioColorRule, ResizablePanelsColorRule, ScrollbarColorRule, SliderColorRule,
    SplitViewColorRule, SwitchColorRule, TabsItemColorRule, TabsListColorRule, TextfieldColorRule,
    TreeViewRowColorRule, TypographyRule,
};
use selector::{AsSelectorState, ButtonSelectorState, badge_variant_key, theme_mode_key};

const EMBEDDED_STYLE_TOML: &str = include_str!("../../assets/style.toml");

fn embedded() -> &'static StylesheetConfig {
    static STYLESHEET: OnceLock<StylesheetConfig> = OnceLock::new();
    STYLESHEET.get_or_init(|| StylesheetConfig::parse(EMBEDDED_STYLE_TOML).expect("embedded style.toml should parse"))
}

pub fn find_button_color_rule(
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
    theme_mode: ThemeMode,
    selected: bool,
) -> Option<&ButtonColorRule> {
    let state = ButtonSelectorState { style, layer, mode: theme_mode, selected };
    let map = state.to_selector_map();
    stylesheet.button.find_color_rule(
        map.get("style").expect("style key"),
        map.get("layer").expect("layer key"),
        map.get("mode").expect("mode key"),
        selected,
    )
}

pub fn find_button_elevation_rule(
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
) -> Option<&crate::stylesheet::config::ButtonElevationRule> {
    stylesheet.button.elevation_rule_for_style(crate::stylesheet::selector::button_style_key(style))
}

pub fn find_textfield_elevation_rule(
    stylesheet: &StylesheetConfig,
    style: crate::controls::ShadcnTextFieldStyle,
) -> Option<&crate::stylesheet::config::TextfieldElevationRule> {
    stylesheet.textfield.elevation_rule_for_style(textfield_style_key(style))
}

fn textfield_style_key(style: crate::controls::ShadcnTextFieldStyle) -> &'static str {
    match style {
        crate::controls::ShadcnTextFieldStyle::Outline => "outline",
        crate::controls::ShadcnTextFieldStyle::Input => "input",
        crate::controls::ShadcnTextFieldStyle::Primary => "primary",
        crate::controls::ShadcnTextFieldStyle::Surface => "surface",
    }
}

pub fn find_checkbox_color_rule(
    stylesheet: &StylesheetConfig,
    checked: bool,
    layer: InteractionLayer,
) -> Option<&CheckboxColorRule> {
    stylesheet.checkbox.find_color_rule(checked, layer)
}

pub fn find_radio_color_rule(
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    selected: bool,
    layer: InteractionLayer,
) -> Option<&RadioColorRule> {
    stylesheet.radio.find_color_rule(style, selected, layer)
}

pub fn find_switch_color_rule(stylesheet: &StylesheetConfig, on: bool, disabled: bool) -> Option<&SwitchColorRule> {
    stylesheet.switch.find_color_rule(on, disabled)
}

pub fn find_slider_color_rule<'a>(
    stylesheet: &'a StylesheetConfig,
    style: &str,
    layer: InteractionLayer,
) -> Option<&'a SliderColorRule> {
    stylesheet.slider.find_color_rule(style, layer)
}

pub fn find_scrollbar_color_rule<'a>(
    stylesheet: &'a StylesheetConfig,
    style: &str,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&'a ScrollbarColorRule> {
    stylesheet.scrollbar.find_color_rule(style, disabled, layer)
}

pub fn find_accordion_trigger_color_rule(
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&AccordionTriggerColorRule> {
    stylesheet.accordion.trigger.find_color_rule(disabled, layer)
}

pub fn find_accordion_content_color_rule(
    stylesheet: &StylesheetConfig,
    expanded: bool,
) -> Option<&AccordionContentColorRule> {
    stylesheet.accordion.content.find_color_rule(expanded)
}

pub fn find_resizable_panels_color_rule(
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&ResizablePanelsColorRule> {
    stylesheet.resizable_panels.find_color_rule(disabled, layer)
}

pub fn find_listbox_list_color_rule(stylesheet: &StylesheetConfig, enabled: bool) -> Option<&ListboxListColorRule> {
    stylesheet.listbox.list.find_color_rule(enabled)
}

pub fn find_listbox_row_color_rule(
    stylesheet: &StylesheetConfig,
    disabled: bool,
    focused: bool,
    layer: InteractionLayer,
) -> Option<&ListboxRowColorRule> {
    stylesheet.listbox.row.find_color_rule(disabled, focused, layer)
}

pub fn find_table_surface_color_rule(stylesheet: &StylesheetConfig, enabled: bool) -> Option<&TableSurfaceColorRule> {
    stylesheet.table.surface.find_color_rule(enabled)
}

pub fn find_table_row_color_rule(
    stylesheet: &StylesheetConfig,
    selected: bool,
    focused: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&TableRowColorRule> {
    stylesheet.table.row.find_color_rule(selected, focused, disabled, layer)
}

pub fn find_floating_menu_surface_color_rule(stylesheet: &StylesheetConfig) -> Option<&FloatingMenuSurfaceColorRule> {
    stylesheet.floating_menu.surface.color_rule()
}

pub fn find_floating_menu_surface_elevation_rule(
    stylesheet: &StylesheetConfig,
) -> Option<&FloatingMenuSurfaceElevationRule> {
    stylesheet.floating_menu.surface.elevation_rule()
}

pub fn find_floating_menu_trigger_color_rule(
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&FloatingMenuTriggerColorRule> {
    stylesheet.floating_menu.trigger.find_color_rule(disabled, layer)
}

pub fn find_tabs_list_color_rule(stylesheet: &StylesheetConfig, enabled: bool) -> Option<&TabsListColorRule> {
    stylesheet.tabs.list.find_color_rule(enabled)
}

pub fn find_tabs_item_color_rule(
    stylesheet: &StylesheetConfig,
    active: bool,
    layer: InteractionLayer,
    focused: bool,
) -> Option<&TabsItemColorRule> {
    stylesheet.tabs.item.find_color_rule(active, layer, focused)
}

pub fn find_tree_view_row_color_rule(
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&TreeViewRowColorRule> {
    stylesheet.tree_view.row.find_color_rule(disabled, layer)
}

pub fn find_sidebar_metrics(stylesheet: &StylesheetConfig) -> &crate::stylesheet::config::SidebarMetricsRule {
    &stylesheet.sidebar.metrics
}

pub fn find_sidebar_container_color_rule(stylesheet: &StylesheetConfig) -> Option<&SidebarContainerColorRule> {
    stylesheet.sidebar.container.color_rule()
}

pub fn find_sidebar_section_color_rule(stylesheet: &StylesheetConfig) -> Option<&SidebarSectionColorRule> {
    stylesheet.sidebar.section.color_rule()
}

pub fn find_sidebar_branch_color_rule(
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&SidebarBranchColorRule> {
    stylesheet.sidebar.branch.find_color_rule(disabled, layer)
}

pub fn find_sidebar_item_color_rule(
    stylesheet: &StylesheetConfig,
    selected: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> Option<&SidebarItemColorRule> {
    stylesheet.sidebar.item.find_color_rule(selected, disabled, layer)
}

pub fn find_textfield_color_rule<'a>(
    stylesheet: &'a StylesheetConfig,
    style: &str,
    enabled: bool,
    invalid: bool,
    theme_mode: ThemeMode,
) -> Option<&'a TextfieldColorRule> {
    stylesheet.textfield.find_color_rule(style, enabled, invalid, theme_mode_key(theme_mode))
}

pub fn find_typography_scale_rule(stylesheet: &StylesheetConfig, size: ShadcnTextSize) -> Option<&TypographyRule> {
    stylesheet.typography.scale_rule(size.as_str())
}

pub fn find_typography_semantic_rule(stylesheet: &StylesheetConfig, role: ShadcnTextRole) -> Option<&TypographyRule> {
    stylesheet.typography.semantic_rule(role.as_str())
}

pub fn find_autocomplete_chrome_color_rule(stylesheet: &StylesheetConfig) -> Option<&AutocompleteChromeColorRule> {
    stylesheet.autocomplete.chrome.color_rule()
}

pub fn find_progress_color_rule(stylesheet: &StylesheetConfig, enabled: bool) -> Option<&ProgressColorRule> {
    stylesheet.progress.find_color_rule(enabled)
}

pub fn find_card_color_rule(stylesheet: &StylesheetConfig) -> Option<&CardColorRule> {
    stylesheet.card.color_rule()
}

pub fn find_card_elevation_rule(stylesheet: &StylesheetConfig) -> Option<&FloatingMenuSurfaceElevationRule> {
    stylesheet.card.elevation_rule()
}

pub fn find_badge_color_rule(
    stylesheet: &StylesheetConfig,
    variant: BadgeVariant,
    theme_mode: ThemeMode,
) -> Option<&BadgeColorRule> {
    stylesheet.badge.find_color_rule(badge_variant_key(variant), theme_mode_key(theme_mode))
}

pub fn find_split_view_color_rule(stylesheet: &StylesheetConfig, enabled: bool) -> Option<&SplitViewColorRule> {
    stylesheet.split_view.find_color_rule(enabled)
}

pub fn find_control_group_list_color_rule(
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> Option<&ControlGroupListColorRule> {
    stylesheet.control_group.list.find_color_rule(enabled)
}

pub fn resolve_button_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .button
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.style.clone().unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
                rule.mode.clone().unwrap_or_else(|| "any".into()),
                rule.selected.map(|selected| selected.to_string()).unwrap_or_else(|| "any".into()),
            ],
            outputs: button_rule_output_strings(rule),
        })
        .collect()
}

pub fn resolve_checkbox_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .checkbox
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.layer.clone().unwrap_or_else(|| "any".into()),
                rule.checked.map(|checked| checked.to_string()).unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("indicator_background", &rule.indicator_background),
                color_output_label("checkmark_color", &rule.checkmark_color),
                color_output_label("label_color", &rule.label_color),
            ],
        })
        .collect()
}

pub fn resolve_radio_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .radio
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.layer.clone().unwrap_or_else(|| "any".into()),
                rule.selected.map(|selected| selected.to_string()).unwrap_or_else(|| "any".into()),
                rule.indicator.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("indicator_background", &rule.indicator_background),
                color_output_label("selection_ring", &rule.selection_ring),
                color_output_label("dot_color", &rule.dot_color),
                color_output_label("label_color", &rule.label_color),
            ],
        })
        .collect()
}

pub fn resolve_switch_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .switch
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.on.map(|on| on.to_string()).unwrap_or_else(|| "any".into()),
                rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("track_background", &rule.track_background),
                color_output_label("thumb_background", &rule.thumb_background),
                color_output_label("thumb_border", &rule.thumb_border),
                color_output_label("label_color", &rule.label_color),
            ],
        })
        .collect()
}

pub fn resolve_slider_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .slider
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.style.clone().unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("track_background", &rule.track_background),
                color_output_label("fill_background", &rule.fill_background),
                color_output_label("thumb_background", &rule.thumb_background),
                color_output_label("thumb_border", &rule.thumb_border),
            ],
        })
        .collect()
}

pub fn resolve_scrollbar_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .scrollbar
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("track_background", &rule.track_background),
                color_output_label("thumb_background", &rule.thumb_background),
            ],
        })
        .collect()
}

pub fn resolve_accordion_trigger_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .accordion
        .trigger
        .color_rules
        .iter()
        .map(|rule| {
            let mut outputs = vec![
                color_output_label("foreground", &rule.foreground),
                color_output_label("icon_color", &rule.icon_color),
                color_output_label("chevron_color", &rule.chevron_color),
                color_output_label("border_color", &rule.border_color),
            ];
            outputs.push(optional_color_output_label("background", rule.background.as_deref()));
            TableRuleMetadata {
                inputs: vec![
                    rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                    rule.layer.clone().unwrap_or_else(|| "any".into()),
                ],
                outputs,
            }
        })
        .collect()
}

pub fn resolve_accordion_content_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .accordion
        .content
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![rule.expanded.map(|expanded| expanded.to_string()).unwrap_or_else(|| "any".into())],
            outputs: vec![
                color_output_label("foreground", &rule.foreground),
                optional_color_output_label("background", rule.background.as_deref()),
            ],
        })
        .collect()
}

pub fn resolve_resizable_panels_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .resizable_panels
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("border", &rule.border),
                color_output_label("divider", &rule.divider),
                color_output_label("grip", &rule.grip),
                color_output_label("grip_emphasis", &rule.grip_emphasis),
            ],
        })
        .collect()
}

pub fn resolve_listbox_list_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .listbox
        .list
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![rule.enabled.map(|enabled| enabled.to_string()).unwrap_or_else(|| "any".into())],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("border", &rule.border),
                color_output_label("divider", &rule.divider),
            ],
        })
        .collect()
}

pub fn resolve_listbox_row_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .listbox
        .row
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                rule.focused.map(|focused| focused.to_string()).unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("label_color", &rule.label_color),
                color_output_label("background", &rule.background),
            ],
        })
        .collect()
}

pub fn resolve_table_surface_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .table
        .surface
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![rule.enabled.map(|enabled| enabled.to_string()).unwrap_or_else(|| "any".into())],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("border", &rule.border),
                color_output_label("header_background", &rule.header_background),
                color_output_label("header_label_color", &rule.header_label_color),
            ],
        })
        .collect()
}

pub fn resolve_table_row_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .table
        .row
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.selected.map(|selected| selected.to_string()).unwrap_or_else(|| "any".into()),
                rule.focused.map(|focused| focused.to_string()).unwrap_or_else(|| "any".into()),
                rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("label_color", &rule.label_color),
                color_output_label("divider", &rule.divider),
            ],
        })
        .collect()
}

pub fn resolve_floating_menu_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .floating_menu
        .surface
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec!["any".into()],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("foreground", &rule.foreground),
                color_output_label("border", &rule.border),
                color_output_label("item_hover_background", &rule.item_hover_background),
                color_output_label("item_hover_foreground", &rule.item_hover_foreground),
                color_output_label("item_disabled_foreground", &rule.item_disabled_foreground),
            ],
        })
        .collect()
}

pub fn resolve_ghost_trigger_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .floating_menu
        .trigger
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.layer.clone().unwrap_or_else(|| "any".into()),
                rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("foreground", &rule.foreground),
            ],
        })
        .collect()
}

pub fn resolve_tabs_list_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .tabs
        .list
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![rule.enabled.map(|enabled| enabled.to_string()).unwrap_or_else(|| "any".into())],
            outputs: vec![color_output_label("disabled_background", &rule.disabled_background)],
        })
        .collect()
}

pub fn resolve_tabs_item_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .tabs
        .item
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.active.map(|active| active.to_string()).unwrap_or_else(|| "any".into()),
                rule.layer.clone().unwrap_or_else(|| "any".into()),
                rule.focused.map(|focused| focused.to_string()).unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("label_color", &rule.label_color),
                optional_color_output_label("indicator", rule.indicator.as_deref()),
            ],
        })
        .collect()
}

pub fn resolve_tree_view_row_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .tree_view
        .row
        .color_rules
        .iter()
        .map(|rule| {
            let mut outputs = vec![
                color_output_label("foreground", &rule.foreground),
                color_output_label("icon_color", &rule.icon_color),
                color_output_label("chevron_color", &rule.chevron_color),
            ];
            outputs.push(optional_color_output_label("background", rule.background.as_deref()));
            TableRuleMetadata {
                inputs: vec![
                    rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                    rule.layer.clone().unwrap_or_else(|| "any".into()),
                ],
                outputs,
            }
        })
        .collect()
}

pub fn resolve_sidebar_container_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .sidebar
        .container
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec!["any".into()],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("foreground", &rule.foreground),
                color_output_label("border", &rule.border),
            ],
        })
        .collect()
}

pub fn resolve_sidebar_section_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .sidebar
        .section
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec!["any".into()],
            outputs: vec![color_output_label("label_color", &rule.label_color)],
        })
        .collect()
}

pub fn resolve_sidebar_branch_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .sidebar
        .branch
        .color_rules
        .iter()
        .map(|rule| {
            let mut outputs = vec![
                color_output_label("foreground", &rule.foreground),
                color_output_label("icon_color", &rule.icon_color),
            ];
            outputs.push(optional_color_output_label("background", rule.background.as_deref()));
            TableRuleMetadata {
                inputs: vec![
                    rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                    rule.layer.clone().unwrap_or_else(|| "any".into()),
                ],
                outputs,
            }
        })
        .collect()
}

pub fn resolve_sidebar_item_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .sidebar
        .item
        .color_rules
        .iter()
        .map(|rule| {
            let mut outputs = vec![
                color_output_label("foreground", &rule.foreground),
                color_output_label("icon_color", &rule.icon_color),
            ];
            outputs.push(optional_color_output_label("background", rule.background.as_deref()));
            TableRuleMetadata {
                inputs: vec![
                    rule.selected.map(|selected| selected.to_string()).unwrap_or_else(|| "any".into()),
                    rule.disabled.map(|disabled| disabled.to_string()).unwrap_or_else(|| "any".into()),
                    rule.layer.clone().unwrap_or_else(|| "any".into()),
                ],
                outputs,
            }
        })
        .collect()
}

pub fn resolve_textfield_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .textfield
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.style.clone().unwrap_or_else(|| "any".into()),
                rule.enabled.map(|enabled| enabled.to_string()).unwrap_or_else(|| "any".into()),
                rule.invalid.map(|invalid| invalid.to_string()).unwrap_or_else(|| "any".into()),
                rule.mode.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("foreground", &rule.foreground),
                color_output_label("border", &rule.border),
                color_output_label("placeholder", &rule.placeholder),
                color_output_label("icon", &rule.icon),
                color_output_label("selection_background", &rule.selection_background),
                color_output_label("selection_foreground", &rule.selection_foreground),
                color_output_label("caret", &rule.caret),
            ],
        })
        .collect()
}

pub fn resolve_autocomplete_chrome_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .autocomplete
        .chrome
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec!["any".into()],
            outputs: vec![
                color_output_label("status_color", &rule.status_color),
                color_output_label("muted_text_color", &rule.muted_text_color),
                color_output_label("clear_icon_color", &rule.clear_icon_color),
                color_output_label("clear_icon_hover_color", &rule.clear_icon_hover_color),
            ],
        })
        .collect()
}

pub fn resolve_progress_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .progress
        .color_rules
        .iter()
        .map(|rule| enabled_color_rule_metadata(rule.enabled, progress_rule_output_strings(rule)))
        .collect()
}

pub fn resolve_badge_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .badge
        .color_rules
        .iter()
        .map(|rule| TableRuleMetadata {
            inputs: vec![
                rule.style.clone().unwrap_or_else(|| "any".into()),
                rule.mode.clone().unwrap_or_else(|| "any".into()),
            ],
            outputs: vec![
                color_output_label("background", &rule.background),
                color_output_label("foreground", &rule.foreground),
                optional_color_output_label("border", rule.border.as_deref()),
            ],
        })
        .collect()
}

pub fn resolve_split_view_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .split_view
        .color_rules
        .iter()
        .map(|rule| enabled_color_rule_metadata(rule.enabled, split_view_rule_output_strings(rule)))
        .collect()
}

pub fn resolve_control_group_list_colors_metadata(stylesheet: &StylesheetConfig) -> Vec<TableRuleMetadata> {
    stylesheet
        .control_group
        .list
        .color_rules
        .iter()
        .map(|rule| enabled_color_rule_metadata(rule.enabled, control_group_list_rule_output_strings(rule)))
        .collect()
}

fn enabled_color_rule_metadata(enabled: Option<bool>, outputs: Vec<String>) -> TableRuleMetadata {
    TableRuleMetadata { inputs: vec![enabled.map(|value| value.to_string()).unwrap_or_else(|| "any".into())], outputs }
}

fn color_output_label(field: &str, raw: &str) -> String {
    match raw.trim() {
        "transparent" => format!("{field}: transparent"),
        value if value.starts_with('@') || value.starts_with("first(") || value.starts_with("first_layer(") => {
            format!("{field}: {value}")
        }
        value => format!("{field}: --{}", value.trim_start_matches('@')),
    }
}

fn optional_color_output_label(field: &str, raw: Option<&str>) -> String {
    raw.map(|value| color_output_label(field, value)).unwrap_or_else(|| format!("{field}: none"))
}

fn button_rule_output_strings(rule: &ButtonColorRule) -> Vec<String> {
    let mut outputs = vec![
        format!("background: --{}", rule.background.trim_start_matches('@')),
        format!("foreground: --{}", rule.foreground.trim_start_matches('@')),
    ];
    if let Some(border) = &rule.border {
        outputs.push(format!("border: --{}", border.trim_start_matches('@')));
    }
    outputs
}

fn progress_rule_output_strings(rule: &ProgressColorRule) -> Vec<String> {
    vec![
        format!("track_color: --{}", rule.track_color.trim_start_matches('@')),
        format!("progress_color: --{}", rule.progress_color.trim_start_matches('@')),
    ]
}

fn split_view_rule_output_strings(rule: &SplitViewColorRule) -> Vec<String> {
    vec![
        format!("separator: --{}", rule.separator.trim_start_matches('@')),
        format!("separator_hover: --{}", rule.separator_hover.trim_start_matches('@')),
    ]
}

fn control_group_list_rule_output_strings(rule: &ControlGroupListColorRule) -> Vec<String> {
    vec![
        format!("background: --{}", rule.background.trim_start_matches('@')),
        format!("border: --{}", rule.border.trim_start_matches('@')),
    ]
}

/// One inspectable color-rule table from the loaded stylesheet.
#[derive(Clone, Debug)]
pub struct ColorRuleMetadataSection {
    pub control: &'static str,
    pub part: &'static str,
    pub rules: Vec<TableRuleMetadata>,
}

/// All color-rule metadata sections derived from a stylesheet config.
pub fn all_color_rule_metadata(stylesheet: &StylesheetConfig) -> Vec<ColorRuleMetadataSection> {
    vec![
        section(stylesheet, "button", "color", resolve_button_colors_metadata(stylesheet)),
        section(stylesheet, "badge", "color", resolve_badge_colors_metadata(stylesheet)),
        section(stylesheet, "checkbox", "color", resolve_checkbox_colors_metadata(stylesheet)),
        section(stylesheet, "radio", "color", resolve_radio_colors_metadata(stylesheet)),
        section(stylesheet, "switch", "color", resolve_switch_colors_metadata(stylesheet)),
        section(stylesheet, "slider", "color", resolve_slider_colors_metadata(stylesheet)),
        section(stylesheet, "scrollbar", "color", resolve_scrollbar_colors_metadata(stylesheet)),
        section(stylesheet, "accordion", "trigger", resolve_accordion_trigger_colors_metadata(stylesheet)),
        section(stylesheet, "accordion", "content", resolve_accordion_content_colors_metadata(stylesheet)),
        section(stylesheet, "resizable_panels", "color", resolve_resizable_panels_colors_metadata(stylesheet)),
        section(stylesheet, "listbox", "list", resolve_listbox_list_colors_metadata(stylesheet)),
        section(stylesheet, "listbox", "row", resolve_listbox_row_colors_metadata(stylesheet)),
        section(stylesheet, "table", "surface", resolve_table_surface_colors_metadata(stylesheet)),
        section(stylesheet, "table", "row", resolve_table_row_colors_metadata(stylesheet)),
        section(stylesheet, "floating_menu", "surface", resolve_floating_menu_colors_metadata(stylesheet)),
        section(stylesheet, "floating_menu", "trigger", resolve_ghost_trigger_colors_metadata(stylesheet)),
        section(stylesheet, "tabs", "list", resolve_tabs_list_colors_metadata(stylesheet)),
        section(stylesheet, "tabs", "item", resolve_tabs_item_colors_metadata(stylesheet)),
        section(stylesheet, "tree_view", "row", resolve_tree_view_row_colors_metadata(stylesheet)),
        section(stylesheet, "sidebar", "container", resolve_sidebar_container_colors_metadata(stylesheet)),
        section(stylesheet, "sidebar", "section", resolve_sidebar_section_colors_metadata(stylesheet)),
        section(stylesheet, "sidebar", "branch", resolve_sidebar_branch_colors_metadata(stylesheet)),
        section(stylesheet, "sidebar", "item", resolve_sidebar_item_colors_metadata(stylesheet)),
        section(stylesheet, "textfield", "color", resolve_textfield_colors_metadata(stylesheet)),
        section(stylesheet, "autocomplete", "chrome", resolve_autocomplete_chrome_colors_metadata(stylesheet)),
        section(stylesheet, "progress", "color", resolve_progress_colors_metadata(stylesheet)),
        section(stylesheet, "split_view", "color", resolve_split_view_colors_metadata(stylesheet)),
        section(stylesheet, "control_group", "list", resolve_control_group_list_colors_metadata(stylesheet)),
    ]
}

pub fn embedded_color_rule_metadata() -> Vec<ColorRuleMetadataSection> {
    all_color_rule_metadata(embedded_stylesheet())
}

fn section(
    _stylesheet: &StylesheetConfig,
    control: &'static str,
    part: &'static str,
    rules: Vec<TableRuleMetadata>,
) -> ColorRuleMetadataSection {
    ColorRuleMetadataSection { control, part, rules }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
