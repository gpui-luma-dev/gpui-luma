use gpui::{Entity, SharedString};

use crate::controls::control_group::{ControlGroupBuilder, ControlGroupControl, ControlGroupItemLike};
use crate::infra::icon::SelectionStatusIcons;

use super::item_template::{default_listbox_item_template, default_listbox_item_template_with_icons};
use super::template::default_listbox_template;

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

impl ControlGroupItemLike for ListBoxItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn label(&self) -> &SharedString {
        &self.label
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

pub type ListBox = Entity<ControlGroupControl<ListBoxItem>>;

fn listbox_builder(id: impl Into<SharedString>) -> ControlGroupBuilder<ListBoxItem> {
    ControlGroupBuilder::new(id)
        .vertical()
        .template(default_listbox_template())
        .item_template(default_listbox_item_template())
}

pub fn new(id: impl Into<SharedString>) -> ControlGroupBuilder<ListBoxItem> {
    listbox_builder(id).single_required()
}

pub fn single(id: impl Into<SharedString>) -> ControlGroupBuilder<ListBoxItem> {
    listbox_builder(id).single_required()
}

pub fn multiple(id: impl Into<SharedString>) -> ControlGroupBuilder<ListBoxItem> {
    listbox_builder(id).multiple()
}

pub fn with_icons(id: impl Into<SharedString>, icons: SelectionStatusIcons) -> ControlGroupBuilder<ListBoxItem> {
    listbox_builder(id).single_required().item_template(default_listbox_item_template_with_icons(icons))
}
