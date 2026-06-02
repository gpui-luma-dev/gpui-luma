//! Theme factories and builder style helpers for Radix-themed controls.
//!
//! [`RadixThemeControlExt`] constructs controls with templates pre-bound from an
//! [`Arc<RadixTheme>`]. Builder style helpers ([`RadixButtonStyleExt`],
//! [`RadixCheckboxStyleExt`], [`RadixSwitchStyleExt`], [`RadixTextFieldExt`]) apply
//! Radix styles to existing builders.

use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use crate::controls::autocomplete::{self, AutocompleteTextBoxBuilder};
use crate::controls::button_group::{self, IconGroupBuilder};
use crate::controls::checkbox::{self, CheckboxBuilder};
use crate::controls::combobox::{self, ComboBoxBuilder};
use crate::controls::command::button::{Button, ButtonBuilder, ControlIcon};
use crate::controls::command::icon_button;
use crate::controls::control_group::{ControlGroupBuilder, ControlGroupItemLike};
use crate::controls::list_view::{self, ListViewBuilder};
use crate::controls::listbox::{self, ListBoxItem};
use crate::controls::navigation_sidebar::{NavigationSidebar, NavigationSidebarBuilder};
use crate::controls::context_menu::ContextMenu;
use crate::controls::popup_menu::PopupMenu;
use crate::controls::progress::{self, ProgressBuilder};
use crate::controls::radio_button;
use crate::controls::radio_group::{self, RadioGroupBuilder, RadioGroupLayout, radio_group_buttons_template};
use crate::controls::scrollbar::{self, ScrollbarBuilder};
use crate::controls::selector::{Selector, SelectorBuilder, SelectorItem};
use crate::controls::selection_panel::{SelectionPanelControl, SelectionPanelItem};
use crate::controls::search_selector::{self, SearchSelectorBuilder};
use crate::controls::slider::{self, SliderBuilder};
use crate::controls::switch::{self, SwitchBuilder};
use crate::controls::accordion::AccordionBuilder;
use crate::controls::tabs_navigation::{TabsNavigation, TabsNavigationBuilder};
use crate::controls::textarea::{self, TextAreaBuilder};
use crate::controls::textfield::{self, TextFieldBuilder};
use crate::controls::toggle;
use crate::theme::ControlSize;

use super::{RadixButtonStyle, RadixTheme};

/// Constructs SDK controls from a shared [`RadixTheme`], with Radix templates applied.
pub trait RadixThemeControlExt {
    fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn primary_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn secondary_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn outline_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn ghost_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;

    fn checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;
    fn primary_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;
    fn secondary_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;
    fn outline_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;
    fn ghost_checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;

    fn switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;
    fn primary_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;
    fn secondary_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;
    fn outline_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;
    fn ghost_switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;

    fn radio(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool>;
    fn primary_radio(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool>;
    fn secondary_radio(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool>;
    fn outline_radio(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool>;
    fn ghost_radio(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool>;

    fn primary_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()>;
    fn secondary_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()>;
    fn outline_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()>;
    fn ghost_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()>;

    fn toggle(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool>;
    fn primary_toggle(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool>;
    fn secondary_toggle(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool>;
    fn outline_toggle(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool>;
    fn ghost_toggle(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool>;

    fn popup_menu(&self, id: impl Into<SharedString>) -> crate::controls::popup_menu::PopupMenuBuilder;
    fn context_menu(&self, id: impl Into<SharedString>) -> crate::controls::context_menu::ContextMenuBuilder;
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
    fn slider(&self, id: impl Into<SharedString>) -> SliderBuilder;
    fn progress(&self, id: impl Into<SharedString>) -> ProgressBuilder;
    fn scrollbar(&self, id: impl Into<SharedString>) -> ScrollbarBuilder;
    fn selector(&self, id: impl Into<SharedString>) -> SelectorBuilder<SelectorItem>;
    fn tabs_navigation(&self, id: impl Into<SharedString>) -> TabsNavigationBuilder;
    fn accordion(&self, id: impl Into<SharedString>) -> AccordionBuilder;
    fn navigation_sidebar(&self, id: impl Into<SharedString>) -> NavigationSidebarBuilder;

    fn autocomplete(
        &self,
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = crate::controls::autocomplete::SelectionItem>,
    ) -> AutocompleteTextBoxBuilder;
    fn combobox(
        &self,
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = crate::controls::combobox::SelectionItem>,
    ) -> ComboBoxBuilder;
    fn search_selector(
        &self,
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = crate::controls::search_selector::SelectionItem>,
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
    fn selection_panel(
        &self,
        id: impl Into<SharedString>,
        cx: &mut impl AppContext,
    ) -> Entity<SelectionPanelControl<SelectionPanelItem>>;

    fn textfield(&self, id: impl Into<SharedString>) -> TextFieldBuilder;
    fn textarea(&self, id: impl Into<SharedString>) -> TextAreaBuilder;
}

impl RadixThemeControlExt for Arc<RadixTheme> {
    fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(RadixButtonStyle::Secondary))
    }

    fn primary_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(RadixButtonStyle::Primary))
    }

    fn secondary_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(RadixButtonStyle::Secondary))
    }

    fn outline_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(RadixButtonStyle::Outline))
    }

    fn ghost_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(RadixButtonStyle::Ghost))
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

    fn radio(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool> {
        radio_button::new(id).template(self.radio_button_template(RadixButtonStyle::Primary))
    }

    fn primary_radio(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool> {
        radio_button::new(id).template(self.radio_button_template(RadixButtonStyle::Primary))
    }

    fn secondary_radio(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool> {
        radio_button::new(id).template(self.radio_button_template(RadixButtonStyle::Secondary))
    }

    fn outline_radio(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool> {
        radio_button::new(id).template(self.radio_button_template(RadixButtonStyle::Outline))
    }

    fn ghost_radio(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool> {
        radio_button::new(id).template(self.radio_button_template(RadixButtonStyle::Ghost))
    }

    fn primary_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()> {
        icon_button::new(id, icon).template(self.button_template(RadixButtonStyle::Primary))
    }

    fn secondary_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()> {
        icon_button::new(id, icon).template(self.button_template(RadixButtonStyle::Secondary))
    }

    fn outline_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()> {
        icon_button::new(id, icon).template(self.button_template(RadixButtonStyle::Outline))
    }

    fn ghost_icon_button(&self, id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()> {
        icon_button::new(id, icon).template(self.button_template(RadixButtonStyle::Ghost))
    }

    fn toggle(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool> {
        toggle::new(id).template(self.toggle_template(RadixButtonStyle::Secondary))
    }

    fn primary_toggle(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool> {
        toggle::new(id).template(self.toggle_template(RadixButtonStyle::Primary))
    }

    fn secondary_toggle(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool> {
        toggle::new(id).template(self.toggle_template(RadixButtonStyle::Secondary))
    }

    fn outline_toggle(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool> {
        toggle::new(id).template(self.toggle_template(RadixButtonStyle::Outline))
    }

    fn ghost_toggle(&self, id: impl Into<SharedString>) -> ButtonBuilder<bool> {
        toggle::new(id).template(self.toggle_template(RadixButtonStyle::Ghost))
    }

    fn popup_menu(&self, id: impl Into<SharedString>) -> crate::controls::popup_menu::PopupMenuBuilder {
        PopupMenu::new(id).template(self.popup_menu_template())
    }

    fn context_menu(&self, id: impl Into<SharedString>) -> crate::controls::context_menu::ContextMenuBuilder {
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
        radio_group::new(id).template(radio_group_buttons_template(
            self.radio_button_template(RadixButtonStyle::Primary),
            RadioGroupLayout::Vertical,
        ))
    }

    fn radio_group_horizontal<T>(&self, id: impl Into<SharedString>) -> RadioGroupBuilder<T>
    where
        T: ControlGroupItemLike + Clone + Send + Sync + 'static,
    {
        radio_group::horizontal(id).template(radio_group_buttons_template(
            self.radio_button_template(RadixButtonStyle::Primary),
            RadioGroupLayout::Horizontal,
        ))
    }

    fn slider(&self, id: impl Into<SharedString>) -> SliderBuilder {
        slider::new(id).template(self.slider_template())
    }

    fn progress(&self, id: impl Into<SharedString>) -> ProgressBuilder {
        progress::new(id).template(self.progress_template())
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
        RadixTheme::accordion(self, id)
    }

    fn navigation_sidebar(&self, id: impl Into<SharedString>) -> NavigationSidebarBuilder {
        NavigationSidebar::new(id)
            .template(self.navigation_sidebar_template())
            .scrollbar_template(self.scrollbar_template())
    }

    fn autocomplete(
        &self,
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = crate::controls::autocomplete::SelectionItem>,
    ) -> AutocompleteTextBoxBuilder {
        autocomplete::new(id, items)
            .textfield_template(self.textfield_template())
            .scrollbar_template(self.scrollbar_template())
    }

    fn combobox(
        &self,
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = crate::controls::combobox::SelectionItem>,
    ) -> ComboBoxBuilder {
        let theme = Arc::clone(self);
        combobox::new(id, items)
            .textfield_template(self.textfield_template())
            .scrollbar_template(self.scrollbar_template())
            .popup_appearance_provider(Arc::new(move || theme.selector_items_panel_appearance(ControlSize::Md)))
    }

    fn search_selector(
        &self,
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = crate::controls::search_selector::SelectionItem>,
    ) -> SearchSelectorBuilder {
        let theme = Arc::clone(self);
        search_selector::new(id, items)
            .textfield_template(self.textfield_template())
            .textfield_theme(self.textfield_theme())
            .scrollbar_template(self.scrollbar_template())
            .popup_appearance_provider(Arc::new(move || theme.selector_items_panel_appearance(ControlSize::Md)))
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

    fn selection_panel(
        &self,
        id: impl Into<SharedString>,
        cx: &mut impl AppContext,
    ) -> Entity<SelectionPanelControl<SelectionPanelItem>> {
        let theme = Arc::clone(self);
        let panel = SelectionPanelControl::new(id, cx);
        panel.update(cx, |panel, cx| {
            panel.with_scrollbar_template(theme.scrollbar_template(), cx);
            panel.set_appearance_provider(Arc::new(move |size| theme.selection_panel_appearance(size)), cx);
        });
        panel
    }

    fn textfield(&self, id: impl Into<SharedString>) -> TextFieldBuilder {
        textfield::new(id).template(self.textfield_template())
    }

    fn textarea(&self, id: impl Into<SharedString>) -> TextAreaBuilder {
        textarea::TextArea::new(id).template(self.textarea_template()).theme(self.textarea_theme())
    }
}

/// Applies a Radix button style to a [`ButtonBuilder`].
pub trait RadixButtonStyleExt {
    fn radix_style(self, theme: &Arc<RadixTheme>, style: RadixButtonStyle) -> ButtonBuilder<()>;
    fn primary(self, theme: &Arc<RadixTheme>) -> ButtonBuilder<()>;
    fn secondary(self, theme: &Arc<RadixTheme>) -> ButtonBuilder<()>;
    fn outline(self, theme: &Arc<RadixTheme>) -> ButtonBuilder<()>;
    fn ghost(self, theme: &Arc<RadixTheme>) -> ButtonBuilder<()>;
}

impl RadixButtonStyleExt for ButtonBuilder<()> {
    fn radix_style(self, theme: &Arc<RadixTheme>, style: RadixButtonStyle) -> ButtonBuilder<()> {
        self.template(theme.button_template(style))
    }

    fn primary(self, theme: &Arc<RadixTheme>) -> ButtonBuilder<()> {
        self.radix_style(theme, RadixButtonStyle::Primary)
    }

    fn secondary(self, theme: &Arc<RadixTheme>) -> ButtonBuilder<()> {
        self.radix_style(theme, RadixButtonStyle::Secondary)
    }

    fn outline(self, theme: &Arc<RadixTheme>) -> ButtonBuilder<()> {
        self.radix_style(theme, RadixButtonStyle::Outline)
    }

    fn ghost(self, theme: &Arc<RadixTheme>) -> ButtonBuilder<()> {
        self.radix_style(theme, RadixButtonStyle::Ghost)
    }
}

/// Binds a Radix text field template on a [`TextFieldBuilder`].
pub trait RadixTextFieldExt {
    fn radix_theme(self, theme: &Arc<RadixTheme>) -> TextFieldBuilder;
}

impl RadixTextFieldExt for TextFieldBuilder {
    fn radix_theme(self, theme: &Arc<RadixTheme>) -> TextFieldBuilder {
        self.template(theme.textfield_template())
    }
}

/// Applies a Radix checkbox style to a [`CheckboxBuilder`].
pub trait RadixCheckboxStyleExt {
    fn radix_style(self, theme: &Arc<RadixTheme>, style: RadixButtonStyle) -> CheckboxBuilder;
    fn primary(self, theme: &Arc<RadixTheme>) -> CheckboxBuilder;
    fn secondary(self, theme: &Arc<RadixTheme>) -> CheckboxBuilder;
    fn outline(self, theme: &Arc<RadixTheme>) -> CheckboxBuilder;
    fn ghost(self, theme: &Arc<RadixTheme>) -> CheckboxBuilder;
}

impl RadixCheckboxStyleExt for CheckboxBuilder {
    fn radix_style(self, theme: &Arc<RadixTheme>, style: RadixButtonStyle) -> CheckboxBuilder {
        self.template(theme.checkbox_template(style))
    }

    fn primary(self, theme: &Arc<RadixTheme>) -> CheckboxBuilder {
        self.radix_style(theme, RadixButtonStyle::Primary)
    }

    fn secondary(self, theme: &Arc<RadixTheme>) -> CheckboxBuilder {
        self.radix_style(theme, RadixButtonStyle::Secondary)
    }

    fn outline(self, theme: &Arc<RadixTheme>) -> CheckboxBuilder {
        self.radix_style(theme, RadixButtonStyle::Outline)
    }

    fn ghost(self, theme: &Arc<RadixTheme>) -> CheckboxBuilder {
        self.radix_style(theme, RadixButtonStyle::Ghost)
    }
}

/// Applies a Radix switch style to a [`SwitchBuilder`].
pub trait RadixSwitchStyleExt {
    fn radix_style(self, theme: &Arc<RadixTheme>, style: RadixButtonStyle) -> SwitchBuilder;
    fn primary(self, theme: &Arc<RadixTheme>) -> SwitchBuilder;
    fn secondary(self, theme: &Arc<RadixTheme>) -> SwitchBuilder;
    fn outline(self, theme: &Arc<RadixTheme>) -> SwitchBuilder;
    fn ghost(self, theme: &Arc<RadixTheme>) -> SwitchBuilder;
}

impl RadixSwitchStyleExt for SwitchBuilder {
    fn radix_style(self, theme: &Arc<RadixTheme>, style: RadixButtonStyle) -> SwitchBuilder {
        self.template(theme.switch_template(style))
    }

    fn primary(self, theme: &Arc<RadixTheme>) -> SwitchBuilder {
        self.radix_style(theme, RadixButtonStyle::Primary)
    }

    fn secondary(self, theme: &Arc<RadixTheme>) -> SwitchBuilder {
        self.radix_style(theme, RadixButtonStyle::Secondary)
    }

    fn outline(self, theme: &Arc<RadixTheme>) -> SwitchBuilder {
        self.radix_style(theme, RadixButtonStyle::Outline)
    }

    fn ghost(self, theme: &Arc<RadixTheme>) -> SwitchBuilder {
        self.radix_style(theme, RadixButtonStyle::Ghost)
    }
}
