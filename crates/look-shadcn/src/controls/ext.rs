//! Theme factories and builder style helpers for shadcn look controls.
//!
//! [`ShadcnLookControlExt`] constructs controls with templates pre-bound from an
//! [`Arc<ShadcnLook>`]. Builder style helpers ([`ShadcnButtonStyleExt`],
//! [`ShadcnCheckboxStyleExt`], [`ShadcnSwitchStyleExt`], [`ShadcnTextFieldExt`]) apply
//! look styles to existing builders.

use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use gpui_luma::controls::autocomplete::{self, AutocompleteTextBoxBuilder};
use gpui_luma::controls::button_group::{self, IconGroupBuilder};
use super::card::ShadcnCardBuilder;
use gpui_luma::controls::checkbox::{self, CheckboxBuilder};
use gpui_luma::controls::combobox::{self, ComboBoxBuilder};
use gpui_luma::controls::command::button::{Button, ButtonBuilder, ControlIcon};
use gpui_luma::controls::command::icon_button;
use gpui_luma::controls::control_group::{ControlGroupBuilder, ControlGroupItemLike};
use gpui_luma::controls::list_view::{self, ListViewBuilder};
use gpui_luma::controls::listbox::{self, ListBoxItem};
use gpui_luma::controls::navigation_sidebar::{NavigationSidebar, NavigationSidebarBuilder};
use gpui_luma::controls::context_menu::ContextMenu;
use gpui_luma::controls::overlay_window::OverlayWindowBuilder;
use gpui_luma::controls::pager::{self, PagerBuilder};
use gpui_luma::controls::popup_menu::PopupMenu;
use gpui_luma::controls::progress::{self, ProgressBuilder};
use gpui_luma::controls::stepper::{self, StepperBuilder};
use gpui_luma::controls::resizable_panels::ResizablePanelsBuilder;
use gpui_luma::controls::radio_button::{self, RadioButtonBuilder};
use gpui_luma::controls::radio_group::{
    self, RadioGroupBuilder, RadioGroupLayout, radio_group_button_item_element_template, radio_group_buttons_template,
};
use gpui_luma::controls::scrollbar::{self, ScrollbarBuilder};
use gpui_luma::controls::selector::{Selector, SelectorBuilder, SelectorItem};
use gpui_luma::controls::selection_panel::{SelectionPanelBuilder, SelectionPanelControl, SelectionPanelItem};
use gpui_luma::controls::search_selector::{self, SearchSelectorBuilder};
use gpui_luma::controls::slider::{self, SliderBuilder};
use gpui_luma::controls::switch::{self, SwitchBuilder};
use gpui_luma::controls::accordion::AccordionBuilder;
use gpui_luma::controls::tree_view::TreeViewBuilder;
use gpui_luma::controls::tabs_navigation::{TabsNavigation, TabsNavigationBuilder};
use gpui_luma::controls::textarea::{self, TextAreaBuilder, ThemedTextAreaTemplate};
use gpui_luma::controls::textfield::{self, TextFieldBuilder, ThemedTextFieldTemplate};
use gpui_luma::controls::toggle::{self, ToggleBuilder};
use gpui_luma::controls::toolbar::{self, ToolbarBuilder};
use super::button::ShadcnButtonStyle;
use crate::elements::Badge;
use crate::look::ShadcnLook;

/// Constructs SDK controls and visual add-ons from a shared [`ShadcnLook`].
pub trait ShadcnLookControlExt {
    fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn primary_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn secondary_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn outline_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn ghost_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn content_only_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;

    fn checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;
    fn primary_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;
    fn secondary_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;
    fn outline_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;
    fn ghost_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;
    fn content_only_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;

    fn switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;
    fn primary_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;
    fn secondary_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;
    fn outline_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;
    fn ghost_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;
    fn content_only_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;

    fn radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder;
    fn primary_radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder;
    fn secondary_radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder;
    fn outline_radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder;
    fn ghost_radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder;
    fn content_only_radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder;

    fn primary_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()>;
    fn secondary_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()>;
    fn outline_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()>;
    fn ghost_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()>;
    fn content_only_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()>;

    fn toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder;
    fn primary_toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder;
    fn secondary_toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder;
    fn outline_toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder;
    fn ghost_toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder;
    fn content_only_toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder;

    fn popup_menu(&self, id: impl Into<SharedString>) -> gpui_luma::controls::popup_menu::PopupMenuBuilder;
    fn context_menu(&self, id: impl Into<SharedString>) -> gpui_luma::controls::context_menu::ContextMenuBuilder;
    fn listbox(&self, id: impl Into<SharedString>) -> ControlGroupBuilder<ListBoxItem>;
    fn listbox_multiple(&self, id: impl Into<SharedString>) -> ControlGroupBuilder<ListBoxItem>;
    fn list_view<T>(&self, id: impl Into<SharedString>) -> ListViewBuilder<T>
    where
        T: 'static;
    fn radio_group<T>(&self, id: impl Into<SharedString>) -> RadioGroupBuilder<T>
    where
        T: ControlGroupItemLike + Clone + Send + Sync + 'static;
    fn radio_group_horizontal<T>(&self, id: impl Into<SharedString>) -> RadioGroupBuilder<T>
    where
        T: ControlGroupItemLike + Clone + Send + Sync + 'static;
    fn menu_choice_group<T>(&self, id: impl Into<SharedString>) -> ControlGroupBuilder<T>
    where
        T: ControlGroupItemLike + 'static;
    fn slider(&self, id: impl Into<SharedString>) -> SliderBuilder;
    fn primary_slider(&self, id: impl Into<SharedString>) -> SliderBuilder;
    fn secondary_slider(&self, id: impl Into<SharedString>) -> SliderBuilder;
    fn pager(&self, id: impl Into<SharedString>) -> PagerBuilder;
    fn progress(&self, id: impl Into<SharedString>) -> ProgressBuilder;
    fn linear_progress(&self, id: impl Into<SharedString>) -> ProgressBuilder;
    fn stepper(&self, id: impl Into<SharedString>, step_count: usize) -> StepperBuilder;
    fn card(&self, id: impl Into<SharedString>) -> ShadcnCardBuilder;
    fn overlay_window(&self, id: impl Into<SharedString>) -> OverlayWindowBuilder;
    fn badge(&self, label: impl Into<SharedString>) -> Badge;
    fn resizable_panels(&self, id: impl Into<SharedString>) -> ResizablePanelsBuilder;
    fn scrollbar(&self, id: impl Into<SharedString>) -> ScrollbarBuilder;
    fn selector(&self, id: impl Into<SharedString>) -> SelectorBuilder<SelectorItem>;
    fn tabs_navigation(&self, id: impl Into<SharedString>) -> TabsNavigationBuilder;
    fn accordion(&self, id: impl Into<SharedString>) -> AccordionBuilder;
    fn tree_view<T>(&self, id: impl Into<SharedString>) -> TreeViewBuilder<T>
    where
        T: Clone + Send + Sync + 'static;
    fn navigation_sidebar(&self, id: impl Into<SharedString>) -> NavigationSidebarBuilder;

    fn autocomplete(
        &self,
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = gpui_luma::controls::autocomplete::SelectionItem>,
    ) -> AutocompleteTextBoxBuilder;
    fn combobox(
        &self,
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = gpui_luma::controls::combobox::SelectionItem>,
    ) -> ComboBoxBuilder;
    fn search_selector(
        &self,
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = gpui_luma::controls::search_selector::SelectionItem>,
    ) -> SearchSelectorBuilder;

    fn button_group<T>(&self, id: impl Into<SharedString>) -> IconGroupBuilder<T>
    where
        T: ControlGroupItemLike + Clone + Send + Sync + 'static;
    fn icon_toolbar<T>(&self, id: impl Into<SharedString>) -> IconGroupBuilder<T>
    where
        T: ControlGroupItemLike + Clone + Send + Sync + 'static;
    fn icon_toolbar_multiple<T>(&self, id: impl Into<SharedString>) -> IconGroupBuilder<T>
    where
        T: ControlGroupItemLike + Clone + Send + Sync + 'static;
    fn toolbar(&self, id: impl Into<SharedString>) -> ToolbarBuilder;
    fn selection_panel(
        &self,
        id: impl Into<SharedString>,
        cx: &mut impl AppContext,
    ) -> Entity<SelectionPanelControl<SelectionPanelItem>>;
    fn selection_panel_builder(&self, id: impl Into<SharedString>) -> SelectionPanelBuilder<SelectionPanelItem>;

    fn textfield(&self, id: impl Into<SharedString>) -> TextFieldBuilder;
    fn textarea(&self, id: impl Into<SharedString>) -> TextAreaBuilder;
}

impl ShadcnLookControlExt for Arc<ShadcnLook> {
    fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(ShadcnButtonStyle::Secondary))
    }

    fn primary_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(ShadcnButtonStyle::Primary))
    }

    fn secondary_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(ShadcnButtonStyle::Secondary))
    }

    fn outline_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(ShadcnButtonStyle::Outline))
    }

    fn ghost_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(ShadcnButtonStyle::Ghost))
    }

    fn content_only_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id)
            .template(self.button_template(ShadcnButtonStyle::ContentOnly))
            .without_adorners()
            .without_elevation()
    }

    fn checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder {
        checkbox::new(id).primary(self)
    }

    fn primary_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder {
        checkbox::new(id).primary(self)
    }

    fn secondary_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder {
        checkbox::new(id).secondary(self)
    }

    fn outline_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder {
        checkbox::new(id).outline(self)
    }

    fn ghost_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder {
        checkbox::new(id).ghost(self)
    }

    fn content_only_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder {
        checkbox::new(id)
            .template(self.checkbox_template(ShadcnButtonStyle::ContentOnly))
            .without_adorners()
            .without_elevation()
    }

    fn switch(&self, id: impl Into<SharedString>) -> SwitchBuilder {
        switch::new(id).primary(self)
    }

    fn primary_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder {
        switch::new(id).primary(self)
    }

    fn secondary_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder {
        switch::new(id).secondary(self)
    }

    fn outline_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder {
        switch::new(id).outline(self)
    }

    fn ghost_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder {
        switch::new(id).ghost(self)
    }

    fn content_only_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder {
        switch::new(id)
            .template(self.switch_template(ShadcnButtonStyle::ContentOnly))
            .without_adorners()
            .without_elevation()
    }

    fn radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder {
        radio_button::new(id).template(self.radio_button_template(ShadcnButtonStyle::Primary))
    }

    fn primary_radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder {
        radio_button::new(id).template(self.radio_button_template(ShadcnButtonStyle::Primary))
    }

    fn secondary_radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder {
        radio_button::new(id).template(self.radio_button_template(ShadcnButtonStyle::Secondary))
    }

    fn outline_radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder {
        radio_button::new(id).template(self.radio_button_template(ShadcnButtonStyle::Outline))
    }

    fn ghost_radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder {
        radio_button::new(id).template(self.radio_button_template(ShadcnButtonStyle::Ghost))
    }

    fn content_only_radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder {
        radio_button::new(id)
            .template(self.radio_button_template(ShadcnButtonStyle::ContentOnly))
            .without_adorners()
            .without_elevation()
    }

    fn primary_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()> {
        icon_button::new(id, icon).template(self.button_template(ShadcnButtonStyle::Primary))
    }

    fn secondary_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()> {
        icon_button::new(id, icon).template(self.button_template(ShadcnButtonStyle::Secondary))
    }

    fn outline_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()> {
        icon_button::new(id, icon).template(self.button_template(ShadcnButtonStyle::Outline))
    }

    fn ghost_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()> {
        icon_button::new(id, icon).template(self.button_template(ShadcnButtonStyle::Ghost))
    }

    fn content_only_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()> {
        icon_button::new(id, icon)
            .template(self.button_template(ShadcnButtonStyle::ContentOnly))
            .without_adorners()
            .without_elevation()
    }

    fn toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder {
        toggle::new(id).template(self.toggle_template(ShadcnButtonStyle::Secondary))
    }

    fn primary_toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder {
        toggle::new(id).template(self.toggle_template(ShadcnButtonStyle::Primary))
    }

    fn secondary_toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder {
        toggle::new(id).template(self.toggle_template(ShadcnButtonStyle::Secondary))
    }

    fn outline_toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder {
        toggle::new(id).template(self.toggle_template(ShadcnButtonStyle::Outline))
    }

    fn ghost_toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder {
        toggle::new(id).template(self.toggle_template(ShadcnButtonStyle::Ghost))
    }

    fn content_only_toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder {
        toggle::new(id)
            .template(self.toggle_template(ShadcnButtonStyle::ContentOnly))
            .without_adorners()
            .without_elevation()
    }

    fn popup_menu(&self, id: impl Into<SharedString>) -> gpui_luma::controls::popup_menu::PopupMenuBuilder {
        PopupMenu::new(id).template(self.popup_menu_template())
    }

    fn context_menu(&self, id: impl Into<SharedString>) -> gpui_luma::controls::context_menu::ContextMenuBuilder {
        ContextMenu::new(id).template(self.context_menu_template())
    }

    fn listbox(&self, id: impl Into<SharedString>) -> ControlGroupBuilder<ListBoxItem> {
        listbox::new(id).template(self.listbox_template())
    }

    fn listbox_multiple(&self, id: impl Into<SharedString>) -> ControlGroupBuilder<ListBoxItem> {
        listbox::multiple(id).template(self.listbox_template())
    }

    fn list_view<T>(&self, id: impl Into<SharedString>) -> ListViewBuilder<T>
    where
        T: 'static,
    {
        list_view::new_typed(id).theme(self.list_view_theme()).template(self.list_view_template())
    }

    fn radio_group<T>(&self, id: impl Into<SharedString>) -> RadioGroupBuilder<T>
    where
        T: ControlGroupItemLike + Clone + Send + Sync + 'static,
    {
        radio_group::new(id)
            .template(radio_group_buttons_template(
                self.radio_button_template(ShadcnButtonStyle::Primary),
                RadioGroupLayout::Vertical,
            ))
            .item_element_template(radio_group_button_item_element_template(
                self.radio_button_template(ShadcnButtonStyle::Primary),
            ))
    }

    fn radio_group_horizontal<T>(&self, id: impl Into<SharedString>) -> RadioGroupBuilder<T>
    where
        T: ControlGroupItemLike + Clone + Send + Sync + 'static,
    {
        radio_group::horizontal(id)
            .template(radio_group_buttons_template(
                self.radio_button_template(ShadcnButtonStyle::Primary),
                RadioGroupLayout::Horizontal,
            ))
            .item_element_template(radio_group_button_item_element_template(
                self.radio_button_template(ShadcnButtonStyle::Primary),
            ))
    }

    fn menu_choice_group<T>(&self, id: impl Into<SharedString>) -> ControlGroupBuilder<T>
    where
        T: ControlGroupItemLike + 'static,
    {
        super::templates::menu_choice_group(Arc::clone(self), id)
    }

    fn slider(&self, id: impl Into<SharedString>) -> SliderBuilder {
        slider::new(id).template(self.slider_template())
    }

    fn primary_slider(&self, id: impl Into<SharedString>) -> SliderBuilder {
        slider::new(id).template(self.slider_template_with_style(ShadcnButtonStyle::Primary))
    }

    fn secondary_slider(&self, id: impl Into<SharedString>) -> SliderBuilder {
        slider::new(id).template(self.slider_template_with_style(ShadcnButtonStyle::Secondary))
    }

    fn pager(&self, id: impl Into<SharedString>) -> PagerBuilder {
        pager::new(id).theme(self.pager_theme())
    }

    fn progress(&self, id: impl Into<SharedString>) -> ProgressBuilder {
        progress::new(id).template(self.progress_template())
    }

    fn linear_progress(&self, id: impl Into<SharedString>) -> ProgressBuilder {
        progress::new(id).linear().template(self.linear_progress_template())
    }

    fn stepper(&self, id: impl Into<SharedString>, step_count: usize) -> StepperBuilder {
        stepper::stepper(id, step_count).template(self.stepper_template())
    }

    fn card(&self, id: impl Into<SharedString>) -> ShadcnCardBuilder {
        ShadcnCardBuilder::new(Arc::clone(self), id)
    }

    fn overlay_window(&self, id: impl Into<SharedString>) -> OverlayWindowBuilder {
        ShadcnLook::overlay_window(self, id)
    }

    fn badge(&self, label: impl Into<SharedString>) -> Badge {
        ShadcnLook::badge(self, label)
    }

    fn resizable_panels(&self, id: impl Into<SharedString>) -> ResizablePanelsBuilder {
        ShadcnLook::resizable_panels(self, id)
    }

    fn scrollbar(&self, id: impl Into<SharedString>) -> ScrollbarBuilder {
        scrollbar::Scrollbar::new(id).template(self.scrollbar_template())
    }

    fn selector(&self, id: impl Into<SharedString>) -> SelectorBuilder<SelectorItem> {
        Selector::new(id).template(self.selector_template())
    }

    fn tabs_navigation(&self, id: impl Into<SharedString>) -> TabsNavigationBuilder {
        TabsNavigation::new(id).template(self.tabs_navigation_template())
    }

    fn accordion(&self, id: impl Into<SharedString>) -> AccordionBuilder {
        ShadcnLook::accordion(self, id)
    }

    fn tree_view<T>(&self, id: impl Into<SharedString>) -> TreeViewBuilder<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        ShadcnLook::tree_view(self, id)
    }

    fn navigation_sidebar(&self, id: impl Into<SharedString>) -> NavigationSidebarBuilder {
        NavigationSidebar::new(id)
            .template(self.navigation_sidebar_template())
            .scrollbar_template(self.scrollbar_template())
    }

    fn autocomplete(
        &self,
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = gpui_luma::controls::autocomplete::SelectionItem>,
    ) -> AutocompleteTextBoxBuilder {
        let theme = Arc::clone(self);
        autocomplete::new(id, items)
            // Editable trigger: Primary textfield elevation (`shadow-xs`), not transparent Input.
            .textfield_template(self.primary_textfield_template())
            .autocomplete_theme(self.autocomplete_textbox_theme())
            .scrollbar_template(self.scrollbar_template())
            .popup_look_provider(Arc::new(move |size| theme.selector_items_panel_look(size)))
    }

    fn combobox(
        &self,
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = gpui_luma::controls::combobox::SelectionItem>,
    ) -> ComboBoxBuilder {
        let theme = Arc::clone(self);
        combobox::new(id, items)
            // Editable trigger: Primary textfield elevation (`shadow-xs`), not transparent Input.
            .textfield_template(self.primary_textfield_template())
            .autocomplete_theme(self.autocomplete_textbox_theme())
            .scrollbar_template(self.scrollbar_template())
            .popup_look_provider(Arc::new(move |size| theme.selector_items_panel_look(size)))
    }

    fn search_selector(
        &self,
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = gpui_luma::controls::search_selector::SelectionItem>,
    ) -> SearchSelectorBuilder {
        let theme = Arc::clone(self);
        search_selector::new(id, items)
            // Popup search field stays Input; display trigger uses selector Outline chrome.
            .textfield_template(self.input_textfield_template())
            .autocomplete_theme(self.autocomplete_textbox_theme())
            .selector_theme(self.selector_theme())
            .scrollbar_template(self.scrollbar_template())
            .popup_look_provider(Arc::new(move |size| theme.selector_items_panel_look(size)))
    }

    fn button_group<T>(&self, id: impl Into<SharedString>) -> IconGroupBuilder<T>
    where
        T: ControlGroupItemLike + Clone + Send + Sync + 'static,
    {
        button_group::new(id).template(self.control_group_template())
    }

    fn icon_toolbar<T>(&self, id: impl Into<SharedString>) -> IconGroupBuilder<T>
    where
        T: ControlGroupItemLike + Clone + Send + Sync + 'static,
    {
        button_group::icon_toolbar(id, self.control_group_theme())
    }

    fn icon_toolbar_multiple<T>(&self, id: impl Into<SharedString>) -> IconGroupBuilder<T>
    where
        T: ControlGroupItemLike + Clone + Send + Sync + 'static,
    {
        button_group::icon_toolbar_multiple(id, self.control_group_theme())
    }

    fn toolbar(&self, id: impl Into<SharedString>) -> ToolbarBuilder {
        toolbar::new(id).template(self.toolbar_template())
    }

    fn selection_panel(
        &self,
        id: impl Into<SharedString>,
        cx: &mut impl AppContext,
    ) -> Entity<SelectionPanelControl<SelectionPanelItem>> {
        self.selection_panel_builder(id).spawn(cx)
    }

    fn selection_panel_builder(&self, id: impl Into<SharedString>) -> SelectionPanelBuilder<SelectionPanelItem> {
        let theme = Arc::clone(self);
        gpui_luma::controls::selection_panel::new(id)
            .scrollbar_template(theme.scrollbar_template())
            .look_provider(Arc::new(move |size| theme.selection_panel_look(size)))
    }

    fn textfield(&self, id: impl Into<SharedString>) -> TextFieldBuilder {
        textfield::new(id).template(self.textfield_template())
    }

    fn textarea(&self, id: impl Into<SharedString>) -> TextAreaBuilder {
        textarea::TextArea::new(id).template(self.textarea_template()).theme(self.textarea_theme())
    }
}

/// Applies a shadcn button style to a [`ButtonBuilder`].
pub trait ShadcnButtonStyleExt {
    fn look_style(self, theme: &Arc<ShadcnLook>, style: ShadcnButtonStyle) -> ButtonBuilder<()>;
    fn primary(self, theme: &Arc<ShadcnLook>) -> ButtonBuilder<()>;
    fn secondary(self, theme: &Arc<ShadcnLook>) -> ButtonBuilder<()>;
    fn outline(self, theme: &Arc<ShadcnLook>) -> ButtonBuilder<()>;
    fn ghost(self, theme: &Arc<ShadcnLook>) -> ButtonBuilder<()>;
}

impl ShadcnButtonStyleExt for ButtonBuilder<()> {
    fn look_style(self, theme: &Arc<ShadcnLook>, style: ShadcnButtonStyle) -> ButtonBuilder<()> {
        self.template(theme.button_template(style))
    }

    fn primary(self, theme: &Arc<ShadcnLook>) -> ButtonBuilder<()> {
        self.look_style(theme, ShadcnButtonStyle::Primary)
    }

    fn secondary(self, theme: &Arc<ShadcnLook>) -> ButtonBuilder<()> {
        self.look_style(theme, ShadcnButtonStyle::Secondary)
    }

    fn outline(self, theme: &Arc<ShadcnLook>) -> ButtonBuilder<()> {
        self.look_style(theme, ShadcnButtonStyle::Outline)
    }

    fn ghost(self, theme: &Arc<ShadcnLook>) -> ButtonBuilder<()> {
        self.look_style(theme, ShadcnButtonStyle::Ghost)
    }
}

/// Binds a shadcn text field template on a [`TextFieldBuilder`].
pub trait ShadcnTextFieldExt {
    fn look_theme(self, theme: &Arc<ShadcnLook>) -> TextFieldBuilder;
    fn outline(self, theme: &Arc<ShadcnLook>) -> TextFieldBuilder;
    fn input(self, theme: &Arc<ShadcnLook>) -> TextFieldBuilder;
    fn surface(self, theme: &Arc<ShadcnLook>) -> TextFieldBuilder;
    fn primary(self, theme: &Arc<ShadcnLook>) -> TextFieldBuilder;
}

impl ShadcnTextFieldExt for TextFieldBuilder {
    fn look_theme(self, theme: &Arc<ShadcnLook>) -> TextFieldBuilder {
        self.template(theme.textfield_template())
    }

    fn outline(self, theme: &Arc<ShadcnLook>) -> TextFieldBuilder {
        self.look_theme(theme)
    }

    fn input(self, theme: &Arc<ShadcnLook>) -> TextFieldBuilder {
        self.template(theme.input_textfield_template())
    }

    fn surface(self, theme: &Arc<ShadcnLook>) -> TextFieldBuilder {
        self.template(Arc::new(ThemedTextFieldTemplate::new(theme.surface_textfield_theme())))
    }

    fn primary(self, theme: &Arc<ShadcnLook>) -> TextFieldBuilder {
        self.template(theme.primary_textfield_template())
    }
}

/// Binds a shadcn text area template and theme on a [`TextAreaBuilder`].
pub trait ShadcnTextAreaExt {
    fn look_theme(self, theme: &Arc<ShadcnLook>) -> TextAreaBuilder;
    fn outline(self, theme: &Arc<ShadcnLook>) -> TextAreaBuilder;
    fn surface(self, theme: &Arc<ShadcnLook>) -> TextAreaBuilder;
    fn primary(self, theme: &Arc<ShadcnLook>) -> TextAreaBuilder;
}

impl ShadcnTextAreaExt for TextAreaBuilder {
    fn look_theme(self, theme: &Arc<ShadcnLook>) -> TextAreaBuilder {
        self.template(theme.textarea_template()).theme(theme.textarea_theme())
    }

    fn outline(self, theme: &Arc<ShadcnLook>) -> TextAreaBuilder {
        self.look_theme(theme)
    }

    fn surface(self, theme: &Arc<ShadcnLook>) -> TextAreaBuilder {
        self.template(Arc::new(ThemedTextAreaTemplate::new(theme.surface_textarea_theme())))
            .theme(theme.surface_textarea_theme())
    }

    fn primary(self, theme: &Arc<ShadcnLook>) -> TextAreaBuilder {
        self.template(theme.primary_textarea_template()).theme(theme.primary_textarea_theme())
    }
}

/// Applies a shadcn checkbox style to a [`CheckboxBuilder`].
pub trait ShadcnCheckboxStyleExt {
    fn look_style(self, theme: &Arc<ShadcnLook>, style: ShadcnButtonStyle) -> CheckboxBuilder;
    fn primary(self, theme: &Arc<ShadcnLook>) -> CheckboxBuilder;
    fn secondary(self, theme: &Arc<ShadcnLook>) -> CheckboxBuilder;
    fn outline(self, theme: &Arc<ShadcnLook>) -> CheckboxBuilder;
    fn ghost(self, theme: &Arc<ShadcnLook>) -> CheckboxBuilder;
}

impl ShadcnCheckboxStyleExt for CheckboxBuilder {
    fn look_style(self, theme: &Arc<ShadcnLook>, style: ShadcnButtonStyle) -> CheckboxBuilder {
        self.template(theme.checkbox_template(style))
    }

    fn primary(self, theme: &Arc<ShadcnLook>) -> CheckboxBuilder {
        self.look_style(theme, ShadcnButtonStyle::Primary)
    }

    fn secondary(self, theme: &Arc<ShadcnLook>) -> CheckboxBuilder {
        self.look_style(theme, ShadcnButtonStyle::Secondary)
    }

    fn outline(self, theme: &Arc<ShadcnLook>) -> CheckboxBuilder {
        self.look_style(theme, ShadcnButtonStyle::Outline)
    }

    fn ghost(self, theme: &Arc<ShadcnLook>) -> CheckboxBuilder {
        self.look_style(theme, ShadcnButtonStyle::Ghost)
    }
}

/// Applies a shadcn switch style to a [`SwitchBuilder`].
pub trait ShadcnSwitchStyleExt {
    fn look_style(self, theme: &Arc<ShadcnLook>, style: ShadcnButtonStyle) -> SwitchBuilder;
    fn primary(self, theme: &Arc<ShadcnLook>) -> SwitchBuilder;
    fn secondary(self, theme: &Arc<ShadcnLook>) -> SwitchBuilder;
    fn outline(self, theme: &Arc<ShadcnLook>) -> SwitchBuilder;
    fn ghost(self, theme: &Arc<ShadcnLook>) -> SwitchBuilder;
}

impl ShadcnSwitchStyleExt for SwitchBuilder {
    fn look_style(self, theme: &Arc<ShadcnLook>, style: ShadcnButtonStyle) -> SwitchBuilder {
        self.template(theme.switch_template(style))
    }

    fn primary(self, theme: &Arc<ShadcnLook>) -> SwitchBuilder {
        self.look_style(theme, ShadcnButtonStyle::Primary)
    }

    fn secondary(self, theme: &Arc<ShadcnLook>) -> SwitchBuilder {
        self.look_style(theme, ShadcnButtonStyle::Secondary)
    }

    fn outline(self, theme: &Arc<ShadcnLook>) -> SwitchBuilder {
        self.look_style(theme, ShadcnButtonStyle::Outline)
    }

    fn ghost(self, theme: &Arc<ShadcnLook>) -> SwitchBuilder {
        self.look_style(theme, ShadcnButtonStyle::Ghost)
    }
}

/// Applies a shadcn slider style to a [`SliderBuilder`].
pub trait ShadcnSliderStyleExt {
    fn look_style(self, theme: &Arc<ShadcnLook>, style: ShadcnButtonStyle) -> SliderBuilder;
    fn primary(self, theme: &Arc<ShadcnLook>) -> SliderBuilder;
    fn secondary(self, theme: &Arc<ShadcnLook>) -> SliderBuilder;
}

impl ShadcnSliderStyleExt for SliderBuilder {
    fn look_style(self, theme: &Arc<ShadcnLook>, style: ShadcnButtonStyle) -> SliderBuilder {
        self.template(theme.slider_template_with_style(style))
    }

    fn primary(self, theme: &Arc<ShadcnLook>) -> SliderBuilder {
        self.look_style(theme, ShadcnButtonStyle::Primary)
    }

    fn secondary(self, theme: &Arc<ShadcnLook>) -> SliderBuilder {
        self.look_style(theme, ShadcnButtonStyle::Secondary)
    }
}
