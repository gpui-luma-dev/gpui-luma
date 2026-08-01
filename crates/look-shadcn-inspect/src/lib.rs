mod controls;
mod look;
mod metrics;

#[cfg(test)]
mod test_support;

pub use gpui_luma_look_shadcn::{
    ColorSource, MetricSource, ResolvedColor, ResolvedMetric, ResolvedTypography, TypographySource,
    format_color_source, format_css_style_ref, format_font_weight, format_inspect_css_key,
    format_inspect_metric_provenance, format_inspect_metric_source, format_inspect_provenance,
    format_inspect_typography_provenance, format_inspect_typography_source, format_metric_px, format_typography_px,
};
pub use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnTextFieldStyle};
pub use look::ShadcnInspect;

pub use controls::{
    format_inspect_box_shadow_layer, AccordionContentInspectPalette, AccordionInspectMetrics,
    AccordionTriggerInspectPalette, AutocompleteChromeInspectPalette, AutocompleteInspectMetrics, BadgeInspectMetrics,
    BadgeInspectPalette, ButtonInspectElevation, ButtonInspectElevationLayer, ButtonInspectMetrics,
    ButtonInspectPalette, ButtonInspectTypography, CardInspectMetrics, CheckboxInspectMetrics, CheckboxInspectPalette,
    ContextMenuInspectMetrics, ContextMenuInspectPalette, ControlGroupInspectMetrics, ControlGroupListInspectPalette,
    FloatingMenuInspectMetrics, FloatingMenuInspectPalette, ListBoxInspectMetrics, ListBoxListInspectPalette,
    ListBoxRowInspectPalette, ListViewInspectMetrics, ListViewInspectPalette, ListViewRowInspectPalette,
    NavigationSidebarContainerInspectPalette, NavigationSidebarInspectMetrics, NavigationSidebarItemInspectPalette,
    NavigationSidebarSectionInspectPalette, OverlayWindowInspectMetrics, OverlayWindowInspectPalette,
    PopupMenuInspectMetrics, PopupMenuInspectPalette, PagerInspectMetrics, PagerShellInspectPalette,
    ProgressInspectMetrics, ProgressInspectPalette, RadioButtonInspectMetrics, RadioButtonInspectPalette,
    StepperInspectMetrics, StepperInspectPalette, ResizablePanelsInspectMetrics, ResizablePanelsInspectPalette,
    ScrollbarInspectMetrics, ScrollbarInspectPalette, SelectorInspectMetrics, SelectorInspectPalette,
    SliderInspectMetrics, SliderInspectPalette, SplitViewInspectMetrics, SplitViewInspectPalette, SwitchInspectMetrics,
    SwitchInspectPalette, TabsNavigationInspectMetrics, TabsNavigationItemInspectPalette,
    TabsNavigationListInspectPalette, TextFieldInspectMetrics, TextFieldInspectPalette, ToolbarInspectMetrics,
    ToolbarInspectPalette, TreeViewInspectMetrics, TreeViewRowInspectPalette, ColorChromeInspectSection,
    ColorChromeProfile, COLOR_ARC_CHROME_PROFILES, COLOR_FIELD_CHROME_PROFILES, COLOR_RING_CHROME_PROFILES,
    COLOR_SLIDER_CHROME_PROFILES, inspect_accordion_content_color_palette, inspect_accordion_metrics,
    inspect_accordion_trigger_color_palette, inspect_autocomplete_chrome_color_palette,
    inspect_autocomplete_menu_color_palette, inspect_autocomplete_metrics, inspect_badge_color_palette,
    inspect_badge_metrics, inspect_button_color_palette, inspect_button_elevation, inspect_button_metrics,
    inspect_button_typography, inspect_card_metrics, inspect_checkbox_color_palette, inspect_checkbox_elevation,
    inspect_checkbox_metrics, inspect_context_menu_color_palette, inspect_context_menu_metrics,
    inspect_control_group_list_color_palette, inspect_control_group_metrics, inspect_floating_menu_color_palette,
    inspect_floating_menu_metrics, inspect_list_view_color_palette, inspect_list_view_metrics,
    inspect_list_view_row_color_palette, inspect_listbox_list_color_palette, inspect_listbox_metrics,
    inspect_listbox_row_color_palette, inspect_navigation_sidebar_branch_color_palette,
    inspect_navigation_sidebar_container_color_palette, inspect_navigation_sidebar_item_color_palette,
    inspect_navigation_sidebar_metrics, inspect_navigation_sidebar_section_color_palette,
    inspect_overlay_window_color_palette, inspect_overlay_window_metrics, inspect_pager_metrics,
    inspect_pager_shell_color_palette, inspect_popup_menu_color_palette, inspect_popup_menu_metrics,
    inspect_progress_color_palette, inspect_progress_metrics, inspect_stepper_color_palette, inspect_stepper_metrics,
    inspect_radio_button_color_palette, inspect_radio_button_elevation, inspect_radio_button_metrics,
    inspect_resizable_panels_color_palette, inspect_resizable_panels_metrics, inspect_scrollbar_color_palette,
    inspect_scrollbar_metrics, inspect_selector_color_palette, inspect_selector_metrics, inspect_slider_color_palette,
    inspect_slider_metrics, inspect_split_view_color_palette, inspect_split_view_metrics, inspect_switch_color_palette,
    inspect_switch_elevation, inspect_switch_metrics, inspect_tabs_navigation_item_color_palette,
    inspect_tabs_navigation_list_color_palette, inspect_tabs_navigation_metrics, inspect_textarea_color_palette,
    inspect_textarea_metrics, inspect_textfield_color_palette, inspect_textfield_elevation, inspect_textfield_metrics,
    inspect_toolbar_color_palette, inspect_toolbar_metrics, inspect_tree_view_metrics,
    inspect_tree_view_row_color_palette, inspect_color_chrome_sections,
};
