use gpui::{AppContext, SharedString};

use super::behavior::SelectionItem;
use super::control::{AutocompleteTextBox, AutocompleteTextBoxControl};
use crate::gallery::theme::GalleryThemePack;

#[derive(Clone)]
pub(super) struct AutocompleteTextBoxModel {
    pub id: SharedString,
    pub items: Vec<SelectionItem>,
    pub placeholder: SharedString,
    pub full_width: bool,
    pub clean_on_escape: bool,
}

pub(super) struct AutocompleteTextBoxBuilder {
    pub(super) model: AutocompleteTextBoxModel,
}

impl AutocompleteTextBoxBuilder {
    pub(super) fn new(id: impl Into<SharedString>, items: Vec<SelectionItem>) -> Self {
        Self {
            model: AutocompleteTextBoxModel {
                id: id.into(),
                items,
                placeholder: SharedString::from("Prompt: start typing…"),
                full_width: true,
                clean_on_escape: true,
            },
        }
    }

    pub(super) fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.model.placeholder = placeholder.into();
        self
    }

    pub(super) fn full_width(mut self, full_width: bool) -> Self {
        self.model.full_width = full_width;
        self
    }

    pub(super) fn clean_on_escape(mut self, clean_on_escape: bool) -> Self {
        self.model.clean_on_escape = clean_on_escape;
        self
    }

    pub(super) fn spawn(self, theme: GalleryThemePack, cx: &mut impl AppContext) -> AutocompleteTextBox {
        cx.new(|cx| AutocompleteTextBoxControl::from_builder(self, theme, cx))
    }
}

pub(super) fn autocomplete_textbox(
    id: impl Into<SharedString>,
    items: Vec<SelectionItem>,
) -> AutocompleteTextBoxBuilder {
    AutocompleteTextBoxBuilder::new(id, items)
}
