mod item_template;
mod model;
mod template;
mod theme;

pub use model::ListBoxItem;
pub use item_template::default_listbox_item_template;
pub use template::{
    ThemedListBoxTemplate, default_listbox_row_item_element_template, default_listbox_template,
    listbox_row_item_element_template_with_theme, listbox_template_with_theme, listbox_template_with_theme_and_size,
    shared_listbox_template,
};
pub use theme::{
    DefaultListBoxTheme, ListBoxListLook, ListBoxRowLook, ListBoxRowPalette, ListBoxTheme, default_listbox_theme,
};

pub use crate::controls::control_group::{
    ControlGroupBuilder, ControlGroupControl, ControlGroupEvent, ControlGroupItemState, ControlGroupStateMode,
    ControlSelectionMode, ControlFocusState,
};

use gpui::{Entity, SharedString};

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
