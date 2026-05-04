use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::behavior::SelectionItem;
use super::control::ComboBoxControl;
use super::template::{ComboBoxTemplate, default_combobox_template};
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};
use crate::controls::textfield::{TextFieldTemplate, default_textfield_template};

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

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ComboBoxControl> {
        cx.new(|cx| ComboBoxControl::from_builder(self, cx))
    }
}

pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> ComboBoxBuilder {
    ComboBoxBuilder::new(id, items)
}
