use std::sync::Arc;

use gpui::{AppContext, Entity, IntoElement, SharedString, div, prelude::*};

use super::{ListBoxTemplate, default_listbox_template};
use super::control::ListBoxControl;
use crate::controls::button_family::ButtonKind as ListBoxKind;
use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::listbox::{ControlFocusState, ListBoxItemState};
use crate::controls::presenter::{ControlPresenter, HasPresenter};
use crate::controls::listbox::default_listbox_item_button_template;
use crate::theme::ControlSize;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ListBoxSelectionMode {
    #[default]
    Single,
    Multiple,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ListBoxStateMode {
    #[default]
    Unmanaged,
    Managed,
}

#[derive(Clone, Debug)]
pub struct ListBoxItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) value: SharedString,
    pub(crate) enabled: bool,
}

impl ListBoxItem {
    pub fn new(id: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self { label: id.clone(), id, value: value.into(), enabled: true }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }

    pub fn value_text(&self) -> &SharedString {
        &self.value
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone)]
pub struct ListBoxModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<ListBoxItem>,
    pub(crate) default_selected_ids: Vec<SharedString>,
    pub(crate) managed_selected_ids: Option<Vec<SharedString>>,
    pub(crate) active_id: Option<SharedString>,
    pub(crate) selection_mode: ListBoxSelectionMode,
    pub(crate) state_mode: ListBoxStateMode,
    pub(crate) enabled: bool,
    pub(crate) size: ControlSize,
    pub(crate) kind: ListBoxKind,
    pub(crate) template: Arc<dyn ListBoxTemplate>,
    pub(crate) content: ListBoxContent,
    pub(crate) item_button_template: Option<ListBoxItemButtonTemplate>,
}

impl ListBoxModel {
    pub fn effective_selected_ids(&self) -> &[SharedString] {
        self.managed_selected_ids.as_deref().unwrap_or(self.default_selected_ids.as_slice())
    }

    pub fn set_managed_selected_ids<I, S>(&mut self, selected_ids: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.state_mode = ListBoxStateMode::Managed;
        self.managed_selected_ids = Some(selected_ids.into_iter().map(Into::into).collect());
    }

    pub fn clear_managed_selected_ids(&mut self) {
        self.managed_selected_ids = None;
    }
}

pub struct ListBoxRenderItem<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub value: &'a SharedString,
    pub selected: bool,
    pub enabled: bool,
    pub state: ListBoxItemState,
    pub content_model: ListBoxItemContentModel,
}

pub struct ListBoxRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: Vec<ListBoxRenderItem<'a>>,
    pub content: &'a ListBoxContent,
    pub item_button_template: Option<&'a ListBoxItemButtonTemplate>,
    pub selected_ids: &'a [SharedString],
    pub active_id: Option<&'a SharedString>,
    pub selection_mode: ListBoxSelectionMode,
    pub state_mode: ListBoxStateMode,
    pub enabled: bool,
    pub kind: ListBoxKind,
    pub size: ControlSize,
    pub focus: ControlFocusState,
}

#[derive(Clone, Debug)]
pub struct ListBoxItemContentModel {
    pub listbox_id: SharedString,
    pub item_id: SharedString,
    pub item_label: SharedString,
    pub item_value: SharedString,
    pub selected: bool,
    pub focused: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub enabled: bool,
    pub index: usize,
    pub sibling_count: usize,
    pub selection_mode: ListBoxSelectionMode,
}

pub type ListBoxContent = ControlPresenter<ListBoxItemContentModel>;

pub type ListBoxItemButtonRenderModel = ButtonRenderModel<bool>;
pub type ListBoxItemButtonTemplate = Arc<dyn ButtonTemplate<bool>>;

pub struct ListBoxBuilder {
    pub(crate) model: ListBoxModel,
}

impl ListBoxBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: ListBoxModel {
                id: id.clone(),
                items: Vec::new(),
                default_selected_ids: Vec::new(),
                managed_selected_ids: None,
                active_id: None,
                selection_mode: ListBoxSelectionMode::Single,
                state_mode: ListBoxStateMode::Unmanaged,
                enabled: true,
                size: ControlSize::Md,
                kind: ListBoxKind::Ghost,
                template: default_listbox_template(),
                content: Arc::new(move |m, _| div().child(m.item_label.clone()).into_any_element()),
                item_button_template: Some(default_listbox_item_button_template()),
            },
        }
    }

    pub fn item(mut self, item: ListBoxItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ListBoxItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn selection_mode(mut self, selection_mode: ListBoxSelectionMode) -> Self {
        self.model.selection_mode = selection_mode;
        self
    }

    pub fn single(self) -> Self {
        self.selection_mode(ListBoxSelectionMode::Single)
    }

    pub fn multiple(self) -> Self {
        self.selection_mode(ListBoxSelectionMode::Multiple)
    }

    pub fn selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.default_selected_ids = vec![selected_id.into()];
        self
    }

    pub fn selected_ids<I, S>(mut self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.default_selected_ids = selected_ids.into_iter().map(Into::into).collect();
        self
    }

    pub fn managed(mut self) -> Self {
        self.model.state_mode = ListBoxStateMode::Managed;
        self
    }

    pub fn unmanaged(mut self) -> Self {
        self.model.state_mode = ListBoxStateMode::Unmanaged;
        self.model.managed_selected_ids = None;
        self
    }

    pub fn managed_selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.state_mode = ListBoxStateMode::Managed;
        self.model.managed_selected_ids = Some(vec![selected_id.into()]);
        self
    }

    pub fn managed_selected_ids<I, S>(mut self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.state_mode = ListBoxStateMode::Managed;
        self.model.managed_selected_ids = Some(selected_ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn kind(mut self, kind: ListBoxKind) -> Self {
        self.model.kind = kind;
        self
    }

    pub fn item_button_template(mut self, template: ListBoxItemButtonTemplate) -> Self {
        self.model.item_button_template = Some(template);
        self
    }

    pub fn bool_button_template(self, template: ListBoxItemButtonTemplate) -> Self {
        self.item_button_template(template)
    }

    pub fn clear_item_button_template(mut self) -> Self {
        self.model.item_button_template = None;
        self
    }

    pub fn template(mut self, template: Arc<dyn ListBoxTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ListBoxControl> {
        cx.new(|cx| ListBoxControl::from_builder(self, cx))
    }
}

impl HasPresenter<ListBoxItemContentModel> for ListBoxBuilder {
    fn set_presenter(&mut self, content: ListBoxContent) {
        self.model.content = content;
    }
}
