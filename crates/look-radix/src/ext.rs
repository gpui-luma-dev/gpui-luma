//! Look-local factory helpers. Apps use these instead of editing the SDK.

use std::sync::Arc;

use gpui::SharedString;
use luma::controls::button::{Button, ButtonBuilder, DefaultButtonTemplate};
use luma::controls::checkbox::{self, CheckboxBuilder, ThemedCheckboxTemplate};
use luma::controls::overlay_window::{self, OverlayWindowBuilder, ThemedOverlayWindowTemplate};
use luma::controls::popup_menu::{PopupMenu, PopupMenuBuilder, ThemedPopupMenuTemplate};
use luma::controls::switch::{self, SwitchBuilder, ThemedSwitchTemplate};
use luma::controls::tabs::{Tabs, TabsBuilder, ThemedTabsTemplate};
use luma::controls::textfield::{self, TextFieldBuilder, ThemedTextFieldTemplate};
use luma::controls::toggle::{self, ToggleBuilder};

use crate::button::{RadixButtonRecipe, button_family_theme};
use crate::checkbox::checkbox_theme;
use crate::look::RadixLook;
use crate::overlay_window::overlay_window_theme;
use crate::popup_menu::popup_menu_theme;
use crate::switch::switch_theme;
use crate::tabs::tabs_theme;
use crate::textfield::textfield_theme;
use crate::toggle::toggle_template;

/// Constructs SDK controls pre-bound to [`RadixLook`] recipes.
pub trait RadixLookControlExt {
    fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn solid_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn soft_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn outline_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn ghost_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn textfield(&self, id: impl Into<SharedString>) -> TextFieldBuilder;
    fn checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;
    fn switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;
    fn toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder;
    fn tabs(&self, id: impl Into<SharedString>) -> TabsBuilder;
    fn popup_menu(&self, id: impl Into<SharedString>) -> PopupMenuBuilder;
    fn solid_popup_menu(&self, id: impl Into<SharedString>) -> PopupMenuBuilder;
    fn overlay_window(&self, id: impl Into<SharedString>) -> OverlayWindowBuilder;
}

impl RadixLookControlExt for Arc<RadixLook> {
    fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        self.solid_button(id)
    }

    fn solid_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(Arc::new(DefaultButtonTemplate::new(button_family_theme(
            Arc::clone(self),
            RadixButtonRecipe::Solid,
        ))))
    }

    fn soft_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(Arc::new(DefaultButtonTemplate::new(button_family_theme(
            Arc::clone(self),
            RadixButtonRecipe::Soft,
        ))))
    }

    fn outline_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(Arc::new(DefaultButtonTemplate::new(button_family_theme(
            Arc::clone(self),
            RadixButtonRecipe::Outline,
        ))))
    }

    fn ghost_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(Arc::new(DefaultButtonTemplate::new(button_family_theme(
            Arc::clone(self),
            RadixButtonRecipe::Ghost,
        ))))
    }

    fn textfield(&self, id: impl Into<SharedString>) -> TextFieldBuilder {
        let theme = textfield_theme(Arc::clone(self));
        textfield::new(id).template(Arc::new(ThemedTextFieldTemplate::new(theme)))
    }

    fn checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder {
        checkbox::new(id).template(Arc::new(ThemedCheckboxTemplate::new(checkbox_theme(Arc::clone(self)))))
    }

    fn switch(&self, id: impl Into<SharedString>) -> SwitchBuilder {
        switch::new(id).template(Arc::new(ThemedSwitchTemplate::new(switch_theme(Arc::clone(self)))))
    }

    fn toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder {
        toggle::new(id).template(toggle_template(Arc::clone(self)))
    }

    fn tabs(&self, id: impl Into<SharedString>) -> TabsBuilder {
        Tabs::new(id).template(Arc::new(ThemedTabsTemplate::new(tabs_theme(Arc::clone(self)))))
    }

    fn popup_menu(&self, id: impl Into<SharedString>) -> PopupMenuBuilder {
        PopupMenu::new(id).template(Arc::new(ThemedPopupMenuTemplate::new(popup_menu_theme(Arc::clone(self)))))
    }

    fn solid_popup_menu(&self, id: impl Into<SharedString>) -> PopupMenuBuilder {
        self.popup_menu(id).trigger_style(luma::controls::popup_menu::PopupMenuTriggerStyle::Primary)
    }

    fn overlay_window(&self, id: impl Into<SharedString>) -> OverlayWindowBuilder {
        overlay_window::overlay_window(id)
            .template(Arc::new(ThemedOverlayWindowTemplate::new(overlay_window_theme(Arc::clone(self)))))
    }
}
