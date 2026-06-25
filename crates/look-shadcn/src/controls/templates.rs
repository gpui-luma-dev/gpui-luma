use std::sync::Arc;

use gpui_luma::controls::autocomplete::AutocompleteTextBoxTheme;
use gpui_luma::controls::button_family::{
    ButtonFamilyPalette, ButtonFamilyRole, ButtonFamilyTheme, button_family_effective_border,
};
use gpui_luma::controls::card::{CardTemplate, CardTheme, ThemedCardTemplate};
use gpui_luma::controls::overlay_window::{
    OverlayWindowLook, OverlayWindowMode, OverlayWindowTemplate, OverlayWindowTheme, ThemedOverlayWindowTemplate,
};
use gpui_luma::controls::checkbox::{CheckboxTheme, ThemedCheckboxTemplate};
use gpui_luma::controls::command::button::{ButtonTemplate, DefaultButtonTemplate};
use gpui_luma::controls::context_menu::{ContextMenuTheme, ThemedContextMenuTemplate};
use gpui_luma::controls::control_group::{
    ControlGroupItemLike, ControlGroupTemplate, ControlGroupTheme, control_group_template_with_theme,
};
use gpui_luma::controls::dock_splitter::DockSplitterTheme;
use gpui_luma::controls::radio_group::{RadioGroupLayout, radio_group_buttons_template};
use gpui_luma::controls::floating_menu::FloatingMenuTheme;
use gpui_luma::controls::list_view::{ListViewTheme, list_view_template_with_theme};
use gpui_luma::controls::navigation_sidebar::{
    NavigationSidebarTemplate, NavigationSidebarTheme, ThemedNavigationSidebarTemplate,
};
use gpui_luma::controls::listbox::{ListBoxTheme, listbox_template_with_theme};
use gpui_luma::controls::pager::{PagerTemplate, PagerTheme, ThemedPagerTemplate};
use gpui_luma::controls::popup_menu::{PopupMenuTheme, ThemedPopupMenuTemplate};
use gpui_luma::controls::progress::{ProgressTheme, ThemedProgressTemplate};
use gpui_luma::controls::radio_button::{RadioButtonTheme, ThemedRadioButtonTemplate};
use gpui_luma::controls::scrollbar::{ScrollbarTheme, ThemedScrollbarTemplate};
use gpui_luma::controls::selector::{SelectorTheme, ThemedSelectorTemplate};
use gpui_luma::controls::selector_panel::default_selector_items_template;
use gpui_luma::controls::selection_panel::SelectionPanelLookProvider;
use gpui_luma::controls::resizable_panels::ResizablePanelsTheme;
use gpui_luma::controls::split_view::SplitViewTheme;
use gpui_luma::controls::slider::{
    SliderTemplate, SliderTheme, ThemedAngularDialTemplate, ThemedCircularRingTemplate, ThemedSliderTemplate,
};
use gpui_luma::controls::switch::{SwitchTheme, ThemedSwitchTemplate};
use gpui_luma::controls::accordion::{AccordionTemplate, AccordionTheme, ThemedAccordionTemplate};
use gpui_luma::controls::tree_view::{TreeViewTemplate, TreeViewTheme, ThemedTreeViewTemplate};
use gpui_luma::controls::tabs_navigation::{TabsNavigationTemplate, TabsNavigationTheme, ThemedTabsNavigationTemplate};
use gpui_luma::controls::textarea::{TextAreaTheme, ThemedTextAreaTemplate};
use gpui_luma::controls::textfield::{TextFieldState, TextFieldTheme, TextFieldVariant, ThemedTextFieldTemplate};
use gpui::Styled;

use gpui_luma::theme::{ControlSize, InteractionState};

use super::autocomplete::autocomplete_textbox_look;
use super::checkbox::checkbox_look;
use super::card::card_look;
use super::control_group::control_group_list_look;
use super::context_menu::context_menu_look;
use super::overlay_window::overlay_window_look;
use super::floating_menu::floating_menu_look;
use super::list_view::{list_view_look, list_view_row_palette};
use super::navigation_sidebar::{
    navigation_sidebar_branch_look, navigation_sidebar_container_look, navigation_sidebar_item_look,
    navigation_sidebar_section_look,
};
use super::listbox::{listbox_list_look, listbox_row_palette};
use super::pager::pager_look;
use super::progress::progress_look;
use super::radio::radio_button_look;
use super::scrollbar::scrollbar_look;
use super::selection_panel::selection_panel_look;
use super::slider::slider_look;
use super::switch::switch_look;
use super::accordion::{accordion_content_palette, accordion_trigger_palette};
use super::resizable_panels::resizable_panels_look;
use super::split_view::split_view_look;
use super::tree_view::tree_view_row_palette;
use super::tabs_navigation::{tabs_navigation_item_look, tabs_navigation_list_look};
use super::button::{button_look, button_palette};
use crate::look_context::LookContext;
use super::button::ShadcnButtonStyle;
use crate::look::ShadcnLook;

struct RadixStyledButtonFamilyTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl ButtonFamilyTheme for RadixStyledButtonFamilyTheme {
    fn resolve(&self, role: ButtonFamilyRole, size: ControlSize, state: InteractionState) -> ButtonFamilyPalette {
        let tokens = self.theme.mode_tokens();
        let stylesheet = self.theme.stylesheet();
        let ctx = LookContext::new(tokens.as_ref(), self.theme.mode(), state);
        button_palette(&ctx, stylesheet.as_ref(), self.style, role, size)
    }

    fn resolve_look(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
        _scale: &gpui_luma::theme::StandardBoxScale,
        _pill_radius: f32,
    ) -> Option<gpui_luma::controls::button_family::ButtonFamilyLook> {
        let tokens = self.theme.mode_tokens();
        Some(button_look(tokens.as_ref(), self.theme.mode(), self.style, role, size, state))
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn styled_button_family_theme(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn ButtonFamilyTheme> {
    Arc::new(RadixStyledButtonFamilyTheme { theme: theme.as_ref().clone(), style })
}

pub fn button_family_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ButtonFamilyTheme> {
    styled_button_family_theme(theme, ShadcnButtonStyle::Secondary)
}

pub fn button_template(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<()>> {
    Arc::new(DefaultButtonTemplate::new(styled_button_family_theme(theme, style)))
}

struct RadixStyledSwitchTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl SwitchTheme for RadixStyledSwitchTheme {
    fn resolve(&self, on: bool, state: InteractionState) -> gpui_luma::controls::switch::SwitchPalette {
        let tokens = self.theme.mode_tokens();
        switch_look(tokens.as_ref(), self.theme.mode(), self.style, on, state)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

struct RadixStyledCheckboxTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl CheckboxTheme for RadixStyledCheckboxTheme {
    fn resolve(&self, checked: bool, state: InteractionState) -> gpui_luma::controls::checkbox::CheckboxPalette {
        let tokens = self.theme.mode_tokens();
        checkbox_look(tokens.as_ref(), self.style, checked, state)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

struct RadixStyledRadioButtonTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl RadioButtonTheme for RadixStyledRadioButtonTheme {
    fn resolve(
        &self,
        selected: bool,
        state: InteractionState,
    ) -> gpui_luma::controls::radio_button::RadioButtonPalette {
        let tokens = self.theme.mode_tokens();
        radio_button_look(tokens.as_ref(), self.style, selected, state)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn checkbox_theme(theme: Arc<ShadcnLook>) -> Arc<dyn CheckboxTheme> {
    checkbox_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn checkbox_theme_with_style(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn CheckboxTheme> {
    Arc::new(RadixStyledCheckboxTheme { theme: theme.as_ref().clone(), style })
}

pub fn switch_theme(theme: Arc<ShadcnLook>) -> Arc<dyn SwitchTheme> {
    switch_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn switch_theme_with_style(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn SwitchTheme> {
    Arc::new(RadixStyledSwitchTheme { theme: theme.as_ref().clone(), style })
}

pub fn radio_button_theme(theme: Arc<ShadcnLook>) -> Arc<dyn RadioButtonTheme> {
    radio_button_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn radio_button_theme_with_style(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn RadioButtonTheme> {
    Arc::new(RadixStyledRadioButtonTheme { theme: theme.as_ref().clone(), style })
}

pub fn switch_template(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(ThemedSwitchTemplate::new(switch_theme_with_style(theme, style)))
}

pub fn checkbox_template(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(ThemedCheckboxTemplate::new(checkbox_theme_with_style(theme, style)))
}

pub fn radio_button_template(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(ThemedRadioButtonTemplate::new(radio_button_theme_with_style(theme, style)))
}

pub fn slider_theme(theme: Arc<ShadcnLook>) -> Arc<dyn SliderTheme> {
    Arc::new(RadixSliderTheme { theme: theme.as_ref().clone() })
}

pub fn dock_splitter_theme(theme: Arc<ShadcnLook>) -> Arc<dyn DockSplitterTheme> {
    Arc::new(RadixDockSplitterTheme { theme: theme.as_ref().clone() })
}

pub fn resizable_panels_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ResizablePanelsTheme> {
    Arc::new(RadixResizablePanelsTheme { theme: theme.as_ref().clone() })
}

pub fn split_view_theme(theme: Arc<ShadcnLook>) -> Arc<dyn SplitViewTheme> {
    Arc::new(RadixSplitViewTheme { theme: theme.as_ref().clone() })
}

pub fn scrollbar_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ScrollbarTheme> {
    Arc::new(RadixScrollbarTheme { theme: theme.as_ref().clone() })
}

pub fn selector_theme(theme: Arc<ShadcnLook>) -> Arc<dyn SelectorTheme> {
    Arc::new(RadixSelectorTheme { theme: theme.as_ref().clone() })
}

pub fn popup_menu_theme(theme: Arc<ShadcnLook>) -> Arc<dyn PopupMenuTheme> {
    Arc::new(RadixPopupMenuTheme { theme: theme.as_ref().clone() })
}

struct RadixDockSplitterTheme {
    theme: ShadcnLook,
}

impl DockSplitterTheme for RadixDockSplitterTheme {
    fn resolve(&self, enabled: bool) -> gpui_luma::controls::dock_splitter::DockSplitterLook {
        let tokens = self.theme.mode_tokens();
        let border = tokens.palette.border_default;
        let disabled = tokens.palette.disabled_foreground;

        gpui_luma::controls::dock_splitter::DockSplitterLook {
            line_color: if enabled { border } else { disabled },
            hover_color: if enabled { border } else { disabled },
            thumb_color: if enabled {
                tokens.palette.primary.background
            } else {
                disabled
            },
            hit_target_px: 8.0,
            visible_line_px: 1.0,
        }
    }
}

struct RadixResizablePanelsTheme {
    theme: ShadcnLook,
}

impl ResizablePanelsTheme for RadixResizablePanelsTheme {
    fn resolve(&self, state: InteractionState) -> gpui_luma::controls::resizable_panels::ResizablePanelsLook {
        let tokens = self.theme.mode_tokens();
        resizable_panels_look(tokens.as_ref(), self.theme.mode(), state)
    }
}

struct RadixSplitViewTheme {
    theme: ShadcnLook,
}

impl SplitViewTheme for RadixSplitViewTheme {
    fn resolve(&self, hovered: bool, enabled: bool) -> gpui_luma::controls::split_view::SplitViewLook {
        let tokens = self.theme.mode_tokens();
        split_view_look(tokens.as_ref(), self.theme.mode(), hovered, enabled)
    }
}

struct RadixSliderTheme {
    theme: ShadcnLook,
}

impl SliderTheme for RadixSliderTheme {
    fn resolve(
        &self,
        size: gpui_luma::theme::ControlSize,
        thumb_size: Option<gpui_luma::controls::slider::SliderThumbSize>,
        state: InteractionState,
    ) -> gpui_luma::controls::slider::SliderLook {
        let tokens = self.theme.mode_tokens();
        slider_look(tokens.as_ref(), self.theme.mode(), size, thumb_size, state)
    }
}

struct RadixScrollbarTheme {
    theme: ShadcnLook,
}

impl ScrollbarTheme for RadixScrollbarTheme {
    fn resolve(
        &self,
        state: InteractionState,
        orientation: gpui_luma::controls::scrollbar::ScrollbarOrientation,
    ) -> gpui_luma::controls::scrollbar::ScrollbarLook {
        let tokens = self.theme.mode_tokens();
        scrollbar_look(tokens.as_ref(), state, orientation)
    }
}

pub fn slider_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::slider::SliderTemplate> {
    Arc::new(ThemedSliderTemplate::new(Arc::new(RadixSliderTheme { theme: theme.as_ref().clone() })))
}

pub fn slider_angular_template(theme: Arc<ShadcnLook>) -> Arc<dyn SliderTemplate> {
    Arc::new(ThemedAngularDialTemplate::new(Arc::new(RadixSliderTheme { theme: theme.as_ref().clone() })))
}

pub fn slider_circular_ring_template(theme: Arc<ShadcnLook>) -> Arc<dyn SliderTemplate> {
    Arc::new(ThemedCircularRingTemplate::new(Arc::new(RadixSliderTheme { theme: theme.as_ref().clone() })))
}

pub fn scrollbar_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::scrollbar::ScrollbarTemplate> {
    Arc::new(ThemedScrollbarTemplate::new(Arc::new(RadixScrollbarTheme { theme: theme.as_ref().clone() })))
}

struct RadixFloatingMenuTheme {
    theme: ShadcnLook,
}

impl FloatingMenuTheme for RadixFloatingMenuTheme {
    fn resolve(&self) -> gpui_luma::controls::floating_menu::FloatingMenuLook {
        let tokens = self.theme.mode_tokens();
        floating_menu_look(tokens.as_ref(), self.theme.mode(), gpui_luma::theme::ControlSize::Md)
    }
}

struct RadixPopupMenuTheme {
    theme: ShadcnLook,
}

impl PopupMenuTheme for RadixPopupMenuTheme {
    fn resolve(
        &self,
        trigger_style: gpui_luma::controls::popup_menu::PopupMenuTriggerStyle,
        state: InteractionState,
    ) -> gpui_luma::controls::popup_menu::PopupMenuPalette {
        let tokens = self.theme.mode_tokens();
        super::popup_menu::popup_menu_palette(tokens.as_ref(), self.theme.mode(), trigger_style, state)
    }

    fn resolve_look(
        &self,
        trigger_style: gpui_luma::controls::popup_menu::PopupMenuTriggerStyle,
        state: InteractionState,
        scale_factor: f32,
        cx: &mut gpui::App,
    ) -> gpui_luma::controls::popup_menu::PopupMenuLook {
        let tokens = self.theme.mode_tokens();
        super::popup_menu::popup_menu_look(
            tokens.as_ref(),
            self.theme.mode(),
            trigger_style,
            state,
            scale_factor,
            cx,
        )
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

struct RadixContextMenuTheme {
    theme: ShadcnLook,
}

impl ContextMenuTheme for RadixContextMenuTheme {
    fn resolve(&self, state: InteractionState) -> gpui_luma::controls::context_menu::ContextMenuLook {
        let tokens = self.theme.mode_tokens();
        context_menu_look(tokens.as_ref(), self.theme.mode(), state)
    }
}

pub fn floating_menu_theme(theme: Arc<ShadcnLook>) -> Arc<dyn FloatingMenuTheme> {
    Arc::new(RadixFloatingMenuTheme { theme: theme.as_ref().clone() })
}

pub fn popup_menu_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::popup_menu::PopupMenuTemplate> {
    Arc::new(ThemedPopupMenuTemplate::new(Arc::new(RadixPopupMenuTheme { theme: theme.as_ref().clone() })))
}

pub fn context_menu_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ContextMenuTheme> {
    Arc::new(RadixContextMenuTheme { theme: theme.as_ref().clone() })
}

pub fn context_menu_template(
    theme: Arc<ShadcnLook>,
) -> Arc<dyn gpui_luma::controls::context_menu::ContextMenuTemplate> {
    Arc::new(ThemedContextMenuTemplate::new(context_menu_theme(Arc::clone(&theme))))
}

struct RadixSelectorTheme {
    theme: ShadcnLook,
}

impl SelectorTheme for RadixSelectorTheme {
    fn resolve(&self, state: InteractionState) -> gpui_luma::controls::selector::SelectorPalette {
        let tokens = self.theme.mode_tokens();
        super::selector::selector_palette(tokens.as_ref(), self.theme.mode(), state)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

struct RadixTextFieldTheme {
    theme: ShadcnLook,
}

impl TextFieldTheme for RadixTextFieldTheme {
    fn resolve(
        &self,
        _variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
    ) -> gpui_luma::controls::textfield::TextFieldPalette {
        let tokens = self.theme.mode_tokens();
        super::textfield::textfield_palette(
            tokens.as_ref(),
            self.theme.mode(),
            super::textfield::ShadcnTextFieldStyle::Surface,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

struct RadixAutocompleteTextBoxTheme {
    theme: ShadcnLook,
}

impl AutocompleteTextBoxTheme for RadixAutocompleteTextBoxTheme {
    fn resolve(&self) -> gpui_luma::controls::autocomplete::AutocompleteTextBoxLook {
        let tokens = self.theme.mode_tokens();
        autocomplete_textbox_look(tokens.as_ref(), self.theme.mode(), ControlSize::Md)
    }
}

pub fn selector_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::selector::SelectorTemplate> {
    Arc::new(ThemedSelectorTemplate::new(
        Arc::new(RadixSelectorTheme { theme: theme.as_ref().clone() }),
        default_selector_items_template(),
    ))
}

pub fn textfield_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::textfield::TextFieldTemplate> {
    Arc::new(ThemedTextFieldTemplate::new(Arc::new(RadixTextFieldTheme { theme: theme.as_ref().clone() })))
}

pub fn textfield_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TextFieldTheme> {
    Arc::new(RadixTextFieldTheme { theme: theme.as_ref().clone() })
}

struct RadixSoftTextFieldTheme {
    theme: ShadcnLook,
}

impl TextFieldTheme for RadixSoftTextFieldTheme {
    fn resolve(
        &self,
        _variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
    ) -> gpui_luma::controls::textfield::TextFieldPalette {
        let tokens = self.theme.mode_tokens();
        super::textfield::textfield_palette(
            tokens.as_ref(),
            self.theme.mode(),
            super::textfield::ShadcnTextFieldStyle::Soft,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn soft_textfield_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TextFieldTheme> {
    Arc::new(RadixSoftTextFieldTheme { theme: theme.as_ref().clone() })
}

struct RadixTextAreaTheme {
    theme: ShadcnLook,
}

impl TextAreaTheme for RadixTextAreaTheme {
    fn resolve(
        &self,
        state: gpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
    ) -> gpui_luma::controls::textarea::TextAreaPalette {
        let tokens = self.theme.mode_tokens();
        super::textarea::textarea_palette(
            tokens.as_ref(),
            self.theme.mode(),
            super::textfield::ShadcnTextFieldStyle::Surface,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn textarea_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::textarea::TextAreaTemplate> {
    Arc::new(ThemedTextAreaTemplate::new(textarea_theme(Arc::clone(&theme))))
}

pub fn textarea_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TextAreaTheme> {
    Arc::new(RadixTextAreaTheme { theme: theme.as_ref().clone() })
}

struct RadixSoftTextAreaTheme {
    theme: ShadcnLook,
}

impl TextAreaTheme for RadixSoftTextAreaTheme {
    fn resolve(
        &self,
        state: gpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
    ) -> gpui_luma::controls::textarea::TextAreaPalette {
        let tokens = self.theme.mode_tokens();
        super::textarea::textarea_palette(
            tokens.as_ref(),
            self.theme.mode(),
            super::textfield::ShadcnTextFieldStyle::Soft,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn soft_textarea_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TextAreaTheme> {
    Arc::new(RadixSoftTextAreaTheme { theme: theme.as_ref().clone() })
}

pub fn autocomplete_textbox_theme(theme: Arc<ShadcnLook>) -> Arc<dyn AutocompleteTextBoxTheme> {
    Arc::new(RadixAutocompleteTextBoxTheme { theme: theme.as_ref().clone() })
}

pub fn selection_panel_look_provider(theme: Arc<ShadcnLook>) -> SelectionPanelLookProvider {
    Arc::new(move |size| {
        let tokens = theme.mode_tokens();
        selection_panel_look(tokens.as_ref(), theme.mode(), size)
    })
}

struct RadixTreeViewTheme {
    theme: ShadcnLook,
}

impl TreeViewTheme for RadixTreeViewTheme {
    fn resolve_row(&self, state: InteractionState, selected: bool) -> gpui_luma::controls::tree_view::TreeViewPalette {
        let tokens = self.theme.mode_tokens();
        tree_view_row_palette(tokens.as_ref(), selected, state)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

struct RadixAccordionTheme {
    theme: ShadcnLook,
}

impl AccordionTheme for RadixAccordionTheme {
    fn resolve_trigger(&self, state: InteractionState) -> gpui_luma::controls::accordion::AccordionPalette {
        let tokens = self.theme.mode_tokens();
        accordion_trigger_palette(tokens.as_ref(), self.theme.mode(), state)
    }

    fn resolve_content(&self, expanded: bool) -> gpui_luma::controls::accordion::AccordionContentPalette {
        let tokens = self.theme.mode_tokens();
        accordion_content_palette(tokens.as_ref(), self.theme.mode(), expanded)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

struct RadixTabsNavigationTheme {
    theme: ShadcnLook,
}

impl TabsNavigationTheme for RadixTabsNavigationTheme {
    fn resolve_list(
        &self,
        enabled: bool,
        size: ControlSize,
    ) -> gpui_luma::controls::tabs_navigation::TabsNavigationListLook {
        let tokens = self.theme.mode_tokens();
        tabs_navigation_list_look(tokens.as_ref(), enabled, size)
    }

    fn resolve_item(
        &self,
        active: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::tabs_navigation::TabsNavigationItemLook {
        let tokens = self.theme.mode_tokens();
        tabs_navigation_item_look(tokens.as_ref(), active, state, size)
    }

    fn font_family(&self) -> gpui::SharedString {
        self.theme.mode_tokens().typography.font.sans.family.clone().into()
    }
}

struct RadixNavigationSidebarTheme {
    theme: ShadcnLook,
}

impl NavigationSidebarTheme for RadixNavigationSidebarTheme {
    fn resolve_container(&self) -> gpui_luma::controls::navigation_sidebar::NavigationSidebarContainerLook {
        let tokens = self.theme.mode_tokens();
        navigation_sidebar_container_look(tokens.as_ref())
    }

    fn resolve_section(&self) -> gpui_luma::controls::navigation_sidebar::NavigationSidebarSectionLook {
        navigation_sidebar_section_look(&self.theme)
    }

    fn resolve_branch(
        &self,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::navigation_sidebar::NavigationSidebarItemLook {
        navigation_sidebar_branch_look(&self.theme, state, size)
    }

    fn resolve_item(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::navigation_sidebar::NavigationSidebarItemLook {
        navigation_sidebar_item_look(&self.theme, selected, state, size)
    }
}

pub fn tree_view_template<T>(theme: Arc<ShadcnLook>) -> Arc<dyn TreeViewTemplate<T>>
where
    T: Send + Sync + 'static,
{
    Arc::new(ThemedTreeViewTemplate::new(tree_view_theme(Arc::clone(&theme))))
}

pub fn tree_view_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TreeViewTheme> {
    Arc::new(RadixTreeViewTheme { theme: theme.as_ref().clone() })
}

pub fn accordion_template(theme: Arc<ShadcnLook>) -> Arc<dyn AccordionTemplate> {
    Arc::new(ThemedAccordionTemplate::new(accordion_theme(Arc::clone(&theme))))
}

pub fn accordion_theme(theme: Arc<ShadcnLook>) -> Arc<dyn AccordionTheme> {
    Arc::new(RadixAccordionTheme { theme: theme.as_ref().clone() })
}

pub fn tabs_navigation_template(theme: Arc<ShadcnLook>) -> Arc<dyn TabsNavigationTemplate> {
    Arc::new(ThemedTabsNavigationTemplate::new(tabs_navigation_theme(Arc::clone(&theme))))
}

pub fn tabs_navigation_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TabsNavigationTheme> {
    Arc::new(RadixTabsNavigationTheme { theme: theme.as_ref().clone() })
}

pub fn navigation_sidebar_template(theme: Arc<ShadcnLook>) -> Arc<dyn NavigationSidebarTemplate> {
    let menu_theme = floating_menu_theme(Arc::clone(&theme));
    Arc::new(ThemedNavigationSidebarTemplate::new_with_floating_menu_theme(
        Arc::new(RadixNavigationSidebarTheme { theme: theme.as_ref().clone() }),
        menu_theme,
    ))
}

pub fn navigation_sidebar_theme(theme: Arc<ShadcnLook>) -> Arc<dyn NavigationSidebarTheme> {
    Arc::new(RadixNavigationSidebarTheme { theme: theme.as_ref().clone() })
}

struct RadixControlGroupTheme {
    theme: ShadcnLook,
}

impl ControlGroupTheme for RadixControlGroupTheme {
    fn resolve_list(&self, enabled: bool) -> gpui_luma::controls::control_group::ControlGroupListLook {
        let tokens = self.theme.mode_tokens();
        control_group_list_look(tokens.as_ref(), enabled)
    }
}

pub fn control_group_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ControlGroupTheme> {
    Arc::new(RadixControlGroupTheme { theme: theme.as_ref().clone() })
}

pub fn control_group_template<T>(theme: Arc<ShadcnLook>) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    control_group_template_with_theme(control_group_theme(theme))
}

struct RadixListBoxTheme {
    theme: ShadcnLook,
}

impl ListBoxTheme for RadixListBoxTheme {
    fn resolve_list(
        &self,
        enabled: bool,
        focused: bool,
        size: ControlSize,
    ) -> gpui_luma::controls::listbox::ListBoxListLook {
        let tokens = self.theme.mode_tokens();
        listbox_list_look(tokens.as_ref(), enabled, focused, size)
    }

    fn resolve_row(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::listbox::ListBoxRowPalette {
        let tokens = self.theme.mode_tokens();
        listbox_row_palette(tokens.as_ref(), selected, state, size)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn listbox_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ListBoxTheme> {
    Arc::new(RadixListBoxTheme { theme: theme.as_ref().clone() })
}

pub fn listbox_template(theme: Arc<ShadcnLook>) -> ControlGroupTemplate<gpui_luma::controls::listbox::ListBoxItem> {
    listbox_template_with_theme(listbox_theme(theme))
}

struct RadixListViewTheme {
    theme: ShadcnLook,
}

impl ListViewTheme for RadixListViewTheme {
    fn resolve_look(
        &self,
        enabled: bool,
        focused: bool,
        size: ControlSize,
    ) -> gpui_luma::controls::list_view::ListViewLook {
        let tokens = self.theme.mode_tokens();
        list_view_look(tokens.as_ref(), enabled, focused, size)
    }

    fn resolve_row(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::list_view::ListViewRowPalette {
        let tokens = self.theme.mode_tokens();
        list_view_row_palette(tokens.as_ref(), selected, state, size)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn list_view_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ListViewTheme> {
    Arc::new(RadixListViewTheme { theme: theme.as_ref().clone() })
}

pub fn list_view_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::list_view::ListViewTemplate> {
    list_view_template_with_theme(list_view_theme(theme))
}

struct RadixPagerTheme {
    theme: ShadcnLook,
}

impl PagerTheme for RadixPagerTheme {
    fn resolve(
        &self,
        enabled: bool,
        style: gpui_luma::controls::pager::PagerStyle,
    ) -> gpui_luma::controls::pager::PagerLook {
        pager_look(&self.theme, enabled, style)
    }
}

pub fn pager_template(theme: Arc<ShadcnLook>) -> Arc<dyn PagerTemplate> {
    Arc::new(ThemedPagerTemplate::new(pager_theme(theme)))
}

pub fn pager_theme(theme: Arc<ShadcnLook>) -> Arc<dyn PagerTheme> {
    Arc::new(RadixPagerTheme { theme: theme.as_ref().clone() })
}

pub fn radio_group_template<T>(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
    layout: RadioGroupLayout,
) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    radio_group_buttons_template(theme.radio_button_template(style), layout)
}

struct RadixProgressTheme {
    theme: ShadcnLook,
}

impl ProgressTheme for RadixProgressTheme {
    fn resolve(&self, enabled: bool) -> gpui_luma::controls::progress::ProgressLook {
        let tokens = self.theme.mode_tokens();
        progress_look(tokens.as_ref(), enabled)
    }
}

pub fn progress_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::progress::ProgressTemplate> {
    Arc::new(ThemedProgressTemplate::new(progress_theme(theme)))
}

pub fn progress_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ProgressTheme> {
    Arc::new(RadixProgressTheme { theme: theme.as_ref().clone() })
}

pub fn card_template(theme: Arc<ShadcnLook>) -> Arc<dyn CardTemplate> {
    Arc::new(ThemedCardTemplate::new(card_theme(theme)))
}

pub fn overlay_window_template(theme: Arc<ShadcnLook>) -> Arc<dyn OverlayWindowTemplate> {
    Arc::new(ThemedOverlayWindowTemplate::new(overlay_window_theme(theme)))
}

pub fn overlay_window_theme(theme: Arc<ShadcnLook>) -> Arc<dyn OverlayWindowTheme> {
    Arc::new(RadixOverlayWindowTheme { theme: theme.as_ref().clone() })
}

pub fn card_theme(theme: Arc<ShadcnLook>) -> Arc<dyn CardTheme> {
    Arc::new(RadixCardTheme { theme: theme.as_ref().clone() })
}

struct RadixCardTheme {
    theme: ShadcnLook,
}

impl CardTheme for RadixCardTheme {
    fn resolve(&self, size: ControlSize) -> gpui_luma::controls::card::CardLook {
        card_look(&self.theme, size)
    }
}

struct RadixOverlayWindowTheme {
    theme: ShadcnLook,
}

impl OverlayWindowTheme for RadixOverlayWindowTheme {
    fn resolve(&self, size: ControlSize, mode: OverlayWindowMode) -> OverlayWindowLook {
        overlay_window_look(&self.theme, size, mode)
    }
}

pub fn toggle_template(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<bool>> {
    let button_theme = styled_button_family_theme(theme, style);
    Arc::new(DefaultButtonTemplate::new(button_theme.clone()).with_modifier(move |element, model| {
        if model.look.is_some() {
            return element;
        }

        let look = button_theme.resolve(ButtonFamilyRole::Toggle { selected: model.data }, model.size, model.state);
        element
            .bg(look.background)
            .text_color(look.foreground)
            .border_color(button_family_effective_border(look.border))
    }))
}
