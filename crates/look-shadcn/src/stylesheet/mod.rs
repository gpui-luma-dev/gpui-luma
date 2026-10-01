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

const EMBEDDED_STYLE_TOML: &str = include_str!("../../assets/style.toml");

fn embedded() -> &'static StylesheetConfig {
    static STYLESHEET: OnceLock<StylesheetConfig> = OnceLock::new();
    STYLESHEET.get_or_init(|| StylesheetConfig::parse(EMBEDDED_STYLE_TOML).expect("embedded style.toml should parse"))
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
}
