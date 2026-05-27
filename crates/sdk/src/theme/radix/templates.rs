use std::sync::Arc;

use crate::controls::autocomplete::AutocompleteTextBoxTheme;
use crate::controls::button_family::{
    ButtonFamilyRole, ButtonFamilyTheme, ButtonKind, ButtonVariant, default_button_family_theme,
};
use crate::controls::checkbox::{CheckboxTheme, ThemedCheckboxTemplate};
use crate::controls::command::button::{ButtonTemplate, DefaultButtonTemplate};
use crate::controls::context_menu::ContextMenuTheme;
use crate::controls::control_group::{
    ControlGroupItemLike, ControlGroupTemplate, ControlGroupTheme, control_group_template_with_theme,
};
use crate::controls::floating_menu::FloatingMenuTheme;
use crate::controls::navigation_sidebar::{
    NavigationSidebarTemplate, NavigationSidebarTheme, ThemedNavigationSidebarTemplate,
};
use crate::controls::listbox::{ListBoxTheme, listbox_template_with_theme};
use crate::controls::popup_menu::{PopupMenuTheme, ThemedPopupMenuTemplate};
use crate::controls::progress::{ProgressTheme, ThemedProgressTemplate};
use crate::controls::radio_button::{RadioButtonTheme, ThemedRadioButtonTemplate};
use crate::controls::scrollbar::{ScrollbarTheme, ThemedScrollbarTemplate};
use crate::controls::selector::{SelectorTheme, ThemedSelectorTemplate};
use crate::controls::selector_panel::default_selector_items_template;
use crate::controls::selection_panel::SelectionPanelAppearanceProvider;
use crate::controls::slider::{SliderTheme, ThemedSliderTemplate};
use crate::controls::switch::{SwitchTheme, ThemedSwitchTemplate};
use crate::controls::tabs_navigation::{TabsNavigationTemplate, TabsNavigationTheme, ThemedTabsNavigationTemplate};
use crate::controls::textarea::{TextAreaTheme, ThemedTextAreaTemplate};
use crate::controls::textfield::{TextFieldState, TextFieldTheme, TextFieldVariant, ThemedTextFieldTemplate};
use gpui::Styled;

use crate::theme::{ControlSize, InteractionState};

use super::autocomplete::autocomplete_textbox_appearance;
use super::checkbox::checkbox_appearance;
use super::control_group::control_group_list_appearance;
use super::context_menu::context_menu_appearance;
use super::floating_menu::floating_menu_appearance;
use super::navigation_sidebar::{
    navigation_sidebar_branch_appearance, navigation_sidebar_container_appearance, navigation_sidebar_item_appearance,
    navigation_sidebar_section_appearance,
};
use super::listbox::{listbox_list_appearance, listbox_row_appearance};
use super::popup_menu::popup_menu_appearance;
use super::progress::progress_appearance;
use super::radio::radio_button_appearance;
use super::scrollbar::scrollbar_appearance;
use super::selection_panel::selection_panel_appearance;
use super::selector::selector_appearance;
use super::slider::slider_appearance;
use super::switch::switch_appearance;
use super::tabs_navigation::{tabs_navigation_item_appearance, tabs_navigation_list_appearance};
use super::textarea::textarea_appearance;
use super::textfield::textfield_appearance;
use super::button::button_appearance;
use super::{RadixButtonStyle, RadixTheme};

struct RadixButtonFamilyTheme {
    theme: RadixTheme,
}

impl ButtonFamilyTheme for RadixButtonFamilyTheme {
    fn resolve(
        &self,
        variant: ButtonVariant,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> crate::controls::button_family::ButtonFamilyAppearance {
        button_appearance(self.theme.mode_tokens(), radix_style_from_variant(variant), role, size, state)
    }
}

fn radix_style_from_variant(variant: ButtonVariant) -> RadixButtonStyle {
    match variant {
        ButtonVariant::Prominent => RadixButtonStyle::Primary,
        ButtonVariant::Standard => RadixButtonStyle::Secondary,
        ButtonVariant::Subtle => RadixButtonStyle::Outline,
        ButtonVariant::Ghost => RadixButtonStyle::Ghost,
    }
}

pub fn button_family_theme(theme: Arc<RadixTheme>) -> Arc<dyn ButtonFamilyTheme> {
    Arc::new(RadixButtonFamilyTheme { theme: theme.as_ref().clone() })
}

pub fn checkbox_theme(theme: Arc<RadixTheme>, style: RadixButtonStyle) -> Arc<dyn CheckboxTheme> {
    Arc::new(RadixCheckboxTheme { theme: theme.as_ref().clone(), style })
}

pub fn switch_theme(theme: Arc<RadixTheme>, style: RadixButtonStyle) -> Arc<dyn SwitchTheme> {
    Arc::new(RadixSwitchTheme { theme: theme.as_ref().clone(), style })
}

pub fn radio_button_theme(theme: Arc<RadixTheme>, style: RadixButtonStyle) -> Arc<dyn RadioButtonTheme> {
    Arc::new(RadixRadioButtonTheme { theme: theme.as_ref().clone(), style })
}

pub fn slider_theme(theme: Arc<RadixTheme>) -> Arc<dyn SliderTheme> {
    Arc::new(RadixSliderTheme { theme: theme.as_ref().clone() })
}

pub fn scrollbar_theme(theme: Arc<RadixTheme>) -> Arc<dyn ScrollbarTheme> {
    Arc::new(RadixScrollbarTheme { theme: theme.as_ref().clone() })
}

pub fn selector_theme(theme: Arc<RadixTheme>) -> Arc<dyn SelectorTheme> {
    Arc::new(RadixSelectorTheme { theme: theme.as_ref().clone() })
}

pub fn popup_menu_theme(theme: Arc<RadixTheme>) -> Arc<dyn PopupMenuTheme> {
    Arc::new(RadixPopupMenuTheme { theme: theme.as_ref().clone() })
}

struct RadixSwitchTheme {
    theme: RadixTheme,
    style: RadixButtonStyle,
}

impl SwitchTheme for RadixSwitchTheme {
    fn resolve(
        &self,
        _kind: ButtonKind,
        on: bool,
        state: InteractionState,
    ) -> crate::controls::switch::SwitchAppearance {
        switch_appearance(self.theme.mode_tokens(), self.theme.mode(), self.style, on, state)
    }
}

struct RadixCheckboxTheme {
    theme: RadixTheme,
    style: RadixButtonStyle,
}

impl CheckboxTheme for RadixCheckboxTheme {
    fn resolve(
        &self,
        _kind: ButtonKind,
        checked: bool,
        state: InteractionState,
    ) -> crate::controls::checkbox::CheckboxAppearance {
        checkbox_appearance(self.theme.mode_tokens(), self.style, checked, state)
    }
}

struct RadixRadioButtonTheme {
    theme: RadixTheme,
    style: RadixButtonStyle,
}

impl RadioButtonTheme for RadixRadioButtonTheme {
    fn resolve(
        &self,
        _kind: ButtonKind,
        selected: bool,
        state: InteractionState,
    ) -> crate::controls::radio_button::RadioButtonAppearance {
        radio_button_appearance(self.theme.mode_tokens(), self.style, selected, state)
    }
}

pub fn switch_template(theme: Arc<RadixTheme>, style: RadixButtonStyle) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(ThemedSwitchTemplate::new(Arc::new(RadixSwitchTheme { theme: theme.as_ref().clone(), style })))
}

pub fn checkbox_template(theme: Arc<RadixTheme>, style: RadixButtonStyle) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(ThemedCheckboxTemplate::new(Arc::new(RadixCheckboxTheme { theme: theme.as_ref().clone(), style })))
}

pub fn radio_button_template(theme: Arc<RadixTheme>, style: RadixButtonStyle) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(ThemedRadioButtonTemplate::new(Arc::new(RadixRadioButtonTheme {
        theme: theme.as_ref().clone(),
        style,
    })))
}

struct RadixSliderTheme {
    theme: RadixTheme,
}

impl SliderTheme for RadixSliderTheme {
    fn resolve(&self, state: InteractionState) -> crate::controls::slider::SliderAppearance {
        slider_appearance(self.theme.mode_tokens(), self.theme.mode(), state)
    }
}

struct RadixScrollbarTheme {
    theme: RadixTheme,
}

impl ScrollbarTheme for RadixScrollbarTheme {
    fn resolve(
        &self,
        state: InteractionState,
        orientation: crate::controls::scrollbar::ScrollbarOrientation,
    ) -> crate::controls::scrollbar::ScrollbarAppearance {
        scrollbar_appearance(self.theme.mode_tokens(), state, orientation)
    }
}

pub fn slider_template(theme: Arc<RadixTheme>) -> Arc<dyn crate::controls::slider::SliderTemplate> {
    Arc::new(ThemedSliderTemplate::new(Arc::new(RadixSliderTheme { theme: theme.as_ref().clone() })))
}

pub fn scrollbar_template(theme: Arc<RadixTheme>) -> Arc<dyn crate::controls::scrollbar::ScrollbarTemplate> {
    Arc::new(ThemedScrollbarTemplate::new(Arc::new(RadixScrollbarTheme { theme: theme.as_ref().clone() })))
}

struct RadixFloatingMenuTheme {
    theme: RadixTheme,
}

impl FloatingMenuTheme for RadixFloatingMenuTheme {
    fn resolve(&self) -> crate::controls::floating_menu::FloatingMenuAppearance {
        floating_menu_appearance(self.theme.mode_tokens(), self.theme.mode(), crate::theme::ControlSize::Md)
    }
}

struct RadixPopupMenuTheme {
    theme: RadixTheme,
}

impl PopupMenuTheme for RadixPopupMenuTheme {
    fn resolve(&self, state: InteractionState) -> crate::controls::popup_menu::PopupMenuAppearance {
        popup_menu_appearance(self.theme.mode_tokens(), self.theme.mode(), state)
    }
}

struct RadixContextMenuTheme {
    theme: RadixTheme,
}

impl ContextMenuTheme for RadixContextMenuTheme {
    fn resolve(&self, state: InteractionState) -> crate::controls::context_menu::ContextMenuAppearance {
        context_menu_appearance(self.theme.mode_tokens(), self.theme.mode(), state)
    }
}

pub fn floating_menu_theme(theme: Arc<RadixTheme>) -> Arc<dyn FloatingMenuTheme> {
    Arc::new(RadixFloatingMenuTheme { theme: theme.as_ref().clone() })
}

pub fn popup_menu_template(theme: Arc<RadixTheme>) -> Arc<dyn crate::controls::popup_menu::PopupMenuTemplate> {
    Arc::new(ThemedPopupMenuTemplate::new(Arc::new(RadixPopupMenuTheme { theme: theme.as_ref().clone() })))
}

pub fn context_menu_theme(theme: Arc<RadixTheme>) -> Arc<dyn ContextMenuTheme> {
    Arc::new(RadixContextMenuTheme { theme: theme.as_ref().clone() })
}

struct RadixSelectorTheme {
    theme: RadixTheme,
}

impl SelectorTheme for RadixSelectorTheme {
    fn resolve(&self, state: InteractionState) -> crate::controls::selector::SelectorAppearance {
        selector_appearance(self.theme.mode_tokens(), self.theme.mode(), state)
    }
}

struct RadixTextFieldTheme {
    theme: RadixTheme,
}

impl TextFieldTheme for RadixTextFieldTheme {
    fn resolve(
        &self,
        variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
    ) -> crate::controls::textfield::TextFieldAppearance {
        textfield_appearance(self.theme.mode_tokens(), variant, state, enabled)
    }
}

struct RadixAutocompleteTextBoxTheme {
    theme: RadixTheme,
}

impl AutocompleteTextBoxTheme for RadixAutocompleteTextBoxTheme {
    fn resolve(&self) -> crate::controls::autocomplete::AutocompleteTextBoxAppearance {
        autocomplete_textbox_appearance(self.theme.mode_tokens(), self.theme.mode(), ControlSize::Md)
    }
}

pub fn selector_template(theme: Arc<RadixTheme>) -> Arc<dyn crate::controls::selector::SelectorTemplate> {
    Arc::new(ThemedSelectorTemplate::new(
        Arc::new(RadixSelectorTheme { theme: theme.as_ref().clone() }),
        default_selector_items_template(),
    ))
}

pub fn textfield_template(theme: Arc<RadixTheme>) -> Arc<dyn crate::controls::textfield::TextFieldTemplate> {
    Arc::new(ThemedTextFieldTemplate::new(Arc::new(RadixTextFieldTheme { theme: theme.as_ref().clone() })))
}

pub fn textfield_theme(theme: Arc<RadixTheme>) -> Arc<dyn TextFieldTheme> {
    Arc::new(RadixTextFieldTheme { theme: theme.as_ref().clone() })
}

struct RadixTextAreaTheme {
    theme: RadixTheme,
}

impl TextAreaTheme for RadixTextAreaTheme {
    fn resolve(
        &self,
        state: crate::controls::textarea::TextAreaState,
        enabled: bool,
    ) -> crate::controls::textarea::TextAreaAppearance {
        textarea_appearance(self.theme.mode_tokens(), state, enabled)
    }
}

pub fn textarea_template(theme: Arc<RadixTheme>) -> Arc<dyn crate::controls::textarea::TextAreaTemplate> {
    Arc::new(ThemedTextAreaTemplate::new(textarea_theme(Arc::clone(&theme))))
}

pub fn textarea_theme(theme: Arc<RadixTheme>) -> Arc<dyn TextAreaTheme> {
    Arc::new(RadixTextAreaTheme { theme: theme.as_ref().clone() })
}

pub fn autocomplete_textbox_theme(theme: Arc<RadixTheme>) -> Arc<dyn AutocompleteTextBoxTheme> {
    Arc::new(RadixAutocompleteTextBoxTheme { theme: theme.as_ref().clone() })
}

pub fn selection_panel_appearance_provider(theme: Arc<RadixTheme>) -> SelectionPanelAppearanceProvider {
    Arc::new(move |size| selection_panel_appearance(theme.mode_tokens(), theme.mode(), size))
}

struct RadixTabsNavigationTheme {
    theme: RadixTheme,
}

impl TabsNavigationTheme for RadixTabsNavigationTheme {
    fn resolve_list(&self, enabled: bool) -> crate::controls::tabs_navigation::TabsNavigationListAppearance {
        tabs_navigation_list_appearance(self.theme.mode_tokens(), enabled)
    }

    fn resolve_item(
        &self,
        active: bool,
        state: InteractionState,
    ) -> crate::controls::tabs_navigation::TabsNavigationItemAppearance {
        tabs_navigation_item_appearance(self.theme.mode_tokens(), active, state)
    }
}

struct RadixNavigationSidebarTheme {
    theme: RadixTheme,
}

impl NavigationSidebarTheme for RadixNavigationSidebarTheme {
    fn resolve_container(&self) -> crate::controls::navigation_sidebar::NavigationSidebarContainerAppearance {
        navigation_sidebar_container_appearance(self.theme.mode_tokens())
    }

    fn resolve_section(&self) -> crate::controls::navigation_sidebar::NavigationSidebarSectionAppearance {
        navigation_sidebar_section_appearance(self.theme.mode_tokens())
    }

    fn resolve_branch(
        &self,
        state: InteractionState,
        size: ControlSize,
    ) -> crate::controls::navigation_sidebar::NavigationSidebarItemAppearance {
        navigation_sidebar_branch_appearance(self.theme.mode_tokens(), state, size)
    }

    fn resolve_item(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> crate::controls::navigation_sidebar::NavigationSidebarItemAppearance {
        navigation_sidebar_item_appearance(self.theme.mode_tokens(), selected, state, size)
    }
}

pub fn tabs_navigation_template(theme: Arc<RadixTheme>) -> Arc<dyn TabsNavigationTemplate> {
    Arc::new(ThemedTabsNavigationTemplate::new(tabs_navigation_theme(Arc::clone(&theme))))
}

pub fn tabs_navigation_theme(theme: Arc<RadixTheme>) -> Arc<dyn TabsNavigationTheme> {
    Arc::new(RadixTabsNavigationTheme { theme: theme.as_ref().clone() })
}

pub fn navigation_sidebar_template(theme: Arc<RadixTheme>) -> Arc<dyn NavigationSidebarTemplate> {
    let menu_theme = floating_menu_theme(Arc::clone(&theme));
    Arc::new(ThemedNavigationSidebarTemplate::new_with_floating_menu_theme(
        Arc::new(RadixNavigationSidebarTheme { theme: theme.as_ref().clone() }),
        menu_theme,
    ))
}

pub fn navigation_sidebar_theme(theme: Arc<RadixTheme>) -> Arc<dyn NavigationSidebarTheme> {
    Arc::new(RadixNavigationSidebarTheme { theme: theme.as_ref().clone() })
}

struct RadixControlGroupTheme {
    theme: RadixTheme,
}

impl ControlGroupTheme for RadixControlGroupTheme {
    fn resolve_list(&self, enabled: bool) -> crate::controls::control_group::ControlGroupListAppearance {
        control_group_list_appearance(self.theme.mode_tokens(), enabled)
    }
}

pub fn control_group_theme(theme: Arc<RadixTheme>) -> Arc<dyn ControlGroupTheme> {
    Arc::new(RadixControlGroupTheme { theme: theme.as_ref().clone() })
}

pub fn control_group_template<T>(theme: Arc<RadixTheme>) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    control_group_template_with_theme(control_group_theme(theme))
}

struct RadixListBoxTheme {
    theme: RadixTheme,
}

impl ListBoxTheme for RadixListBoxTheme {
    fn resolve_list(
        &self,
        enabled: bool,
        focused: bool,
        size: ControlSize,
    ) -> crate::controls::listbox::ListBoxListAppearance {
        listbox_list_appearance(self.theme.mode_tokens(), enabled, focused, size)
    }

    fn resolve_row(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> crate::controls::listbox::ListBoxRowAppearance {
        listbox_row_appearance(self.theme.mode_tokens(), selected, state, size)
    }
}

pub fn listbox_theme(theme: Arc<RadixTheme>) -> Arc<dyn ListBoxTheme> {
    Arc::new(RadixListBoxTheme { theme: theme.as_ref().clone() })
}

pub fn listbox_template(theme: Arc<RadixTheme>) -> ControlGroupTemplate<crate::controls::listbox::ListBoxItem> {
    listbox_template_with_theme(listbox_theme(theme))
}

struct RadixProgressTheme {
    theme: RadixTheme,
}

impl ProgressTheme for RadixProgressTheme {
    fn resolve(&self, enabled: bool) -> crate::controls::progress::ProgressAppearance {
        progress_appearance(self.theme.mode_tokens(), enabled)
    }
}

pub fn progress_template(theme: Arc<RadixTheme>) -> Arc<dyn crate::controls::progress::ProgressTemplate> {
    Arc::new(ThemedProgressTemplate::new(progress_theme(theme)))
}

pub fn progress_theme(theme: Arc<RadixTheme>) -> Arc<dyn ProgressTheme> {
    Arc::new(RadixProgressTheme { theme: theme.as_ref().clone() })
}

pub fn toggle_template(theme: Arc<RadixTheme>, style: RadixButtonStyle) -> Arc<dyn ButtonTemplate<bool>> {
    let radix = theme.as_ref().clone();
    Arc::new(DefaultButtonTemplate::new(default_button_family_theme()).with_modifier(move |element, model| {
        if model.appearance.is_some() {
            return element;
        }

        let appearance = match style {
            RadixButtonStyle::Primary => {
                radix.primary_button(ButtonFamilyRole::Toggle { selected: model.data }, model.size, model.state)
            }
            RadixButtonStyle::Secondary => {
                radix.secondary_button(ButtonFamilyRole::Toggle { selected: model.data }, model.size, model.state)
            }
            RadixButtonStyle::Outline => {
                radix.outline_button(ButtonFamilyRole::Toggle { selected: model.data }, model.size, model.state)
            }
            RadixButtonStyle::Ghost => {
                radix.ghost_button(ButtonFamilyRole::Toggle { selected: model.data }, model.size, model.state)
            }
        };

        element.bg(appearance.background).text_color(appearance.foreground).border_color(appearance.border)
    }))
}
