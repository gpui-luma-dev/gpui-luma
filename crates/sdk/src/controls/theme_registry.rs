use crate::theme::ThemeUsage;

pub fn all_theme_usages() -> &'static [&'static ThemeUsage] {
    REGISTERED_THEME_USAGES
}

const REGISTERED_THEME_USAGES: &[&ThemeUsage] = &[
    &crate::controls::button_family::BUTTON_THEME_USAGE,
    &crate::controls::navigation_sidebar::NAVIGATION_SIDEBAR_THEME_USAGE,
    &crate::controls::button_family::ICON_BUTTON_THEME_USAGE,
    &crate::controls::button_family::TOGGLE_THEME_USAGE,
    &crate::controls::control_group::CONTROL_GROUP_THEME_USAGE,
    &crate::controls::checkbox::CHECKBOX_THEME_USAGE,
    &crate::controls::radio_button::RADIO_BUTTON_THEME_USAGE,
    &crate::controls::switch::SWITCH_THEME_USAGE,
    &crate::controls::slider::SLIDER_THEME_USAGE,
    &crate::controls::scrollbar::SCROLLBAR_THEME_USAGE,
    &crate::controls::textarea::TEXTAREA_THEME_USAGE,
    &crate::controls::textfield::TEXTFIELD_THEME_USAGE,
    &crate::controls::autocomplete::AUTOCOMPLETE_TEXTBOX_THEME_USAGE,
    &crate::controls::autocomplete::COMBOBOX_THEME_USAGE,
    &crate::controls::search_selector::SEARCH_SELECTOR_THEME_USAGE,
    &crate::controls::floating_menu::FLOATING_MENU_THEME_USAGE,
    &crate::controls::popup_menu::POPUP_MENU_THEME_USAGE,
    &crate::controls::selector::SELECTOR_THEME_USAGE,
    &crate::controls::context_menu::CONTEXT_MENU_THEME_USAGE,
    &crate::controls::tabs_navigation::TABS_NAVIGATION_THEME_USAGE,
    &crate::controls::listbox::LISTBOX_THEME_USAGE,
    &crate::controls::progress::PROGRESS_THEME_USAGE,
    &crate::controls::selection_panel::SELECTION_PANEL_THEME_USAGE,
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::all_theme_usages;
    use crate::theme::ThemeTokens;
    use crate::theme::resolve_palette_color;

    #[test]
    fn usage_metadata_tokens_are_known_palette_tokens() {
        let tokens = ThemeTokens::default();

        for usage in all_theme_usages() {
            for part in usage.parts {
                assert!(
                    resolve_palette_color(&tokens, part.token).is_some(),
                    "{} uses unknown palette token {}",
                    usage.label,
                    part.token
                );
            }
        }
    }

    #[test]
    fn usage_labels_are_unique() {
        let mut labels = HashSet::new();

        for usage in all_theme_usages() {
            assert!(labels.insert(usage.label), "duplicate theme usage label: {}", usage.label);
        }
    }
}
