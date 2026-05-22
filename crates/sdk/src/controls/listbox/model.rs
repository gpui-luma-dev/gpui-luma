use gpui::SharedString;

use crate::controls::control_group::ControlGroupItemLike;

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
