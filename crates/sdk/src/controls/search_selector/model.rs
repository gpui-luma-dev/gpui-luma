use std::sync::Arc;

use gpui::{App, AppContext, Entity, IntoElement, SharedString};

use super::behavior::SelectionItem;
use super::control::SearchSelectorControl;
use super::item_template::{SearchSelectorItemRenderModel, SearchSelectorItemTemplate, make_search_selector_item_template};
use super::panel_template::{SearchSelectorPanelTemplate, default_search_selector_panel_template};
use super::template::{
    SearchSelectorItemsTemplate, SearchSelectorTemplate, default_search_selector_items_template,
    default_search_selector_template,
};
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};
use crate::controls::selector_panel::{SelectorItemsPanelAppearance, default_selector_items_panel_appearance};
use crate::controls::textfield::{TextFieldTemplate, TextFieldTheme, default_textfield_template, default_textfield_theme};
use crate::theme::{ControlSize, ThemeTokens};

pub type SearchSelectorPopupAppearanceProvider = Arc<dyn Fn() -> SelectorItemsPanelAppearance + Send + Sync + 'static>;

#[derive(Clone)]
pub struct SearchSelectorModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<SelectionItem>,
    pub(crate) placeholder: SharedString,
    pub(crate) search_placeholder: SharedString,
    pub(crate) enabled: bool,
    pub(crate) full_width: bool,
    pub(crate) clean_on_escape: bool,
    pub(crate) scrolling: bool,
    pub(crate) min_visible_rows: usize,
    pub(crate) max_visible_rows: usize,
    pub(crate) textfield_template: Arc<dyn TextFieldTemplate>,
    pub(crate) textfield_theme: Arc<dyn TextFieldTheme>,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) template: Arc<dyn SearchSelectorTemplate>,
    pub(crate) items_template: Arc<dyn SearchSelectorItemsTemplate>,
    pub(crate) panel_template: Arc<dyn SearchSelectorPanelTemplate>,
    pub(crate) item_template: Option<SearchSelectorItemTemplate<SelectionItem>>,
    pub(crate) popup_appearance_provider: SearchSelectorPopupAppearanceProvider,
}

pub struct SearchSelectorBuilder {
    pub(crate) model: SearchSelectorModel,
}

impl SearchSelectorBuilder {
    pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        Self {
            model: SearchSelectorModel {
                id: id.into(),
                items: items.into_iter().collect(),
                placeholder: SharedString::from("Select…"),
                search_placeholder: SharedString::from("Selection search"),
                enabled: true,
                full_width: true,
                clean_on_escape: true,
                scrolling: true,
                min_visible_rows: 1,
                max_visible_rows: 7,
                textfield_template: default_textfield_template(),
                textfield_theme: default_textfield_theme(),
                scrollbar_template: default_scrollbar_template(),
                template: default_search_selector_template(),
                items_template: default_search_selector_items_template(),
                panel_template: default_search_selector_panel_template(),
                item_template: None,
                popup_appearance_provider: Arc::new(|| {
                    default_selector_items_panel_appearance(&ThemeTokens::default(), ControlSize::Md)
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

    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.model.search_placeholder = placeholder.into();
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

    pub fn textfield_template(mut self, template: Arc<dyn TextFieldTemplate>) -> Self {
        self.model.textfield_template = template;
        self
    }

    pub fn textfield_theme(mut self, theme: Arc<dyn TextFieldTheme>) -> Self {
        self.model.textfield_theme = theme;
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

    pub fn popup_appearance_provider(mut self, provider: SearchSelectorPopupAppearanceProvider) -> Self {
        self.model.popup_appearance_provider = provider;
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

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<SearchSelectorControl> {
        cx.new(|cx| SearchSelectorControl::from_builder(self, cx))
    }
}

pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> SearchSelectorBuilder {
    SearchSelectorBuilder::new(id, items)
}
