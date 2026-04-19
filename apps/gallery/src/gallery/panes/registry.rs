use gpui::{AnyElement, Context, Subscription};

use crate::gallery::control::GalleryApp;

use super::{
    button, checkbox, context_menu, dropdown_menu, icon_button, introduction, progress, radio_group, scrollbar, search,
    settings, slider, switch, toggle_button,
};

pub(in crate::gallery) struct GalleryPanes {
    pub(super) button: button::ButtonPane,
    pub(super) icon_button: icon_button::IconButtonPane,
    pub(super) toggle_button: toggle_button::ToggleButtonPane,
    pub(super) switch: switch::SwitchPane,
    pub(super) checkbox: checkbox::CheckboxPane,
    pub(super) radio_group: radio_group::RadioGroupPane,
    pub(super) slider: slider::SliderPane,
    pub(super) scrollbar: scrollbar::ScrollbarPane,
    pub(super) dropdown_menu: dropdown_menu::DropdownMenuPane,
    pub(super) context_menu: context_menu::ContextMenuPane,
    pub(super) progress: progress::ProgressPane,
}

impl GalleryPanes {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            button: button::ButtonPane::new(cx),
            icon_button: icon_button::IconButtonPane::new(cx),
            toggle_button: toggle_button::ToggleButtonPane::new(cx),
            switch: switch::SwitchPane::new(cx),
            checkbox: checkbox::CheckboxPane::new(cx),
            radio_group: radio_group::RadioGroupPane::new(cx),
            slider: slider::SliderPane::new(cx),
            scrollbar: scrollbar::ScrollbarPane::new(cx),
            dropdown_menu: dropdown_menu::DropdownMenuPane::new(cx),
            context_menu: context_menu::ContextMenuPane::new(cx),
            progress: progress::ProgressPane::new(cx),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        self.button.subscribe(cx, subscriptions);
        self.icon_button.subscribe(cx, subscriptions);
        self.toggle_button.subscribe(cx, subscriptions);
        self.switch.subscribe(cx, subscriptions);
        self.checkbox.subscribe(cx, subscriptions);
        self.radio_group.subscribe(cx, subscriptions);
        self.slider.subscribe(cx, subscriptions);
        self.scrollbar.subscribe(cx, subscriptions);
        self.dropdown_menu.subscribe(cx, subscriptions);
        self.context_menu.subscribe(cx, subscriptions);
    }

    pub(in crate::gallery) fn render_selected(&self, selection: &str) -> AnyElement {
        match selection {
            "introduction" => introduction::render(),
            "button" => self.button.render(),
            "icon-button" => self.icon_button.render(),
            "toggle-button" => self.toggle_button.render(),
            "switch" => self.switch.render(),
            "checkbox" => self.checkbox.render(),
            "radio-group" => self.radio_group.render(),
            "slider" => self.slider.render(),
            "scrollbar" => self.scrollbar.render(),
            "dropdown-menu" => self.dropdown_menu.render(),
            "context-menu" => self.context_menu.render(),
            "progress" => self.progress.render(),
            "search" => search::render(),
            "settings" => settings::render(),
            _ => introduction::render(),
        }
    }
}
