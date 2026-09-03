use std::sync::Arc;

use gpui::{App, AppContext, Entity, IntoElement, ParentElement, SharedString, Styled};

use super::behavior::SelectionItem;
use super::control::SearchSelectorControl;
use crate::controls::autocomplete::{AutocompleteTheme, default_autocomplete_theme};
use super::item_template::{
    SearchSelectorItemRenderModel, SearchSelectorItemTemplate, item_template_with_modifier,
    make_search_selector_item_template,
};
use super::panel_template::{
    SearchSelectorPanelTemplate, default_search_selector_panel_template, panel_template_with_modifier,
};
use super::template::{
    SearchSelectorItemsTemplate, SearchSelectorTemplate, default_search_selector_items_template,
    default_search_selector_template, template_with_modifier,
};
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};
use crate::controls::selector::{SelectorTheme, SelectorTriggerStyle, default_selector_theme};
use crate::controls::selector_list::{SelectorItemsPanelLook, default_selector_items_panel_look};
use crate::controls::textfield::{TextFieldTemplate, default_textfield_template};
use crate::theme::{ControlSize, ThemeTokens};

pub type SearchSelectorPopupLookProvider = Arc<dyn Fn(ControlSize) -> SelectorItemsPanelLook + Send + Sync + 'static>;

#[derive(Clone)]
pub struct SearchSelectorModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<SelectionItem>,
    pub(crate) placeholder: SharedString,
    pub(crate) search_placeholder: SharedString,
    pub(crate) enabled: bool,
    pub(crate) invalid: bool,
    pub(crate) size: ControlSize,
    pub(crate) full_width: bool,
    pub(crate) clean_on_escape: bool,
    pub(crate) scrolling: bool,
    pub(crate) min_visible_rows: usize,
    pub(crate) max_visible_rows: usize,
    pub(crate) fill_popup_viewport: bool,
    pub(crate) textfield_template: Arc<dyn TextFieldTemplate>,
    pub(crate) selector_theme: Arc<dyn SelectorTheme>,
    pub(crate) trigger_style: SelectorTriggerStyle,
    pub(crate) without_elevation: bool,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) template: Arc<dyn SearchSelectorTemplate>,
    pub(crate) items_template: Arc<dyn SearchSelectorItemsTemplate>,
    pub(crate) panel_template: Arc<dyn SearchSelectorPanelTemplate>,
    pub(crate) item_template: Option<SearchSelectorItemTemplate<SelectionItem>>,
    pub(crate) autocomplete_theme: Arc<dyn AutocompleteTheme>,
    pub(crate) popup_look_provider: SearchSelectorPopupLookProvider,
}

pub struct SearchSelectorBuilder {
    pub(crate) model: SearchSelectorModel,
    pub(crate) initial_selected_id: Option<SharedString>,
}

impl SearchSelectorBuilder {
    pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        Self {
            initial_selected_id: None,
            model: SearchSelectorModel {
                id: id.into(),
                items: items.into_iter().collect(),
                placeholder: SharedString::from("Select…"),
                search_placeholder: SharedString::from("Selection search"),
                enabled: true,
                invalid: false,
                size: ControlSize::Md,
                full_width: true,
                clean_on_escape: true,
                scrolling: true,
                min_visible_rows: 1,
                max_visible_rows: 7,
                fill_popup_viewport: false,
                textfield_template: default_textfield_template(),
                selector_theme: default_selector_theme(),
                trigger_style: SelectorTriggerStyle::default(),
                without_elevation: false,
                scrollbar_template: default_scrollbar_template(),
                template: default_search_selector_template(),
                items_template: default_search_selector_items_template(),
                panel_template: default_search_selector_panel_template(),
                item_template: None,
                autocomplete_theme: default_autocomplete_theme(),
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

    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.model.search_placeholder = placeholder.into();
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

    pub fn selected_id(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.initial_selected_id = Some(selected_id.into());
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

    pub fn min_visible_rows(mut self, min_visible_rows: usize) -> Self {
        self.model.min_visible_rows = min_visible_rows.max(1);
        if self.model.max_visible_rows < self.model.min_visible_rows {
            self.model.max_visible_rows = self.model.min_visible_rows;
        }
        self
    }

    pub fn max_visible_rows(mut self, max_visible_rows: usize) -> Self {
        self.model.max_visible_rows = max_visible_rows.max(1);
        if self.model.min_visible_rows > self.model.max_visible_rows {
            self.model.min_visible_rows = self.model.max_visible_rows;
        }
        self
    }

    /// Expand the popup list to use available viewport space below the trigger,
    /// similar to `Selector` smart placement, instead of capping at `max_visible_rows`.
    pub fn fill_popup_viewport(mut self, fill_popup_viewport: bool) -> Self {
        self.model.fill_popup_viewport = fill_popup_viewport;
        self
    }

    pub fn textfield_template(mut self, template: Arc<dyn TextFieldTemplate>) -> Self {
        self.model.textfield_template = template;
        self
    }

    /// Theme for the read-only display trigger (Outline/Ghost command-button chrome).
    pub fn selector_theme(mut self, theme: Arc<dyn SelectorTheme>) -> Self {
        self.model.selector_theme = theme;
        self
    }

    pub fn trigger_style(mut self, style: SelectorTriggerStyle) -> Self {
        self.model.trigger_style = style;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.model.trigger_style = SelectorTriggerStyle::Ghost;
        self
    }

    pub fn without_elevation(mut self) -> Self {
        self.model.without_elevation = true;
        self
    }

    pub fn scrollbar_template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = template;
        self
    }

    pub fn template(mut self, template: Arc<dyn SearchSelectorTemplate>) -> Self {
        self.model.template = template;
        self
    }

    /// Transitional row-list template hook.
    ///
    /// Prefer `panel_template(...)` and `with_item_template(...)` for new code.
    #[deprecated(note = "Prefer panel_template(...) + with_item_template(...)")]
    pub fn items_template(mut self, template: Arc<dyn SearchSelectorItemsTemplate>) -> Self {
        self.model.items_template = template;
        self
    }

    pub fn panel_template(mut self, template: Arc<dyn SearchSelectorPanelTemplate>) -> Self {
        self.model.panel_template = template;
        self
    }

    pub fn autocomplete_theme(mut self, theme: Arc<dyn AutocompleteTheme>) -> Self {
        self.model.autocomplete_theme = theme;
        self
    }

    pub fn popup_look_provider(mut self, provider: SearchSelectorPopupLookProvider) -> Self {
        self.model.popup_look_provider = provider;
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&SearchSelectorItemRenderModel<'a, SelectionItem>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.item_template = Some(make_search_selector_item_template(template));
        self
    }

    pub fn with_item_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(
                gpui::AnyElement,
                &SearchSelectorItemRenderModel<'a, SelectionItem>,
                &mut App,
            ) -> gpui::AnyElement
            + Send
            + Sync
            + 'static,
    {
        let base = self.model.item_template.take().unwrap_or_else(|| {
            make_search_selector_item_template(|item: &SearchSelectorItemRenderModel<'_, SelectionItem>, _cx| {
                gpui::div().flex_1().child(item.item.label.clone())
            })
        });
        self.model.item_template = Some(item_template_with_modifier(base, modifier));
        self
    }

    pub fn with_panel_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::AnyElement, &mut App) -> gpui::AnyElement + Send + Sync + 'static,
    {
        self.model.panel_template = panel_template_with_modifier(Arc::clone(&self.model.panel_template), modifier);
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &mut App) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<SearchSelectorControl> {
        cx.new(|cx| SearchSelectorControl::from_builder(self, cx))
    }
}

pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> SearchSelectorBuilder {
    SearchSelectorBuilder::new(id, items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_item_template_modifier_creates_template_from_default_content() {
        let builder = SearchSelectorBuilder::new("search-selector-test", Vec::<SelectionItem>::new())
            .with_item_template_modifier(|content, _, _| content);

        assert!(builder.model.item_template.is_some());
    }

    #[test]
    fn with_panel_template_modifier_wraps_panel_template() {
        let template = default_search_selector_panel_template();
        let builder = SearchSelectorBuilder::new("search-selector-test", Vec::<SelectionItem>::new())
            .panel_template(template.clone())
            .with_panel_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.panel_template, &template));
    }

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_search_selector_template();
        let builder = SearchSelectorBuilder::new("search-selector-test", Vec::<SelectionItem>::new())
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn with_item_template_modifier_wraps_existing_item_template() {
        let builder = SearchSelectorBuilder::new("search-selector-test", Vec::<SelectionItem>::new())
            .with_item_template(|_item: &SearchSelectorItemRenderModel<'_, SelectionItem>, _cx| gpui::div());
        let item_template = builder.model.item_template.as_ref().unwrap().clone();
        let builder = builder.with_item_template_modifier(|content, _, _| content);

        assert!(!Arc::ptr_eq(builder.model.item_template.as_ref().unwrap(), &item_template));
    }

    #[test]
    fn template_panel_and_item_modifiers_compose_on_builder() {
        let template = default_search_selector_template();
        let panel_template = default_search_selector_panel_template();
        let builder = SearchSelectorBuilder::new("search-selector-test", Vec::<SelectionItem>::new())
            .template(template.clone())
            .panel_template(panel_template.clone())
            .with_item_template(|_item: &SearchSelectorItemRenderModel<'_, SelectionItem>, _cx| gpui::div());
        let item_template = builder.model.item_template.as_ref().unwrap().clone();
        let builder = builder
            .with_template_modifier(|element, _| element)
            .with_panel_template_modifier(|element, _| element)
            .with_item_template_modifier(|content, _, _| content);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
        assert!(!Arc::ptr_eq(&builder.model.panel_template, &panel_template));
        assert!(!Arc::ptr_eq(builder.model.item_template.as_ref().unwrap(), &item_template));
    }
}
