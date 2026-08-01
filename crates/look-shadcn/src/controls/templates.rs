use std::sync::Arc;

use gpui_luma::controls::autocomplete::AutocompleteTextBoxTheme;
use gpui_luma::controls::button_family::{ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole, ButtonFamilyTheme};
use gpui_luma::controls::overlay_window::{
    OverlayWindowLook, OverlayWindowMode, OverlayWindowTemplate, OverlayWindowTheme, ThemedOverlayWindowTemplate,
};
use gpui_luma::controls::checkbox::{CheckboxTheme, ThemedCheckboxTemplate};
use gpui_luma::controls::command::button::{ButtonTemplate, DefaultButtonTemplate};
use gpui_luma::controls::context_menu::{ContextMenuTheme, ThemedContextMenuTemplate};
use gpui_luma::controls::control_group::{
    ControlGroupBuilder, ControlGroupItemLike, ControlGroupItemPalette, ControlGroupTemplate, ControlGroupTheme,
    control_group_template_with_theme,
};
use gpui_luma::controls::dock_splitter::DockSplitterTheme;
use gpui_luma::controls::radio_group::{RadioGroupLayout, radio_group_buttons_template};
use gpui_luma::controls::floating_menu::FloatingMenuTheme;
use gpui_luma::controls::list_view::{ListViewTheme, list_view_template_with_theme};
use gpui_luma::controls::navigation_sidebar::{
    NavigationSidebarTemplate, NavigationSidebarTheme, ThemedNavigationSidebarTemplate,
};
use gpui_luma::controls::listbox::{ListBoxTheme, listbox_template_with_theme};
use gpui_luma::controls::pager::{PagerLook, PagerTemplate, PagerTheme, ThemedPagerTemplate};
use gpui_luma::controls::popup_menu::{PopupMenuTheme, ThemedPopupMenuTemplate};
use gpui_luma::controls::progress::{ProgressTheme, ThemedLinearProgressTemplate, ThemedProgressTemplate};
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
use gpui_luma::controls::toolbar::{ThemedToolbarTemplate, ToolbarLook, ToolbarTemplate, ToolbarTheme, ToolbarVariant};

use gpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale};

use super::autocomplete::autocomplete_textbox_look;
use super::checkbox::checkbox_look;
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
use super::pager::{pager_button_look, pager_look};
use super::progress::progress_look;
use super::radio::radio_button_look;
use super::scrollbar::scrollbar_look;
use super::selection_panel::selection_panel_look;
use super::slider::slider_look;
use super::switch::{switch_look, switch_scale};
use super::accordion::{accordion_content_palette, accordion_trigger_palette};
use super::resizable_panels::resizable_panels_look;
use super::split_view::split_view_look;
use super::tree_view::tree_view_row_palette;
use super::tabs_navigation::{tabs_navigation_item_look, tabs_navigation_list_look};
use super::button::{button_look, button_palette};
use crate::look_context::LookContext;
use super::button::ShadcnButtonStyle;
use crate::look::ShadcnLook;

struct ShadcnStyledButtonFamilyTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl ButtonFamilyTheme for ShadcnStyledButtonFamilyTheme {
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
    Arc::new(ShadcnStyledButtonFamilyTheme { theme: theme.as_ref().clone(), style })
}

pub fn button_family_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ButtonFamilyTheme> {
    styled_button_family_theme(theme, ShadcnButtonStyle::Secondary)
}

pub fn button_template(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<()>> {
    Arc::new(DefaultButtonTemplate::new(styled_button_family_theme(theme, style)))
}

struct ShadcnStyledSwitchTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl SwitchTheme for ShadcnStyledSwitchTheme {
    fn resolve(
        &self,
        on: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::switch::SwitchPalette {
        let tokens = self.theme.mode_tokens();
        switch_look(tokens.as_ref(), self.theme.mode(), self.style, on, state, size)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn scale(&self, size: ControlSize, scale_factor: f32) -> gpui_luma::controls::switch::SwitchScale {
        let tokens = self.theme.mode_tokens();
        switch_scale(tokens.as_ref(), self.theme.mode(), self.style, size, scale_factor)
    }
}

struct ShadcnStyledCheckboxTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl CheckboxTheme for ShadcnStyledCheckboxTheme {
    fn resolve(
        &self,
        checked: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::checkbox::CheckboxPalette {
        let tokens = self.theme.mode_tokens();
        checkbox_look(tokens.as_ref(), self.style, checked, state, size)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

struct ShadcnStyledRadioButtonTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl RadioButtonTheme for ShadcnStyledRadioButtonTheme {
    fn resolve(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::radio_button::RadioButtonPalette {
        let tokens = self.theme.mode_tokens();
        radio_button_look(tokens.as_ref(), self.style, selected, state, size)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn checkbox_theme(theme: Arc<ShadcnLook>) -> Arc<dyn CheckboxTheme> {
    checkbox_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn checkbox_theme_with_style(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn CheckboxTheme> {
    Arc::new(ShadcnStyledCheckboxTheme { theme: theme.as_ref().clone(), style })
}

pub fn switch_theme(theme: Arc<ShadcnLook>) -> Arc<dyn SwitchTheme> {
    switch_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn switch_theme_with_style(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn SwitchTheme> {
    Arc::new(ShadcnStyledSwitchTheme { theme: theme.as_ref().clone(), style })
}

pub fn radio_button_theme(theme: Arc<ShadcnLook>) -> Arc<dyn RadioButtonTheme> {
    radio_button_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn radio_button_theme_with_style(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn RadioButtonTheme> {
    Arc::new(ShadcnStyledRadioButtonTheme { theme: theme.as_ref().clone(), style })
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
    slider_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn slider_theme_with_style(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn SliderTheme> {
    Arc::new(ShadcnSliderTheme { theme: theme.as_ref().clone(), style })
}

pub fn dock_splitter_theme(theme: Arc<ShadcnLook>) -> Arc<dyn DockSplitterTheme> {
    Arc::new(ShadcnDockSplitterTheme { theme: theme.as_ref().clone() })
}

pub fn resizable_panels_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ResizablePanelsTheme> {
    Arc::new(ShadcnResizablePanelsTheme { theme: theme.as_ref().clone() })
}

pub fn split_view_theme(theme: Arc<ShadcnLook>) -> Arc<dyn SplitViewTheme> {
    Arc::new(ShadcnSplitViewTheme { theme: theme.as_ref().clone() })
}

pub fn scrollbar_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ScrollbarTheme> {
    Arc::new(ShadcnScrollbarTheme { theme: theme.as_ref().clone() })
}

pub fn selector_theme(theme: Arc<ShadcnLook>) -> Arc<dyn SelectorTheme> {
    Arc::new(ShadcnSelectorTheme { theme: theme.as_ref().clone() })
}

pub fn popup_menu_theme(theme: Arc<ShadcnLook>) -> Arc<dyn PopupMenuTheme> {
    Arc::new(ShadcnPopupMenuTheme { theme: theme.as_ref().clone() })
}

struct ShadcnDockSplitterTheme {
    theme: ShadcnLook,
}

impl DockSplitterTheme for ShadcnDockSplitterTheme {
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

struct ShadcnResizablePanelsTheme {
    theme: ShadcnLook,
}

impl ResizablePanelsTheme for ShadcnResizablePanelsTheme {
    fn resolve(&self, state: InteractionState) -> gpui_luma::controls::resizable_panels::ResizablePanelsLook {
        let tokens = self.theme.mode_tokens();
        resizable_panels_look(tokens.as_ref(), self.theme.mode(), state)
    }
}

struct ShadcnSplitViewTheme {
    theme: ShadcnLook,
}

impl SplitViewTheme for ShadcnSplitViewTheme {
    fn resolve(&self, hovered: bool, enabled: bool) -> gpui_luma::controls::split_view::SplitViewLook {
        let tokens = self.theme.mode_tokens();
        split_view_look(tokens.as_ref(), self.theme.mode(), hovered, enabled)
    }
}

struct ShadcnSliderTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl SliderTheme for ShadcnSliderTheme {
    fn resolve(
        &self,
        size: gpui_luma::theme::ControlSize,
        thumb_size: Option<gpui_luma::controls::slider::SliderThumbSize>,
        state: InteractionState,
    ) -> gpui_luma::controls::slider::SliderLook {
        let tokens = self.theme.mode_tokens();
        slider_look(tokens.as_ref(), self.theme.mode(), self.style, size, thumb_size, state)
    }
}

struct ShadcnScrollbarTheme {
    theme: ShadcnLook,
}

impl ScrollbarTheme for ShadcnScrollbarTheme {
    fn resolve(
        &self,
        state: InteractionState,
        orientation: gpui_luma::controls::scrollbar::ScrollbarOrientation,
        size: gpui_luma::theme::ControlSize,
        style: gpui_luma::controls::scrollbar::ScrollbarStyle,
    ) -> gpui_luma::controls::scrollbar::ScrollbarLook {
        let tokens = self.theme.mode_tokens();
        scrollbar_look(tokens.as_ref(), state, orientation, size, style)
    }
}

pub fn slider_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::slider::SliderTemplate> {
    slider_template_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn slider_template_with_style(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> Arc<dyn gpui_luma::controls::slider::SliderTemplate> {
    Arc::new(ThemedSliderTemplate::new(slider_theme_with_style(theme, style)))
}

pub fn slider_angular_template(theme: Arc<ShadcnLook>) -> Arc<dyn SliderTemplate> {
    Arc::new(ThemedAngularDialTemplate::new(slider_theme(theme)))
}

pub fn slider_circular_ring_template(theme: Arc<ShadcnLook>) -> Arc<dyn SliderTemplate> {
    Arc::new(ThemedCircularRingTemplate::new(slider_theme(theme)))
}

pub fn scrollbar_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::scrollbar::ScrollbarTemplate> {
    Arc::new(ThemedScrollbarTemplate::new(Arc::new(ShadcnScrollbarTheme { theme: theme.as_ref().clone() })))
}

struct ShadcnFloatingMenuTheme {
    theme: ShadcnLook,
}

impl FloatingMenuTheme for ShadcnFloatingMenuTheme {
    fn resolve(&self) -> gpui_luma::controls::floating_menu::FloatingMenuLook {
        let tokens = self.theme.mode_tokens();
        floating_menu_look(tokens.as_ref(), self.theme.mode(), gpui_luma::theme::ControlSize::Md)
    }
}

struct ShadcnPopupMenuTheme {
    theme: ShadcnLook,
}

impl PopupMenuTheme for ShadcnPopupMenuTheme {
    fn resolve(
        &self,
        trigger_style: gpui_luma::controls::popup_menu::PopupMenuTriggerStyle,
        metrics: gpui_luma::controls::popup_menu::PopupMenuTriggerMetrics,
        state: InteractionState,
    ) -> gpui_luma::controls::popup_menu::PopupMenuPalette {
        let tokens = self.theme.mode_tokens();
        super::popup_menu::popup_menu_palette(tokens.as_ref(), self.theme.mode(), trigger_style, metrics, state)
    }

    fn resolve_look(
        &self,
        trigger_style: gpui_luma::controls::popup_menu::PopupMenuTriggerStyle,
        metrics: gpui_luma::controls::popup_menu::PopupMenuTriggerMetrics,
        state: InteractionState,
        scale_factor: f32,
        cx: &mut gpui::App,
    ) -> gpui_luma::controls::popup_menu::PopupMenuLook {
        let tokens = self.theme.mode_tokens();
        super::popup_menu::popup_menu_look(
            tokens.as_ref(),
            self.theme.mode(),
            trigger_style,
            metrics,
            state,
            scale_factor,
            cx,
        )
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

struct ShadcnContextMenuTheme {
    theme: ShadcnLook,
}

impl ContextMenuTheme for ShadcnContextMenuTheme {
    fn resolve(&self, state: InteractionState) -> gpui_luma::controls::context_menu::ContextMenuLook {
        let tokens = self.theme.mode_tokens();
        context_menu_look(tokens.as_ref(), self.theme.mode(), state)
    }
}

pub fn floating_menu_theme(theme: Arc<ShadcnLook>) -> Arc<dyn FloatingMenuTheme> {
    Arc::new(ShadcnFloatingMenuTheme { theme: theme.as_ref().clone() })
}

pub fn popup_menu_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::popup_menu::PopupMenuTemplate> {
    Arc::new(ThemedPopupMenuTemplate::new(Arc::new(ShadcnPopupMenuTheme { theme: theme.as_ref().clone() })))
}

pub fn context_menu_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ContextMenuTheme> {
    Arc::new(ShadcnContextMenuTheme { theme: theme.as_ref().clone() })
}

pub fn context_menu_template(
    theme: Arc<ShadcnLook>,
) -> Arc<dyn gpui_luma::controls::context_menu::ContextMenuTemplate> {
    Arc::new(ThemedContextMenuTemplate::new(context_menu_theme(Arc::clone(&theme))))
}

struct ShadcnSelectorTheme {
    theme: ShadcnLook,
}

impl SelectorTheme for ShadcnSelectorTheme {
    fn resolve(
        &self,
        trigger_style: gpui_luma::controls::selector::SelectorTriggerStyle,
        state: InteractionState,
        without_elevation: bool,
    ) -> gpui_luma::controls::selector::SelectorPalette {
        let tokens = self.theme.mode_tokens();
        super::selector::selector_palette(tokens.as_ref(), self.theme.mode(), trigger_style, state, without_elevation)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        trigger_style: gpui_luma::controls::selector::SelectorTriggerStyle,
        state: InteractionState,
        size: ControlSize,
        scale: &gpui_luma::theme::StandardBoxScale,
        without_elevation: bool,
    ) -> gpui_luma::controls::selector::SelectorLook {
        let tokens = self.theme.mode_tokens();
        super::selector::selector_look(
            tokens.as_ref(),
            self.theme.mode(),
            trigger_style,
            state,
            size,
            scale,
            without_elevation,
        )
    }
}

struct ShadcnTextFieldTheme {
    theme: ShadcnLook,
}

impl TextFieldTheme for ShadcnTextFieldTheme {
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
            super::textfield::ShadcnTextFieldStyle::Outline,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        _variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> gpui_luma::controls::textfield::TextFieldLook {
        let tokens = self.theme.mode_tokens();
        super::textfield::textfield_look(
            tokens.as_ref(),
            self.theme.mode(),
            super::textfield::ShadcnTextFieldStyle::Outline,
            state,
            enabled,
            size,
            scale,
        )
    }
}

struct ShadcnAutocompleteTextBoxTheme {
    theme: ShadcnLook,
}

impl AutocompleteTextBoxTheme for ShadcnAutocompleteTextBoxTheme {
    fn resolve(&self, size: ControlSize) -> gpui_luma::controls::autocomplete::AutocompleteTextBoxLook {
        let tokens = self.theme.mode_tokens();
        autocomplete_textbox_look(tokens.as_ref(), self.theme.mode(), size)
    }
}

pub fn selector_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::selector::SelectorTemplate> {
    Arc::new(ThemedSelectorTemplate::new(
        Arc::new(ShadcnSelectorTheme { theme: theme.as_ref().clone() }),
        default_selector_items_template::<gpui_luma::controls::selector::SelectorItem>(),
    ))
}

pub fn textfield_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::textfield::TextFieldTemplate> {
    Arc::new(ThemedTextFieldTemplate::new(Arc::new(ShadcnTextFieldTheme { theme: theme.as_ref().clone() })))
}

pub fn textfield_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TextFieldTheme> {
    Arc::new(ShadcnTextFieldTheme { theme: theme.as_ref().clone() })
}

struct ShadcnInputTextFieldTheme {
    theme: ShadcnLook,
}

impl TextFieldTheme for ShadcnInputTextFieldTheme {
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
            super::textfield::ShadcnTextFieldStyle::Input,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        _variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> gpui_luma::controls::textfield::TextFieldLook {
        let tokens = self.theme.mode_tokens();
        super::textfield::textfield_look(
            tokens.as_ref(),
            self.theme.mode(),
            super::textfield::ShadcnTextFieldStyle::Input,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn input_textfield_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TextFieldTheme> {
    Arc::new(ShadcnInputTextFieldTheme { theme: theme.as_ref().clone() })
}

pub fn input_textfield_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::textfield::TextFieldTemplate> {
    Arc::new(ThemedTextFieldTemplate::new(Arc::new(ShadcnInputTextFieldTheme { theme: theme.as_ref().clone() })))
}

struct ShadcnSurfaceTextFieldTheme {
    theme: ShadcnLook,
}

impl TextFieldTheme for ShadcnSurfaceTextFieldTheme {
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

    fn resolve_look(
        &self,
        _variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> gpui_luma::controls::textfield::TextFieldLook {
        let tokens = self.theme.mode_tokens();
        super::textfield::textfield_look(
            tokens.as_ref(),
            self.theme.mode(),
            super::textfield::ShadcnTextFieldStyle::Surface,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn surface_textfield_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TextFieldTheme> {
    Arc::new(ShadcnSurfaceTextFieldTheme { theme: theme.as_ref().clone() })
}

struct ShadcnPrimaryTextFieldTheme {
    theme: ShadcnLook,
}

impl TextFieldTheme for ShadcnPrimaryTextFieldTheme {
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
            super::textfield::ShadcnTextFieldStyle::Primary,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        _variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> gpui_luma::controls::textfield::TextFieldLook {
        let tokens = self.theme.mode_tokens();
        super::textfield::textfield_look(
            tokens.as_ref(),
            self.theme.mode(),
            super::textfield::ShadcnTextFieldStyle::Primary,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn primary_textfield_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TextFieldTheme> {
    Arc::new(ShadcnPrimaryTextFieldTheme { theme: theme.as_ref().clone() })
}

pub fn primary_textfield_template(
    theme: Arc<ShadcnLook>,
) -> Arc<dyn gpui_luma::controls::textfield::TextFieldTemplate> {
    Arc::new(ThemedTextFieldTemplate::new(Arc::new(ShadcnPrimaryTextFieldTheme {
        theme: theme.as_ref().clone(),
    })))
}

struct ShadcnTextAreaTheme {
    theme: ShadcnLook,
}

impl TextAreaTheme for ShadcnTextAreaTheme {
    fn resolve(
        &self,
        state: gpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
    ) -> gpui_luma::controls::textarea::TextAreaPalette {
        let tokens = self.theme.mode_tokens();
        super::textarea::textarea_palette(
            tokens.as_ref(),
            self.theme.mode(),
            super::textfield::ShadcnTextFieldStyle::Outline,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        state: gpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> gpui_luma::controls::textarea::TextAreaLook {
        let tokens = self.theme.mode_tokens();
        super::textarea::textarea_look(
            tokens.as_ref(),
            self.theme.mode(),
            super::textfield::ShadcnTextFieldStyle::Outline,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn textarea_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::textarea::TextAreaTemplate> {
    Arc::new(ThemedTextAreaTemplate::new(textarea_theme(Arc::clone(&theme))))
}

pub fn textarea_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TextAreaTheme> {
    Arc::new(ShadcnTextAreaTheme { theme: theme.as_ref().clone() })
}

struct ShadcnSurfaceTextAreaTheme {
    theme: ShadcnLook,
}

impl TextAreaTheme for ShadcnSurfaceTextAreaTheme {
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

    fn resolve_look(
        &self,
        state: gpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> gpui_luma::controls::textarea::TextAreaLook {
        let tokens = self.theme.mode_tokens();
        super::textarea::textarea_look(
            tokens.as_ref(),
            self.theme.mode(),
            super::textfield::ShadcnTextFieldStyle::Surface,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn surface_textarea_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TextAreaTheme> {
    Arc::new(ShadcnSurfaceTextAreaTheme { theme: theme.as_ref().clone() })
}

struct ShadcnPrimaryTextAreaTheme {
    theme: ShadcnLook,
}

impl TextAreaTheme for ShadcnPrimaryTextAreaTheme {
    fn resolve(
        &self,
        state: gpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
    ) -> gpui_luma::controls::textarea::TextAreaPalette {
        let tokens = self.theme.mode_tokens();
        super::textarea::textarea_palette(
            tokens.as_ref(),
            self.theme.mode(),
            super::textfield::ShadcnTextFieldStyle::Primary,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        state: gpui_luma::controls::textarea::TextAreaState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> gpui_luma::controls::textarea::TextAreaLook {
        let tokens = self.theme.mode_tokens();
        super::textarea::textarea_look(
            tokens.as_ref(),
            self.theme.mode(),
            super::textfield::ShadcnTextFieldStyle::Primary,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn primary_textarea_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TextAreaTheme> {
    Arc::new(ShadcnPrimaryTextAreaTheme { theme: theme.as_ref().clone() })
}

pub fn primary_textarea_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::textarea::TextAreaTemplate> {
    Arc::new(ThemedTextAreaTemplate::new(primary_textarea_theme(Arc::clone(&theme))))
}

pub fn autocomplete_textbox_theme(theme: Arc<ShadcnLook>) -> Arc<dyn AutocompleteTextBoxTheme> {
    Arc::new(ShadcnAutocompleteTextBoxTheme { theme: theme.as_ref().clone() })
}

pub fn selection_panel_look_provider(theme: Arc<ShadcnLook>) -> SelectionPanelLookProvider {
    Arc::new(move |size| {
        let tokens = theme.mode_tokens();
        selection_panel_look(tokens.as_ref(), theme.mode(), size)
    })
}

struct ShadcnTreeViewTheme {
    theme: ShadcnLook,
}

impl TreeViewTheme for ShadcnTreeViewTheme {
    fn resolve_row(
        &self,
        state: InteractionState,
        selected: bool,
        size: ControlSize,
    ) -> gpui_luma::controls::tree_view::TreeViewPalette {
        let tokens = self.theme.mode_tokens();
        tree_view_row_palette(tokens.as_ref(), selected, state, size)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

struct ShadcnAccordionTheme {
    theme: ShadcnLook,
}

impl AccordionTheme for ShadcnAccordionTheme {
    fn resolve_trigger(
        &self,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::accordion::AccordionPalette {
        let tokens = self.theme.mode_tokens();
        accordion_trigger_palette(tokens.as_ref(), self.theme.mode(), state, size)
    }

    fn resolve_content(&self, expanded: bool) -> gpui_luma::controls::accordion::AccordionContentPalette {
        let tokens = self.theme.mode_tokens();
        accordion_content_palette(tokens.as_ref(), self.theme.mode(), expanded)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

struct ShadcnTabsNavigationTheme {
    theme: ShadcnLook,
}

impl TabsNavigationTheme for ShadcnTabsNavigationTheme {
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

struct ShadcnNavigationSidebarTheme {
    theme: ShadcnLook,
}

impl NavigationSidebarTheme for ShadcnNavigationSidebarTheme {
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
    Arc::new(ShadcnTreeViewTheme { theme: theme.as_ref().clone() })
}

pub fn accordion_template(theme: Arc<ShadcnLook>) -> Arc<dyn AccordionTemplate> {
    Arc::new(ThemedAccordionTemplate::new(accordion_theme(Arc::clone(&theme))))
}

pub fn accordion_theme(theme: Arc<ShadcnLook>) -> Arc<dyn AccordionTheme> {
    Arc::new(ShadcnAccordionTheme { theme: theme.as_ref().clone() })
}

pub fn tabs_navigation_template(theme: Arc<ShadcnLook>) -> Arc<dyn TabsNavigationTemplate> {
    Arc::new(ThemedTabsNavigationTemplate::new(tabs_navigation_theme(Arc::clone(&theme))))
}

pub fn tabs_navigation_theme(theme: Arc<ShadcnLook>) -> Arc<dyn TabsNavigationTheme> {
    Arc::new(ShadcnTabsNavigationTheme { theme: theme.as_ref().clone() })
}

pub fn navigation_sidebar_template(theme: Arc<ShadcnLook>) -> Arc<dyn NavigationSidebarTemplate> {
    let menu_theme = floating_menu_theme(Arc::clone(&theme));
    Arc::new(ThemedNavigationSidebarTemplate::new_with_floating_menu_theme(
        Arc::new(ShadcnNavigationSidebarTheme { theme: theme.as_ref().clone() }),
        menu_theme,
    ))
}

pub fn navigation_sidebar_theme(theme: Arc<ShadcnLook>) -> Arc<dyn NavigationSidebarTheme> {
    Arc::new(ShadcnNavigationSidebarTheme { theme: theme.as_ref().clone() })
}

struct ShadcnControlGroupTheme {
    theme: ShadcnLook,
}

impl ControlGroupTheme for ShadcnControlGroupTheme {
    fn resolve_list(&self, enabled: bool) -> gpui_luma::controls::control_group::ControlGroupListLook {
        let tokens = self.theme.mode_tokens();
        control_group_list_look(tokens.as_ref(), enabled)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn default_item_palette(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
        _scale: &StandardBoxScale,
    ) -> ControlGroupItemPalette {
        let look = self.theme.resolve_ghost_button(ButtonFamilyRole::Toggle { selected }, size, state);
        let chrome = self.theme.chrome();

        ControlGroupItemPalette {
            foreground: look.foreground,
            background: look.background,
            muted_foreground: chrome.muted_text,
            typography: look.typography,
            font_family: look.font_family.clone(),
            focus_ring: look.focus_ring,
        }
    }
}

pub fn control_group_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ControlGroupTheme> {
    Arc::new(ShadcnControlGroupTheme { theme: theme.as_ref().clone() })
}

pub fn control_group_template<T>(theme: Arc<ShadcnLook>) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    control_group_template_with_theme(control_group_theme(theme))
}

pub fn menu_choice_group<T>(_theme: Arc<ShadcnLook>, id: impl Into<gpui::SharedString>) -> ControlGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    gpui_luma::controls::control_group::new(id).active_descendant()
}

struct ShadcnToolbarTheme {
    theme: ShadcnLook,
}

impl ToolbarTheme for ShadcnToolbarTheme {
    fn resolve(&self, enabled: bool, size: ControlSize, variant: ToolbarVariant) -> ToolbarLook {
        let tokens = self.theme.mode_tokens();
        let metrics = &tokens.metrics;
        let control = metrics.for_size(size);
        let transparent = gpui::hsla(0.0, 0.0, 0.0, 0.0);

        let (background, border) = match variant {
            ToolbarVariant::Outline => {
                let background = if enabled {
                    tokens.palette.muted_background
                } else {
                    tokens.palette.disabled_background
                };
                (background, tokens.palette.border_default)
            }
            ToolbarVariant::Ghost => (transparent, transparent),
        };

        ToolbarLook {
            background,
            border,
            separator: tokens.palette.border_default,
            radius: metrics.radius.md,
            padding_x: 6.0,
            padding_y: 4.0,
            gap: control.gap,
            separator_height: control.height,
        }
    }
}

pub fn toolbar_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ToolbarTheme> {
    Arc::new(ShadcnToolbarTheme { theme: theme.as_ref().clone() })
}

pub fn toolbar_template(theme: Arc<ShadcnLook>) -> Arc<dyn ToolbarTemplate> {
    Arc::new(ThemedToolbarTemplate::new(toolbar_theme(theme)))
}

struct ShadcnListBoxTheme {
    theme: ShadcnLook,
}

impl ListBoxTheme for ShadcnListBoxTheme {
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
    Arc::new(ShadcnListBoxTheme { theme: theme.as_ref().clone() })
}

pub fn listbox_template(theme: Arc<ShadcnLook>) -> ControlGroupTemplate<gpui_luma::controls::listbox::ListBoxItem> {
    listbox_template_with_theme(listbox_theme(theme))
}

struct ShadcnListViewTheme {
    theme: ShadcnLook,
}

impl ListViewTheme for ShadcnListViewTheme {
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
    Arc::new(ShadcnListViewTheme { theme: theme.as_ref().clone() })
}

pub fn list_view_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::list_view::ListViewTemplate> {
    list_view_template_with_theme(list_view_theme(theme))
}

struct ShadcnPagerTheme {
    theme: Arc<ShadcnLook>,
}

struct ShadcnPagerButtonTheme {
    theme: Arc<ShadcnLook>,
    pager_look: PagerLook,
}

impl ButtonFamilyTheme for ShadcnPagerButtonTheme {
    fn resolve(&self, role: ButtonFamilyRole, size: ControlSize, state: InteractionState) -> ButtonFamilyPalette {
        let tokens = self.theme.mode_tokens();
        let stylesheet = self.theme.stylesheet();
        let ctx = LookContext::new(tokens.as_ref(), self.theme.mode(), state);
        button_palette(&ctx, stylesheet.as_ref(), ShadcnButtonStyle::Outline, role, size)
    }

    fn resolve_look(
        &self,
        role: ButtonFamilyRole,
        _size: ControlSize,
        state: InteractionState,
        _scale: &gpui_luma::theme::StandardBoxScale,
        _pill_radius: f32,
    ) -> Option<ButtonFamilyLook> {
        Some(pager_button_look(self.theme.as_ref(), &self.pager_look, role, state))
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

impl PagerTheme for ShadcnPagerTheme {
    fn resolve(
        &self,
        enabled: bool,
        style: gpui_luma::controls::pager::PagerStyle,
    ) -> gpui_luma::controls::pager::PagerLook {
        pager_look(self.theme.as_ref(), enabled, style)
    }

    fn button_template(&self, pager_look: &PagerLook) -> Arc<dyn ButtonTemplate<()>> {
        Arc::new(DefaultButtonTemplate::new(Arc::new(ShadcnPagerButtonTheme {
            theme: Arc::clone(&self.theme),
            pager_look: pager_look.clone(),
        })))
    }
}

pub fn pager_template(theme: Arc<ShadcnLook>) -> Arc<dyn PagerTemplate> {
    Arc::new(ThemedPagerTemplate::new(pager_theme(theme)))
}

pub fn pager_theme(theme: Arc<ShadcnLook>) -> Arc<dyn PagerTheme> {
    Arc::new(ShadcnPagerTheme { theme })
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

struct ShadcnProgressTheme {
    theme: ShadcnLook,
}

impl ProgressTheme for ShadcnProgressTheme {
    fn resolve(
        &self,
        enabled: bool,
        size: gpui_luma::theme::ControlSize,
    ) -> gpui_luma::controls::progress::ProgressLook {
        let tokens = self.theme.mode_tokens();
        progress_look(tokens.as_ref(), enabled, size)
    }
}

pub fn progress_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::progress::ProgressTemplate> {
    Arc::new(ThemedProgressTemplate::new(progress_theme(theme)))
}

pub fn linear_progress_template(theme: Arc<ShadcnLook>) -> Arc<dyn gpui_luma::controls::progress::ProgressTemplate> {
    Arc::new(ThemedLinearProgressTemplate::new(progress_theme(theme)))
}

pub fn progress_theme(theme: Arc<ShadcnLook>) -> Arc<dyn ProgressTheme> {
    Arc::new(ShadcnProgressTheme { theme: theme.as_ref().clone() })
}

pub fn overlay_window_template(theme: Arc<ShadcnLook>) -> Arc<dyn OverlayWindowTemplate> {
    Arc::new(ThemedOverlayWindowTemplate::new(overlay_window_theme(theme)))
}

pub fn overlay_window_theme(theme: Arc<ShadcnLook>) -> Arc<dyn OverlayWindowTheme> {
    Arc::new(ShadcnOverlayWindowTheme { theme: theme.as_ref().clone() })
}

struct ShadcnOverlayWindowTheme {
    theme: ShadcnLook,
}

impl OverlayWindowTheme for ShadcnOverlayWindowTheme {
    fn resolve(&self, size: ControlSize, mode: OverlayWindowMode) -> OverlayWindowLook {
        overlay_window_look(&self.theme, size, mode)
    }
}

pub fn toggle_template(theme: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(DefaultButtonTemplate::new(styled_button_family_theme(theme, style)))
}
