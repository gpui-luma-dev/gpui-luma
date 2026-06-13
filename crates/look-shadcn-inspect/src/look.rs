use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::controls::resizable_panels::ResizeHandleSize;
use gpui_luma::controls::scrollbar::ScrollbarOrientation;
use gpui_luma::controls::textarea::TextAreaState;
use gpui_luma::controls::textfield::TextFieldState;
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook, ShadcnTextFieldStyle};

use crate::controls::{
    AccordionContentInspectPalette, AccordionInspectMetrics, AccordionTriggerInspectPalette,
    AutocompleteChromeInspectPalette, AutocompleteInspectMetrics, ButtonInspectMetrics, ButtonInspectPalette,
    ButtonInspectTypography, CardInspectMetrics, CheckboxInspectMetrics, CheckboxInspectPalette,
    ContextMenuInspectMetrics, ContextMenuInspectPalette, ControlGroupInspectMetrics, ControlGroupListInspectPalette,
    FloatingMenuInspectMetrics, FloatingMenuInspectPalette, ListBoxInspectMetrics, ListBoxListInspectPalette,
    ListBoxRowInspectPalette, ListViewInspectMetrics, ListViewInspectPalette, ListViewRowInspectPalette,
    NavigationSidebarContainerInspectPalette, NavigationSidebarInspectMetrics, NavigationSidebarItemInspectPalette,
    NavigationSidebarSectionInspectPalette, PopupMenuInspectMetrics, PopupMenuInspectPalette, ProgressInspectMetrics,
    ProgressInspectPalette, RadioButtonInspectMetrics, RadioButtonInspectPalette, ResizablePanelsInspectMetrics,
    ResizablePanelsInspectPalette, ScrollbarInspectMetrics, ScrollbarInspectPalette, SelectorInspectMetrics,
    SelectorInspectPalette, SliderInspectMetrics, SliderInspectPalette, SplitViewInspectMetrics,
    SplitViewInspectPalette, SwitchInspectMetrics, SwitchInspectPalette, TabsNavigationInspectMetrics,
    TabsNavigationItemInspectPalette, TabsNavigationListInspectPalette, TextFieldInspectMetrics,
    TextFieldInspectPalette, TreeViewInspectMetrics, TreeViewRowInspectPalette,
    inspect_accordion_content_color_palette, inspect_accordion_metrics, inspect_accordion_trigger_color_palette,
    inspect_autocomplete_chrome_color_palette, inspect_autocomplete_menu_color_palette, inspect_autocomplete_metrics,
    inspect_button_color_palette, inspect_button_metrics, inspect_button_typography, inspect_card_metrics,
    inspect_checkbox_color_palette, inspect_checkbox_metrics, inspect_context_menu_color_palette,
    inspect_context_menu_metrics, inspect_control_group_list_color_palette, inspect_control_group_metrics,
    inspect_floating_menu_color_palette, inspect_floating_menu_metrics, inspect_list_view_color_palette,
    inspect_list_view_metrics, inspect_list_view_row_color_palette, inspect_listbox_list_color_palette,
    inspect_listbox_metrics, inspect_listbox_row_color_palette, inspect_navigation_sidebar_branch_color_palette,
    inspect_navigation_sidebar_container_color_palette, inspect_navigation_sidebar_item_color_palette,
    inspect_navigation_sidebar_metrics, inspect_navigation_sidebar_section_color_palette,
    inspect_popup_menu_color_palette, inspect_popup_menu_metrics, inspect_progress_color_palette,
    inspect_progress_metrics, inspect_radio_button_color_palette, inspect_radio_button_metrics,
    inspect_resizable_panels_color_palette, inspect_resizable_panels_metrics, inspect_scrollbar_color_palette,
    inspect_scrollbar_metrics, inspect_selector_color_palette, inspect_selector_metrics, inspect_slider_color_palette,
    inspect_slider_metrics, inspect_split_view_color_palette, inspect_split_view_metrics, inspect_switch_color_palette,
    inspect_switch_metrics, inspect_tabs_navigation_item_color_palette, inspect_tabs_navigation_list_color_palette,
    inspect_tabs_navigation_metrics, inspect_textarea_color_palette, inspect_textarea_metrics,
    inspect_textfield_color_palette, inspect_textfield_metrics, inspect_tree_view_metrics,
    inspect_tree_view_row_color_palette,
};

/// Inspect-time wrapper around a [`ShadcnLook`] appearance resolver.
pub struct ShadcnInspect<'a> {
    look: &'a ShadcnLook,
}

impl<'a> ShadcnInspect<'a> {
    pub fn new(look: &'a ShadcnLook) -> Self {
        Self { look }
    }

    pub fn look(&self) -> &ShadcnLook {
        self.look
    }

    fn mode_tokens(&self) -> std::sync::Arc<gpui_luma_look_shadcn::ShadcnModeTokens> {
        self.look.mode_tokens()
    }

    fn theme_mode(&self) -> ThemeMode {
        self.look.mode()
    }

    pub fn inspect_button_color_palette(
        &self,
        style: ShadcnButtonStyle,
        role: ButtonFamilyRole,
        state: InteractionState,
    ) -> ButtonInspectPalette {
        inspect_button_color_palette(&self.mode_tokens(), self.theme_mode(), style, role, state)
    }

    pub fn inspect_button_metrics(
        &self,
        style: ShadcnButtonStyle,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonInspectMetrics {
        inspect_button_metrics(&self.mode_tokens(), self.theme_mode(), style, role, size, state)
    }

    pub fn inspect_button_typography(&self) -> ButtonInspectTypography {
        inspect_button_typography(&self.mode_tokens(), self.theme_mode())
    }

    pub fn inspect_card_metrics(&self, size: ControlSize) -> CardInspectMetrics {
        inspect_card_metrics(self.look, size)
    }

    pub fn inspect_checkbox_color_palette(
        &self,
        style: ShadcnButtonStyle,
        checked: bool,
        state: InteractionState,
    ) -> CheckboxInspectPalette {
        inspect_checkbox_color_palette(&self.mode_tokens(), self.theme_mode(), style, checked, state)
    }

    pub fn inspect_textfield_color_palette(
        &self,
        style: ShadcnTextFieldStyle,
        state: TextFieldState,
        enabled: bool,
    ) -> TextFieldInspectPalette {
        inspect_textfield_color_palette(&self.mode_tokens(), self.theme_mode(), style, state, enabled)
    }

    pub fn inspect_radio_button_color_palette(
        &self,
        style: ShadcnButtonStyle,
        selected: bool,
        state: InteractionState,
    ) -> RadioButtonInspectPalette {
        inspect_radio_button_color_palette(&self.mode_tokens(), self.theme_mode(), style, selected, state)
    }

    pub fn inspect_switch_color_palette(
        &self,
        style: ShadcnButtonStyle,
        on: bool,
        state: InteractionState,
    ) -> SwitchInspectPalette {
        inspect_switch_color_palette(&self.mode_tokens(), self.theme_mode(), style, on, state)
    }

    pub fn inspect_checkbox_metrics(&self, size: ControlSize) -> CheckboxInspectMetrics {
        inspect_checkbox_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_radio_button_metrics(&self, size: ControlSize) -> RadioButtonInspectMetrics {
        inspect_radio_button_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_switch_metrics(&self, size: ControlSize) -> SwitchInspectMetrics {
        inspect_switch_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_scrollbar_color_palette(
        &self,
        orientation: ScrollbarOrientation,
        state: InteractionState,
    ) -> ScrollbarInspectPalette {
        inspect_scrollbar_color_palette(&self.mode_tokens(), self.theme_mode(), orientation, state)
    }

    pub fn inspect_scrollbar_metrics(&self, orientation: ScrollbarOrientation) -> ScrollbarInspectMetrics {
        inspect_scrollbar_metrics(&self.mode_tokens(), self.theme_mode(), orientation)
    }

    pub fn inspect_slider_color_palette(&self, state: InteractionState) -> SliderInspectPalette {
        inspect_slider_color_palette(&self.mode_tokens(), self.theme_mode(), state)
    }

    pub fn inspect_slider_metrics(&self) -> SliderInspectMetrics {
        inspect_slider_metrics(&self.mode_tokens(), self.theme_mode())
    }

    pub fn inspect_progress_color_palette(&self, enabled: bool) -> ProgressInspectPalette {
        inspect_progress_color_palette(&self.mode_tokens(), self.theme_mode(), enabled)
    }

    pub fn inspect_progress_metrics(&self) -> ProgressInspectMetrics {
        inspect_progress_metrics(&self.mode_tokens(), self.theme_mode())
    }

    pub fn inspect_textarea_color_palette(
        &self,
        style: ShadcnTextFieldStyle,
        state: TextAreaState,
        enabled: bool,
    ) -> TextFieldInspectPalette {
        inspect_textarea_color_palette(&self.mode_tokens(), self.theme_mode(), style, state, enabled)
    }

    pub fn inspect_textarea_metrics(&self, size: ControlSize) -> TextFieldInspectMetrics {
        inspect_textarea_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_textfield_metrics(&self, size: ControlSize) -> TextFieldInspectMetrics {
        inspect_textfield_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_floating_menu_color_palette(&self, size: ControlSize) -> FloatingMenuInspectPalette {
        inspect_floating_menu_color_palette(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_floating_menu_metrics(&self, size: ControlSize) -> FloatingMenuInspectMetrics {
        inspect_floating_menu_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_context_menu_color_palette(
        &self,
        state: InteractionState,
        size: ControlSize,
    ) -> ContextMenuInspectPalette {
        inspect_context_menu_color_palette(&self.mode_tokens(), self.theme_mode(), state, size)
    }

    pub fn inspect_context_menu_metrics(&self, size: ControlSize) -> ContextMenuInspectMetrics {
        inspect_context_menu_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_popup_menu_color_palette(
        &self,
        state: InteractionState,
        size: ControlSize,
    ) -> PopupMenuInspectPalette {
        inspect_popup_menu_color_palette(&self.mode_tokens(), self.theme_mode(), state, size)
    }

    pub fn inspect_popup_menu_metrics(&self, size: ControlSize) -> PopupMenuInspectMetrics {
        inspect_popup_menu_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_tabs_navigation_item_color_palette(
        &self,
        active: bool,
        state: InteractionState,
    ) -> TabsNavigationItemInspectPalette {
        inspect_tabs_navigation_item_color_palette(&self.mode_tokens(), self.theme_mode(), active, state)
    }

    pub fn inspect_tabs_navigation_list_color_palette(&self, enabled: bool) -> TabsNavigationListInspectPalette {
        inspect_tabs_navigation_list_color_palette(&self.mode_tokens(), self.theme_mode(), enabled)
    }

    pub fn inspect_tabs_navigation_metrics(&self, size: ControlSize) -> TabsNavigationInspectMetrics {
        inspect_tabs_navigation_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_split_view_color_palette(&self, enabled: bool) -> SplitViewInspectPalette {
        inspect_split_view_color_palette(&self.mode_tokens(), self.theme_mode(), enabled)
    }

    pub fn inspect_split_view_metrics(&self) -> SplitViewInspectMetrics {
        inspect_split_view_metrics()
    }

    pub fn inspect_resizable_panels_color_palette(&self, state: InteractionState) -> ResizablePanelsInspectPalette {
        inspect_resizable_panels_color_palette(&self.mode_tokens(), self.theme_mode(), state)
    }

    pub fn inspect_resizable_panels_metrics(&self, handle_size: ResizeHandleSize) -> ResizablePanelsInspectMetrics {
        inspect_resizable_panels_metrics(handle_size)
    }

    pub fn inspect_list_view_color_palette(&self, enabled: bool) -> ListViewInspectPalette {
        inspect_list_view_color_palette(&self.mode_tokens(), self.theme_mode(), enabled)
    }

    pub fn inspect_list_view_row_color_palette(
        &self,
        selected: bool,
        state: InteractionState,
    ) -> ListViewRowInspectPalette {
        inspect_list_view_row_color_palette(&self.mode_tokens(), self.theme_mode(), selected, state)
    }

    pub fn inspect_list_view_metrics(&self, size: ControlSize) -> ListViewInspectMetrics {
        inspect_list_view_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_accordion_trigger_color_palette(&self, state: InteractionState) -> AccordionTriggerInspectPalette {
        inspect_accordion_trigger_color_palette(&self.mode_tokens(), self.theme_mode(), state)
    }

    pub fn inspect_accordion_content_color_palette(&self, expanded: bool) -> AccordionContentInspectPalette {
        inspect_accordion_content_color_palette(&self.mode_tokens(), self.theme_mode(), expanded)
    }

    pub fn inspect_accordion_metrics(&self, size: ControlSize) -> AccordionInspectMetrics {
        inspect_accordion_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_listbox_list_color_palette(&self, enabled: bool, focused: bool) -> ListBoxListInspectPalette {
        inspect_listbox_list_color_palette(&self.mode_tokens(), self.theme_mode(), enabled, focused)
    }

    pub fn inspect_listbox_row_color_palette(&self, state: InteractionState) -> ListBoxRowInspectPalette {
        inspect_listbox_row_color_palette(&self.mode_tokens(), self.theme_mode(), state)
    }

    pub fn inspect_listbox_metrics(&self, size: ControlSize) -> ListBoxInspectMetrics {
        inspect_listbox_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_tree_view_row_color_palette(&self, state: InteractionState) -> TreeViewRowInspectPalette {
        inspect_tree_view_row_color_palette(&self.mode_tokens(), self.theme_mode(), state)
    }

    pub fn inspect_tree_view_metrics(&self, size: ControlSize) -> TreeViewInspectMetrics {
        inspect_tree_view_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_navigation_sidebar_container_color_palette(&self) -> NavigationSidebarContainerInspectPalette {
        inspect_navigation_sidebar_container_color_palette(&self.mode_tokens(), self.theme_mode())
    }

    pub fn inspect_navigation_sidebar_section_color_palette(&self) -> NavigationSidebarSectionInspectPalette {
        inspect_navigation_sidebar_section_color_palette(&self.mode_tokens(), self.theme_mode())
    }

    pub fn inspect_navigation_sidebar_branch_color_palette(
        &self,
        state: InteractionState,
    ) -> NavigationSidebarItemInspectPalette {
        inspect_navigation_sidebar_branch_color_palette(&self.mode_tokens(), self.theme_mode(), state)
    }

    pub fn inspect_navigation_sidebar_item_color_palette(
        &self,
        selected: bool,
        state: InteractionState,
    ) -> NavigationSidebarItemInspectPalette {
        inspect_navigation_sidebar_item_color_palette(&self.mode_tokens(), self.theme_mode(), selected, state)
    }

    pub fn inspect_navigation_sidebar_metrics(&self, size: ControlSize) -> NavigationSidebarInspectMetrics {
        inspect_navigation_sidebar_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_autocomplete_chrome_color_palette(&self) -> AutocompleteChromeInspectPalette {
        inspect_autocomplete_chrome_color_palette(&self.mode_tokens(), self.theme_mode())
    }

    pub fn inspect_autocomplete_menu_color_palette(&self, size: ControlSize) -> FloatingMenuInspectPalette {
        inspect_autocomplete_menu_color_palette(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_autocomplete_metrics(&self, size: ControlSize) -> AutocompleteInspectMetrics {
        inspect_autocomplete_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_selector_color_palette(&self, state: InteractionState, size: ControlSize) -> SelectorInspectPalette {
        inspect_selector_color_palette(&self.mode_tokens(), self.theme_mode(), state, size)
    }

    pub fn inspect_selector_metrics(&self, size: ControlSize) -> SelectorInspectMetrics {
        inspect_selector_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_control_group_list_color_palette(&self, enabled: bool) -> ControlGroupListInspectPalette {
        inspect_control_group_list_color_palette(&self.mode_tokens(), self.theme_mode(), enabled)
    }

    pub fn inspect_control_group_metrics(&self, size: ControlSize) -> ControlGroupInspectMetrics {
        inspect_control_group_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }
}
