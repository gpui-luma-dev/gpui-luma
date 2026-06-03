//! Radix CSS token usage metadata for gallery sidebars and theme docs.

use crate::theme::{ThemePartUsage, ThemeUsage};

pub fn all_radix_theme_usages() -> &'static [&'static ThemeUsage] {
    RADIX_THEME_USAGES
}

const TEXTFIELD_PARTS: &[ThemePartUsage] = &[
    part("background", "background", &["standard default"], &["TextFieldPalette.background"]),
    part("border", "input", &["standard default"], &["TextFieldPalette.border"]),
    part("foreground", "foreground", &["standard default"], &["TextFieldPalette.foreground"]),
    part("placeholder", "muted-foreground", &["standard default"], &["TextFieldPalette.placeholder"]),
    part("selection", "primary", &["standard focused"], &["TextFieldPalette.selection_background"]),
    part("focus ring", "ring", &["standard focused"], &["TextFieldPalette.focus_ring"]),
    part("ghost hover", "accent", &["ghost hovered"], &["TextFieldPalette.background"]),
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
            part("checked fill", "primary", &["checked"], &["CheckboxPalette.indicator_background"]),
            part("checkmark", "primary-foreground", &["checked"], &["CheckboxPalette.checkmark_color"]),
            part("border", "input", &["unchecked"], &["CheckboxPalette.indicator_border"]),
            part("focus ring", "ring", &["focused"], &["CheckboxPalette.adorner"]),
        ],
    },
    &ThemeUsage {
        label: "Radio Button",
        parts: &[
            part("selected fill", "primary", &["selected"], &["RadioButtonPalette.indicator_border"]),
            part("dot", "primary-foreground", &["selected"], &["RadioButtonPalette.dot_color"]),
            part("border", "input", &["unselected"], &["RadioButtonPalette.indicator_border"]),
            part("focus ring", "ring", &["focused"], &["RadioButtonPalette.adorner"]),
        ],
    },
    &ThemeUsage {
        label: "Switch",
        parts: &[
            part("off track", "input", &["off"], &["SwitchPalette.track_background"]),
            part("track border", "border", &["off", "disabled"], &["SwitchPalette.track_border"]),
            part("on track", "primary", &["on"], &["SwitchPalette.track_background", "SwitchPalette.track_border"]),
            part("off thumb", "background", &["off"], &["SwitchPalette.thumb_background"]),
            part("thumb border", "border", &["off"], &["SwitchPalette.thumb_border"]),
            part(
                "on thumb",
                "primary-foreground",
                &["on"],
                &["SwitchPalette.thumb_background", "SwitchPalette.thumb_border"],
            ),
            part("disabled thumb", "muted-foreground", &["disabled"], &["SwitchPalette.thumb_background"]),
            part("disabled thumb border", "muted", &["disabled"], &["SwitchPalette.thumb_border"]),
            part("focus ring", "ring", &["focused"], &["SwitchPalette.adorner"]),
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
            part("trigger hover", "accent", &["hovered"], &["PopupMenuPalette.trigger_background"]),
            part("menu surface", "popover", &["open"], &["FloatingMenuAppearance.background"]),
        ],
    },
    &ThemeUsage {
        label: "Selector",
        parts: &[
            part("trigger background", "background", &["default"], &["SelectorPalette.trigger_background"]),
            part("trigger border", "input", &["default"], &["SelectorPalette.trigger_border"]),
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
        label: "Accordion",
        parts: &[
            part("trigger label", "foreground", &["default"], &["AccordionPalette.foreground"]),
            part("trigger hover", "accent", &["hovered"], &["AccordionPalette.background"]),
            part("disabled label", "muted-foreground", &["disabled"], &["AccordionPalette.foreground"]),
            part("chevron", "muted-foreground", &["default"], &["AccordionPalette.chevron_color"]),
            part("item border", "border", &["default"], &["AccordionPalette.border_color"]),
            part("content label", "foreground", &["expanded"], &["AccordionContentPalette.foreground"]),
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
        label: "TreeView",
        parts: &[
            part("row label", "sidebar-foreground", &["default"], &["TreeViewPalette.foreground"]),
            part("row hover", "sidebar-accent", &["hovered"], &["TreeViewPalette.background"]),
            part("row pressed", "accent", &["pressed"], &["TreeViewPalette.background"]),
            part("disabled label", "muted-foreground", &["disabled"], &["TreeViewPalette.foreground"]),
            part("icon", "sidebar-foreground", &["default"], &["TreeViewPalette.icon_color"]),
            part("chevron", "sidebar-foreground", &["default"], &["TreeViewPalette.chevron_color"]),
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
            part("row hover", "accent", &["hovered"], &["ListBoxRowPalette.background"]),
            part("focus ring", "ring", &["focused"], &["ListBoxListAppearance.adorner"]),
        ],
    },
    &ThemeUsage {
        label: "ListView",
        parts: &[
            part("list background", "background", &["enabled"], &["ListViewAppearance.background"]),
            part("list border", "input", &["enabled"], &["ListViewAppearance.border"]),
            part("header background", "muted", &["enabled"], &["ListViewAppearance.header_background"]),
            part("header label", "muted-foreground", &["enabled"], &["ListViewAppearance.header_label_color"]),
            part("row hover", "accent", &["hovered"], &["ListViewRowPalette.background"]),
            part("row active", "muted", &["focused"], &["ListViewRowPalette.background"]),
            part("row divider", "border", &["enabled"], &["ListViewRowPalette.divider"]),
            part("selected row", "muted", &["selected"], &["ListViewRowPalette.background"]),
            part("selected label", "foreground", &["selected"], &["ListViewRowPalette.label_color"]),
        ],
    },
    &ThemeUsage {
        label: "ResizablePanels",
        parts: &[
            part("panel border", "border", &["enabled"], &["ResizablePanelsAppearance.border"]),
            part("divider line", "border", &["enabled"], &["ResizablePanelsAppearance.divider"]),
            part("handle grip", "border", &["enabled"], &["ResizablePanelsAppearance.grip"]),
            part("grip emphasis", "accent", &["enabled"], &["ResizablePanelsAppearance.grip_emphasis"]),
            part("disabled divider", "muted-foreground", &["disabled"], &["ResizablePanelsAppearance.divider"]),
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
