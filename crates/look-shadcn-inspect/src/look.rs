use luma::controls::button_family::ButtonFamilyRole;
use luma::controls::resizable_panels::ResizeHandleSize;
use luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};
use luma::controls::textarea::TextAreaState;
use luma::controls::textfield::TextFieldState;
use luma::theme::{ControlSize, InteractionState, ThemeMode};

use luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook, ShadcnTextFieldStyle};
use luma_look_shadcn::BadgeVariant;

use crate::controls::{
    AccordionContentInspectPalette, AccordionInspectMetrics, AccordionTriggerInspectPalette,
    AutocompleteChromeInspectPalette, AutocompleteInspectMetrics, BadgeInspectMetrics, BadgeInspectPalette,
    ButtonInspectElevation, ButtonInspectMetrics, ButtonInspectPalette, ButtonInspectTypography, CardInspectMetrics,
    CheckboxInspectMetrics, CheckboxInspectPalette, ContextMenuInspectMetrics, ContextMenuInspectPalette,
    ControlGroupInspectMetrics, ControlGroupListInspectPalette, FloatingMenuInspectMetrics, FloatingMenuInspectPalette,
    ListBoxListInspectPalette, ListBoxRowInspectPalette, TableInspectMetrics, TableInspectPalette,
    TableRowInspectPalette, SidebarContainerInspectPalette, SidebarInspectMetrics, SidebarItemInspectPalette,
    SidebarSectionInspectPalette, OverlayWindowInspectMetrics, OverlayWindowInspectPalette, PagerInspectMetrics,
    PagerShellInspectPalette, PopupMenuInspectMetrics, PopupMenuInspectPalette, ProgressInspectMetrics,
    ProgressInspectPalette, RadioButtonInspectMetrics, RadioButtonInspectPalette, ResizablePanelsInspectMetrics,
    ResizablePanelsInspectPalette, ScrollbarInspectMetrics, ScrollbarInspectPalette, SelectorInspectMetrics,
    SelectorInspectPalette, SliderInspectMetrics, SliderInspectPalette, SplitViewInspectMetrics,
    SplitViewInspectPalette, StepperInspectMetrics, StepperInspectPalette, SwitchInspectMetrics, SwitchInspectPalette,
    TabsInspectMetrics, TabsItemInspectPalette, TabsListInspectPalette, TextFieldInspectMetrics,
    TextFieldInspectPalette, ToolbarInspectMetrics, ToolbarInspectPalette, TreeViewInspectMetrics,
    TreeViewRowInspectPalette, ColorChromeInspectSection, ColorChromeProfile, inspect_accordion_content_color_palette,
    inspect_accordion_metrics, inspect_accordion_trigger_color_palette, inspect_autocomplete_chrome_color_palette,
    inspect_autocomplete_menu_color_palette, inspect_autocomplete_metrics, inspect_badge_color_palette,
    inspect_badge_metrics, inspect_button_color_palette, inspect_button_elevation, inspect_button_metrics,
    inspect_button_typography, inspect_card_metrics, inspect_checkbox_color_palette, inspect_checkbox_elevation,
    inspect_checkbox_metrics, inspect_context_menu_color_palette, inspect_context_menu_metrics,
    inspect_control_group_list_color_palette, inspect_control_group_metrics, inspect_floating_menu_color_palette,
    inspect_floating_menu_metrics, inspect_table_color_palette, inspect_table_metrics, inspect_table_row_color_palette,
    inspect_listbox_list_color_palette, inspect_listbox_row_color_palette, inspect_sidebar_branch_color_palette,
    inspect_sidebar_container_color_palette, inspect_sidebar_item_color_palette, inspect_sidebar_metrics,
    inspect_sidebar_section_color_palette, inspect_overlay_window_color_palette, inspect_overlay_window_metrics,
    inspect_pager_metrics, inspect_pager_shell_color_palette, inspect_popup_menu_color_palette,
    inspect_popup_menu_metrics, inspect_progress_color_palette, inspect_progress_metrics,
    inspect_radio_button_color_palette, inspect_radio_button_elevation, inspect_radio_button_metrics,
    inspect_resizable_panels_color_palette, inspect_resizable_panels_metrics, inspect_scrollbar_color_palette,
    inspect_scrollbar_metrics, inspect_selector_color_palette, inspect_selector_metrics, inspect_slider_color_palette,
    inspect_slider_metrics, inspect_split_view_color_palette, inspect_split_view_metrics,
    inspect_stepper_color_palette, inspect_stepper_metrics, inspect_switch_color_palette, inspect_switch_elevation,
    inspect_switch_metrics, inspect_tabs_item_color_palette, inspect_tabs_list_color_palette, inspect_tabs_metrics,
    inspect_textarea_color_palette, inspect_textarea_metrics, inspect_textfield_color_palette,
    inspect_textfield_elevation, inspect_textfield_metrics, inspect_toolbar_color_palette, inspect_toolbar_metrics,
    inspect_tree_view_metrics, inspect_tree_view_row_color_palette, inspect_color_chrome_sections,
};

/// Inspect-time wrapper around a [`ShadcnLook`] look resolver.
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

    fn mode_tokens(&self) -> std::sync::Arc<luma_look_shadcn::ShadcnModeTokens> {
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

    pub fn inspect_button_elevation(
        &self,
        style: ShadcnButtonStyle,
        role: ButtonFamilyRole,
        state: InteractionState,
    ) -> ButtonInspectElevation {
        inspect_button_elevation(&self.mode_tokens(), self.theme_mode(), style, role, state)
    }

    pub fn inspect_card_metrics(&self, size: ControlSize) -> CardInspectMetrics {
        inspect_card_metrics(self.look, size)
    }

    pub fn inspect_badge_color_palette(&self, variant: BadgeVariant) -> BadgeInspectPalette {
        inspect_badge_color_palette(self.look, variant)
    }

    pub fn inspect_badge_metrics(&self, variant: BadgeVariant, size: ControlSize) -> BadgeInspectMetrics {
        inspect_badge_metrics(&self.mode_tokens(), self.look, variant, size, self.theme_mode())
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

    pub fn inspect_checkbox_elevation(
        &self,
        style: ShadcnButtonStyle,
        checked: bool,
        state: InteractionState,
    ) -> ButtonInspectElevation {
        inspect_checkbox_elevation(&self.mode_tokens(), self.theme_mode(), style, checked, state)
    }

    pub fn inspect_radio_button_metrics(&self, size: ControlSize) -> RadioButtonInspectMetrics {
        inspect_radio_button_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_radio_button_elevation(
        &self,
        style: ShadcnButtonStyle,
        selected: bool,
        state: InteractionState,
    ) -> ButtonInspectElevation {
        inspect_radio_button_elevation(&self.mode_tokens(), self.theme_mode(), style, selected, state)
    }

    pub fn inspect_switch_metrics(&self, size: ControlSize) -> SwitchInspectMetrics {
        inspect_switch_metrics(&self.mode_tokens(), self.theme_mode(), ShadcnButtonStyle::Primary, size)
    }

    pub fn inspect_switch_metrics_for_style(
        &self,
        style: ShadcnButtonStyle,
        size: ControlSize,
    ) -> SwitchInspectMetrics {
        inspect_switch_metrics(&self.mode_tokens(), self.theme_mode(), style, size)
    }

    pub fn inspect_switch_elevation(
        &self,
        style: ShadcnButtonStyle,
        on: bool,
        state: InteractionState,
    ) -> ButtonInspectElevation {
        inspect_switch_elevation(&self.mode_tokens(), self.theme_mode(), style, on, state)
    }

    pub fn inspect_scrollbar_color_palette(
        &self,
        style: ScrollbarStyle,
        state: InteractionState,
    ) -> ScrollbarInspectPalette {
        inspect_scrollbar_color_palette(&self.mode_tokens(), self.theme_mode(), style, state)
    }

    pub fn inspect_scrollbar_metrics(
        &self,
        orientation: ScrollbarOrientation,
        style: ScrollbarStyle,
    ) -> ScrollbarInspectMetrics {
        inspect_scrollbar_metrics(&self.mode_tokens(), self.theme_mode(), orientation, style)
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

    pub fn inspect_stepper_color_palette(&self, enabled: bool) -> StepperInspectPalette {
        inspect_stepper_color_palette(&self.mode_tokens(), self.theme_mode(), enabled)
    }

    pub fn inspect_stepper_metrics(&self) -> StepperInspectMetrics {
        inspect_stepper_metrics(&self.mode_tokens(), self.theme_mode())
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

    pub fn inspect_textfield_elevation(&self, style: ShadcnTextFieldStyle, enabled: bool) -> ButtonInspectElevation {
        inspect_textfield_elevation(&self.mode_tokens(), self.theme_mode(), style, enabled)
    }

    pub fn inspect_floating_menu_color_palette(&self, size: ControlSize) -> FloatingMenuInspectPalette {
        inspect_floating_menu_color_palette(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_floating_menu_metrics(&self, size: ControlSize) -> FloatingMenuInspectMetrics {
        inspect_floating_menu_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_overlay_window_color_palette(
        &self,
        size: ControlSize,
        mode: luma::controls::overlay_window::OverlayWindowMode,
    ) -> OverlayWindowInspectPalette {
        inspect_overlay_window_color_palette(self.look(), size, mode)
    }

    pub fn inspect_overlay_window_metrics(
        &self,
        size: ControlSize,
        mode: luma::controls::overlay_window::OverlayWindowMode,
    ) -> OverlayWindowInspectMetrics {
        inspect_overlay_window_metrics(self.look(), size, mode)
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
        trigger_style: luma::controls::popup_menu::PopupMenuTriggerStyle,
        state: InteractionState,
        size: ControlSize,
    ) -> PopupMenuInspectPalette {
        inspect_popup_menu_color_palette(&self.mode_tokens(), self.theme_mode(), trigger_style, state, size)
    }

    pub fn inspect_popup_menu_metrics(
        &self,
        trigger_style: luma::controls::popup_menu::PopupMenuTriggerStyle,
        size: ControlSize,
    ) -> PopupMenuInspectMetrics {
        inspect_popup_menu_metrics(&self.mode_tokens(), self.theme_mode(), trigger_style, size)
    }

    pub fn inspect_tabs_item_color_palette(&self, active: bool, state: InteractionState) -> TabsItemInspectPalette {
        inspect_tabs_item_color_palette(&self.mode_tokens(), self.theme_mode(), active, state)
    }

    pub fn inspect_tabs_list_color_palette(&self, enabled: bool) -> TabsListInspectPalette {
        inspect_tabs_list_color_palette(&self.mode_tokens(), self.theme_mode(), enabled)
    }

    pub fn inspect_tabs_metrics(&self, size: ControlSize) -> TabsInspectMetrics {
        inspect_tabs_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_toolbar_color_palette(
        &self,
        enabled: bool,
        variant: luma::controls::toolbar::ToolbarVariant,
    ) -> ToolbarInspectPalette {
        inspect_toolbar_color_palette(&self.mode_tokens(), self.theme_mode(), enabled, variant)
    }

    pub fn inspect_toolbar_metrics(&self, size: ControlSize) -> ToolbarInspectMetrics {
        inspect_toolbar_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_pager_shell_color_palette(
        &self,
        enabled: bool,
        style: luma::controls::pager::PagerStyle,
    ) -> PagerShellInspectPalette {
        inspect_pager_shell_color_palette(self.look, enabled, style)
    }

    pub fn inspect_pager_metrics(&self, style: luma::controls::pager::PagerStyle) -> PagerInspectMetrics {
        inspect_pager_metrics(self.look, style)
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

    pub fn inspect_table_color_palette(&self, enabled: bool) -> TableInspectPalette {
        inspect_table_color_palette(&self.mode_tokens(), self.theme_mode(), enabled)
    }

    pub fn inspect_table_row_color_palette(&self, selected: bool, state: InteractionState) -> TableRowInspectPalette {
        inspect_table_row_color_palette(&self.mode_tokens(), self.theme_mode(), selected, state)
    }

    pub fn inspect_table_metrics(&self, size: ControlSize) -> TableInspectMetrics {
        inspect_table_metrics(&self.mode_tokens(), self.theme_mode(), size)
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

    pub fn inspect_tree_view_row_color_palette(&self, state: InteractionState) -> TreeViewRowInspectPalette {
        inspect_tree_view_row_color_palette(&self.mode_tokens(), self.theme_mode(), state)
    }

    pub fn inspect_tree_view_metrics(&self, size: ControlSize) -> TreeViewInspectMetrics {
        inspect_tree_view_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_sidebar_container_color_palette(&self) -> SidebarContainerInspectPalette {
        inspect_sidebar_container_color_palette(&self.mode_tokens(), self.theme_mode())
    }

    pub fn inspect_sidebar_section_color_palette(&self) -> SidebarSectionInspectPalette {
        inspect_sidebar_section_color_palette(&self.mode_tokens(), self.theme_mode())
    }

    pub fn inspect_sidebar_branch_color_palette(&self, state: InteractionState) -> SidebarItemInspectPalette {
        inspect_sidebar_branch_color_palette(&self.mode_tokens(), self.theme_mode(), state)
    }

    pub fn inspect_sidebar_item_color_palette(
        &self,
        selected: bool,
        state: InteractionState,
    ) -> SidebarItemInspectPalette {
        inspect_sidebar_item_color_palette(&self.mode_tokens(), self.theme_mode(), selected, state)
    }

    pub fn inspect_sidebar_metrics(&self, size: ControlSize) -> SidebarInspectMetrics {
        inspect_sidebar_metrics(&self.mode_tokens(), self.theme_mode(), size)
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
        inspect_selector_color_palette(
            &self.mode_tokens(),
            self.theme_mode(),
            luma::controls::selector::SelectorTriggerStyle::Outline,
            state,
            size,
        )
    }

    pub fn inspect_selector_metrics(&self, size: ControlSize) -> SelectorInspectMetrics {
        inspect_selector_metrics(
            &self.mode_tokens(),
            self.theme_mode(),
            luma::controls::selector::SelectorTriggerStyle::Outline,
            size,
        )
    }

    pub fn inspect_control_group_list_color_palette(&self, enabled: bool) -> ControlGroupListInspectPalette {
        inspect_control_group_list_color_palette(&self.mode_tokens(), self.theme_mode(), enabled)
    }

    pub fn inspect_control_group_metrics(&self, size: ControlSize) -> ControlGroupInspectMetrics {
        inspect_control_group_metrics(&self.mode_tokens(), self.theme_mode(), size)
    }

    pub fn inspect_color_chrome_sections(&self, profiles: &[ColorChromeProfile]) -> Vec<ColorChromeInspectSection> {
        inspect_color_chrome_sections(self.look, profiles)
    }
}
