use std::sync::Arc;

use gpui::{App, AppContext, Entity, IntoElement, SharedString};

use super::behavior::SelectionItem;
use super::control::ComboBoxControl;
use super::item_template::{ComboBoxItemRenderModel, ComboBoxItemTemplate, make_combobox_item_template};
use super::panel_template::{ComboBoxPanelTemplate, default_combobox_panel_template};
use super::template::{ComboBoxItemsTemplate, ComboBoxTemplate, default_combobox_items_template, default_combobox_template};
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};
use crate::controls::selector_panel::{SelectorItemsPanelAppearance, default_selector_items_panel_appearance};
use crate::controls::textfield::{TextFieldTemplate, default_textfield_template};
use crate::theme::{ControlSize, ThemeTokens};

pub type ComboBoxPopupAppearanceProvider = Arc<dyn Fn() -> SelectorItemsPanelAppearance + Send + Sync + 'static>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum TypingPolicy {
    #[default]
    Flexible,
    Strict,
}

#[derive(Clone)]
pub struct ComboBoxModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<SelectionItem>,
    pub(crate) placeholder: SharedString,
    pub(crate) full_width: bool,
    pub(crate) clean_on_escape: bool,
    pub(crate) typing_policy: TypingPolicy,
    pub(crate) show_down_arrow: bool,
    pub(crate) show_clear_button: bool,
    pub(crate) scrolling: bool,
    pub(crate) min_visible_rows: usize,
    pub(crate) max_visible_rows: usize,
    pub(crate) textfield_template: Arc<dyn TextFieldTemplate>,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) template: Arc<dyn ComboBoxTemplate>,
    pub(crate) items_template: Arc<dyn ComboBoxItemsTemplate>,
    pub(crate) panel_template: Arc<dyn ComboBoxPanelTemplate>,
    pub(crate) item_template: Option<ComboBoxItemTemplate<SelectionItem>>,
    pub(crate) popup_appearance_provider: ComboBoxPopupAppearanceProvider,
}

pub struct ComboBoxBuilder {
    pub(crate) model: ComboBoxModel,
}

impl ComboBoxBuilder {
    pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        Self {
            model: ComboBoxModel {
                id: id.into(),
                items: items.into_iter().collect(),
                placeholder: SharedString::from("Select…"),
                full_width: true,
                clean_on_escape: true,
                typing_policy: TypingPolicy::Flexible,
                show_down_arrow: true,
                show_clear_button: true,
                scrolling: true,
                min_visible_rows: 1,
                max_visible_rows: 7,
                textfield_template: default_textfield_template(),
                scrollbar_template: default_scrollbar_template(),
                template: default_combobox_template(),
                items_template: default_combobox_items_template(),
                panel_template: default_combobox_panel_template(),
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

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.model.full_width = full_width;
        self
    }

    pub fn clean_on_escape(mut self, clean_on_escape: bool) -> Self {
        self.model.clean_on_escape = clean_on_escape;
        self
    }

    pub fn typing_policy(mut self, typing_policy: TypingPolicy) -> Self {
        self.model.typing_policy = typing_policy;
        self
    }

    pub fn show_down_arrow(mut self, show_down_arrow: bool) -> Self {
        self.model.show_down_arrow = show_down_arrow;
        self
    }

    pub fn show_clear_button(mut self, show_clear_button: bool) -> Self {
        self.model.show_clear_button = show_clear_button;
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

    pub fn scrollbar_template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = template;
        self
    }

    pub fn template(mut self, template: Arc<dyn ComboBoxTemplate>) -> Self {
        self.model.template = template;
        self
    }

    /// Transitional row-list template hook.
    ///
    /// Prefer `panel_template(...)` and `with_item_template(...)` for new code.
    pub fn items_template(mut self, template: Arc<dyn ComboBoxItemsTemplate>) -> Self {
        self.model.items_template = template;
        self
    }

    pub fn panel_template(mut self, template: Arc<dyn ComboBoxPanelTemplate>) -> Self {
        self.model.panel_template = template;
        self
    }

    pub fn popup_appearance_provider(mut self, provider: ComboBoxPopupAppearanceProvider) -> Self {
        self.model.popup_appearance_provider = provider;
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&ComboBoxItemRenderModel<'a, SelectionItem>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.item_template = Some(make_combobox_item_template(template));
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ComboBoxControl> {
        cx.new(|cx| ComboBoxControl::from_builder(self, cx))
    }
}

pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> ComboBoxBuilder {
    ComboBoxBuilder::new(id, items)
}
