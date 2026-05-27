//! Radix CSS token usage metadata for gallery sidebars and theme docs.

use crate::theme::{ThemePartUsage, ThemeUsage};

pub fn all_radix_theme_usages() -> &'static [&'static ThemeUsage] {
    RADIX_THEME_USAGES
}

const TEXTFIELD_PARTS: &[ThemePartUsage] = &[
    part("background", "background", &["standard default"], &["TextFieldAppearance.background"]),
    part("border", "input", &["standard default"], &["TextFieldAppearance.border"]),
    part("foreground", "foreground", &["standard default"], &["TextFieldAppearance.foreground"]),
    part("placeholder", "muted-foreground", &["standard default"], &["TextFieldAppearance.placeholder"]),
    part("selection", "primary", &["standard focused"], &["TextFieldAppearance.selection_background"]),
    part("focus ring", "ring", &["standard focused"], &["TextFieldAppearance.focus_ring"]),
    part("ghost hover", "accent", &["ghost hovered"], &["TextFieldAppearance.background"]),
];

const BUTTON_PARTS: &[ThemePartUsage] = &[
    part("primary background", "primary", &["prominent default"], &["ButtonFamilyAppearance.background"]),
    part(
        "primary foreground",
        "primary-foreground",
        &["prominent default"],
        &["ButtonFamilyAppearance.foreground"],
    ),
    part("secondary background", "secondary", &["standard default"], &["ButtonFamilyAppearance.background"]),
    part(
        "secondary foreground",
        "secondary-foreground",
        &["standard default"],
        &["ButtonFamilyAppearance.foreground"],
    ),
    part("outline border", "border", &["subtle default"], &["ButtonFamilyAppearance.border"]),
    part("ghost hover", "accent", &["ghost hovered"], &["ButtonFamilyAppearance.background"]),
    part("disabled", "muted", &["disabled"], &["ButtonFamilyAppearance.background"]),
    part("focus ring", "ring", &["focused"], &["ButtonFamilyAppearance.adorner"]),
];

const RADIX_THEME_USAGES: &[&ThemeUsage] = &[
    &ThemeUsage { label: "Button", parts: BUTTON_PARTS },
    &ThemeUsage { label: "Icon Button", parts: BUTTON_PARTS },
    &ThemeUsage { label: "Toggle", parts: BUTTON_PARTS },
    &ThemeUsage {
        label: "Checkbox",
        parts: &[
            part("checked fill", "primary", &["checked"], &["CheckboxAppearance.background"]),
            part("checkmark", "primary-foreground", &["checked"], &["CheckboxAppearance.foreground"]),
            part("border", "input", &["unchecked"], &["CheckboxAppearance.border"]),
            part("focus ring", "ring", &["focused"], &["CheckboxAppearance.adorner"]),
        ],
    },
    &ThemeUsage {
        label: "Radio Button",
        parts: &[
            part("selected fill", "primary", &["selected"], &["RadioButtonAppearance.background"]),
            part("dot", "primary-foreground", &["selected"], &["RadioButtonAppearance.foreground"]),
            part("border", "input", &["unselected"], &["RadioButtonAppearance.border"]),
            part("focus ring", "ring", &["focused"], &["RadioButtonAppearance.adorner"]),
        ],
    },
    &ThemeUsage {
        label: "Switch",
        parts: &[
            part("off track", "input", &["off"], &["SwitchAppearance.track_background"]),
            part("track border", "border", &["off", "disabled"], &["SwitchAppearance.track_border"]),
            part(
                "on track",
                "primary",
                &["on"],
                &["SwitchAppearance.track_background", "SwitchAppearance.track_border"],
            ),
            part("off thumb", "background", &["off"], &["SwitchAppearance.thumb_background"]),
            part("thumb border", "border", &["off"], &["SwitchAppearance.thumb_border"]),
            part(
                "on thumb",
                "primary-foreground",
                &["on"],
                &["SwitchAppearance.thumb_background", "SwitchAppearance.thumb_border"],
            ),
            part(
                "disabled thumb",
                "muted-foreground",
                &["disabled"],
                &["SwitchAppearance.thumb_background"],
            ),
            part(
                "disabled thumb border",
                "muted",
                &["disabled"],
                &["SwitchAppearance.thumb_border"],
            ),
            part("focus ring", "ring", &["focused"], &["SwitchAppearance.adorner"]),
        ],
    },
    &ThemeUsage {
        label: "Slider",
        parts: &[
            part("track", "border", &["default"], &["SliderAppearance.track_background"]),
            part("fill", "primary", &["default"], &["SliderAppearance.fill_background"]),
            part("thumb border", "primary", &["default"], &["SliderAppearance.thumb_border"]),
            part("focus ring", "ring", &["focused"], &["SliderAppearance.focus_ring"]),
        ],
    },
    &ThemeUsage {
        label: "Scrollbar",
        parts: &[
            part("track", "background", &["default"], &["ScrollbarAppearance.track_color"]),
            part("thumb", "border", &["default", "hovered"], &["ScrollbarAppearance.thumb_color"]),
        ],
    },
    &ThemeUsage { label: "TextField", parts: TEXTFIELD_PARTS },
    &ThemeUsage { label: "TextArea", parts: TEXTFIELD_PARTS },
    &ThemeUsage {
        label: "ComboBox",
        parts: &[
            part("textbox", "background", &["default"], &["TextFieldAppearance.background"]),
            part("panel surface", "popover", &["open"], &["SelectorItemsPanelAppearance.background"]),
            part("item hover", "accent", &["hovered"], &["SelectorItemsPanelAppearance.item_hover_background"]),
        ],
    },
    &ThemeUsage {
        label: "AutocompleteTextField",
        parts: &[
            part("textbox background", "background", &["default"], &["AutocompleteTextBoxAppearance.background"]),
            part("textbox border", "input", &["default"], &["AutocompleteTextBoxAppearance.border"]),
            part("panel surface", "popover", &["open"], &["FloatingMenuAppearance.background"]),
        ],
    },
    &ThemeUsage {
        label: "SearchSelector",
        parts: &[
            part("textbox", "background", &["default"], &["TextFieldAppearance.background"]),
            part("panel surface", "popover", &["open"], &["SelectorItemsPanelAppearance.background"]),
        ],
    },
    &ThemeUsage {
        label: "Floating Menu",
        parts: &[
            part("surface", "popover", &["default"], &["FloatingMenuAppearance.background"]),
            part("foreground", "popover-foreground", &["default"], &["FloatingMenuAppearance.foreground"]),
            part("item hover", "accent", &["hovered"], &["FloatingMenuAppearance.item_hover_background"]),
            part("border", "border", &["default"], &["FloatingMenuAppearance.border"]),
        ],
    },
    &ThemeUsage {
        label: "Popup Menu",
        parts: &[
            part("trigger hover", "accent", &["hovered"], &["PopupMenuAppearance.trigger_background"]),
            part("menu surface", "popover", &["open"], &["FloatingMenuAppearance.background"]),
        ],
    },
    &ThemeUsage {
        label: "Selector",
        parts: &[
            part("trigger background", "background", &["default"], &["SelectorAppearance.background"]),
            part("trigger border", "input", &["default"], &["SelectorAppearance.border"]),
            part("panel surface", "popover", &["open"], &["SelectorItemsPanelAppearance.background"]),
        ],
    },
    &ThemeUsage {
        label: "Context Menu",
        parts: &[
            part("surface", "popover", &["default"], &["ContextMenuAppearance.background"]),
            part("item hover", "accent", &["hovered"], &["ContextMenuAppearance.item_hover_background"]),
        ],
    },
    &ThemeUsage {
        label: "Tabs Navigation",
        parts: &[
            part("inactive label", "foreground", &["inactive"], &["TabsNavigationItemAppearance.foreground"]),
            part("active label", "primary", &["active"], &["TabsNavigationItemAppearance.foreground"]),
            part("focus ring", "ring", &["focused"], &["TabsNavigationItemAppearance.adorner"]),
        ],
    },
    &ThemeUsage {
        label: "Navigation Sidebar",
        parts: &[
            part("surface", "sidebar", &["default"], &["NavigationSidebarContainerAppearance.background"]),
            part("foreground", "sidebar-foreground", &["default"], &["NavigationSidebarItemAppearance.foreground"]),
            part("active", "sidebar-primary", &["selected"], &["NavigationSidebarItemAppearance.foreground"]),
            part("hover", "sidebar-accent", &["hovered"], &["NavigationSidebarItemAppearance.background"]),
            part("border", "sidebar-border", &["default"], &["NavigationSidebarContainerAppearance.border"]),
        ],
    },
    &ThemeUsage {
        label: "Control Group",
        parts: &[
            part("list background", "muted", &["enabled"], &["ControlGroupListAppearance.background"]),
            part("list border", "border", &["enabled"], &["ControlGroupListAppearance.border"]),
        ],
    },
    &ThemeUsage {
        label: "ListBox",
        parts: &[
            part("list background", "background", &["enabled"], &["ListBoxListAppearance.background"]),
            part("list border", "input", &["enabled"], &["ListBoxListAppearance.border"]),
            part("row hover", "accent", &["hovered"], &["ListBoxRowAppearance.background"]),
            part("focus ring", "ring", &["focused"], &["ListBoxListAppearance.adorner"]),
        ],
    },
    &ThemeUsage {
        label: "Progress",
        parts: &[
            part("track", "muted", &["enabled"], &["ProgressAppearance.track_color"]),
            part("fill", "primary", &["enabled"], &["ProgressAppearance.progress_color"]),
        ],
    },
    &ThemeUsage {
        label: "Selection Panel",
        parts: &[
            part("surface", "popover", &["default"], &["SelectionPanelAppearance.background"]),
            part("border", "border", &["default"], &["SelectionPanelAppearance.border"]),
            part("item hover", "accent", &["hovered"], &["SelectionPanelAppearance.item_hover_background"]),
        ],
    },
];

const fn part(
    part: &'static str,
    token: &'static str,
    states: &'static [&'static str],
    appearance_fields: &'static [&'static str],
) -> ThemePartUsage {
    ThemePartUsage { part, token, states, appearance_fields }
}

#[cfg(test)]
mod tests {
    use super::all_radix_theme_usages;
    use crate::theme::RadixTheme;

    #[test]
    fn radix_usage_tokens_resolve_in_css_theme() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/gallery/tweakcn/retro-arcade.css");
        let theme = RadixTheme::from_css_path(&path).expect("retro-arcade css");
        for usage in all_radix_theme_usages() {
            for part in usage.parts {
                assert!(
                    theme.token_color(part.token).is_ok(),
                    "{} part `{}` missing token `--{}`",
                    usage.label,
                    part.part,
                    part.token
                );
            }
        }
    }
}
