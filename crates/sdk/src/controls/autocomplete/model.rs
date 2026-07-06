use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::behavior::SelectionItem;
use super::control::AutocompleteTextBoxControl;
use super::template::{
    AutocompleteItemsTemplate, AutocompleteTextBoxTemplate, default_autocomplete_items_template,
    default_autocomplete_textbox_template,
};
use crate::controls::selector_panel::{SelectorItemsPanelLook, default_selector_items_panel_look};
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};
use crate::controls::textfield::{TextFieldTemplate, default_textfield_template};
use crate::theme::{ControlSize, ThemeTokens};

pub type AutocompletePopupLookProvider = Arc<dyn Fn() -> SelectorItemsPanelLook + Send + Sync + 'static>;

#[derive(Clone)]
pub struct AutocompleteTextBoxModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<SelectionItem>,
    pub(crate) placeholder: SharedString,
    pub(crate) enabled: bool,
    pub(crate) full_width: bool,
    pub(crate) clean_on_escape: bool,
    pub(crate) scrolling: bool,
    pub(crate) textfield_template: Arc<dyn TextFieldTemplate>,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) template: Arc<dyn AutocompleteTextBoxTemplate>,
    pub(crate) items_template: Arc<dyn AutocompleteItemsTemplate>,
    pub(crate) popup_look_provider: AutocompletePopupLookProvider,
}

pub struct AutocompleteTextBoxBuilder {
    pub(crate) model: AutocompleteTextBoxModel,
}

impl AutocompleteTextBoxBuilder {
    pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        Self {
            model: AutocompleteTextBoxModel {
                id: id.into(),
                items: items.into_iter().collect(),
                placeholder: SharedString::from("Type to filter…"),
                enabled: true,
                full_width: true,
                clean_on_escape: true,
                scrolling: true,
                textfield_template: default_textfield_template(),
                scrollbar_template: default_scrollbar_template(),
                template: default_autocomplete_textbox_template(),
                items_template: default_autocomplete_items_template(),
                popup_look_provider: Arc::new(|| {
                    default_selector_items_panel_look(&ThemeTokens::default(), ControlSize::Md)
                }),
            },
        }
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.model.placeholder = placeholder.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.model.full_width = full_width;
        self
    }

    pub fn clean_on_escape(mut self, clean_on_escape: bool) -> Self {
        self.model.clean_on_escape = clean_on_escape;
        self
    }

    pub fn scrolling(mut self, scrolling: bool) -> Self {
        self.model.scrolling = scrolling;
        self
    }

    pub fn textfield_template(mut self, template: Arc<dyn TextFieldTemplate>) -> Self {
        self.model.textfield_template = template;
        self
    }

    pub fn scrollbar_template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = template;
        self
    }

    pub fn template(mut self, template: Arc<dyn AutocompleteTextBoxTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn items_template(mut self, template: Arc<dyn AutocompleteItemsTemplate>) -> Self {
        self.model.items_template = template;
        self
    }

    pub fn popup_look_provider(mut self, provider: AutocompletePopupLookProvider) -> Self {
        self.model.popup_look_provider = provider;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<AutocompleteTextBoxControl> {
        cx.new(|cx| AutocompleteTextBoxControl::from_builder(self, cx))
    }
}

pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> AutocompleteTextBoxBuilder {
    AutocompleteTextBoxBuilder::new(id, items)
}
