use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::behavior::SelectionItem;
use super::control::AutocompleteTextBoxControl;
use super::theme::{AutocompleteTextBoxTheme, default_autocomplete_textbox_theme};
use super::template::{
    AutocompleteItemsTemplate, AutocompleteTextBoxTemplate, default_autocomplete_items_template,
    default_autocomplete_textbox_template, modified_autocomplete_items_template,
    modified_autocomplete_textbox_template,
};
use crate::controls::selector_list::{SelectorItemsPanelLook, default_selector_items_panel_look};
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};
use crate::controls::textfield::{TextFieldTemplate, default_textfield_template};
use crate::theme::{ControlSize, ThemeTokens};

pub type AutocompletePopupLookProvider = Arc<dyn Fn(ControlSize) -> SelectorItemsPanelLook + Send + Sync + 'static>;

#[derive(Clone)]
pub struct AutocompleteTextBoxModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<SelectionItem>,
    pub(crate) placeholder: SharedString,
    pub(crate) enabled: bool,
    pub(crate) invalid: bool,
    pub(crate) size: ControlSize,
    pub(crate) full_width: bool,
    pub(crate) clean_on_escape: bool,
    pub(crate) scrolling: bool,
    pub(crate) textfield_template: Arc<dyn TextFieldTemplate>,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) template: Arc<dyn AutocompleteTextBoxTemplate>,
    pub(crate) items_template: Arc<dyn AutocompleteItemsTemplate>,
    pub(crate) autocomplete_theme: Arc<dyn AutocompleteTextBoxTheme>,
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
                invalid: false,
                size: ControlSize::Md,
                full_width: true,
                clean_on_escape: true,
                scrolling: true,
                textfield_template: default_textfield_template(),
                scrollbar_template: default_scrollbar_template(),
                template: default_autocomplete_textbox_template(),
                items_template: default_autocomplete_items_template(),
                autocomplete_theme: default_autocomplete_textbox_theme(),
                popup_look_provider: Arc::new(|size| default_selector_items_panel_look(&ThemeTokens::default(), size)),
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

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.model.invalid = invalid;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
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

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &mut gpui::App) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = modified_autocomplete_textbox_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn items_template(mut self, template: Arc<dyn AutocompleteItemsTemplate>) -> Self {
        self.model.items_template = template;
        self
    }

    pub fn with_items_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(
                gpui::Stateful<gpui::Div>,
                &super::template::AutocompleteItemsRenderModel<'a>,
            ) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.items_template =
            modified_autocomplete_items_template(Arc::clone(&self.model.items_template), modifier);
        self
    }

    pub fn autocomplete_theme(mut self, theme: Arc<dyn AutocompleteTextBoxTheme>) -> Self {
        self.model.autocomplete_theme = theme;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_autocomplete_textbox_template();
        let builder = AutocompleteTextBoxBuilder::new("autocomplete-test", Vec::<SelectionItem>::new())
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn template_modifier_composes_with_items_template() {
        let template = default_autocomplete_textbox_template();
        let items_template = default_autocomplete_items_template();
        let builder = AutocompleteTextBoxBuilder::new("autocomplete-test", Vec::<SelectionItem>::new())
            .template(template.clone())
            .items_template(items_template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
        assert!(Arc::ptr_eq(&builder.model.items_template, &items_template));
    }

    #[test]
    fn with_items_template_modifier_wraps_items_template() {
        let items_template = default_autocomplete_items_template();
        let builder = AutocompleteTextBoxBuilder::new("autocomplete-test", Vec::<SelectionItem>::new())
            .items_template(items_template.clone())
            .with_items_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.items_template, &items_template));
    }
}
