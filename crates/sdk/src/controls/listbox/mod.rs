mod control;
mod item_template;
mod model;
mod template;
mod theme;

pub use control::{ListBoxControl, ListBoxEvent};
pub use model::{
    ListBoxBuilder, ListBoxContent, ListBoxItem, ListBoxItemButtonRenderModel, ListBoxItemButtonTemplate,
    ListBoxItemContentModel, ListBoxModel, ListBoxRenderItem, ListBoxRenderModel, ListBoxSelectionMode,
    ListBoxStateMode,
};
pub use item_template::default_listbox_item_button_template;
pub use template::{
    ListBoxClickHandler, ListBoxHoverHandler, ListBoxMouseDownHandler, ListBoxMouseUpHandler, ListBoxTemplate,
    ListBoxTemplateHandlers, ThemedListBoxTemplate, default_listbox_template,
};
pub use theme::{
    DefaultListBoxTheme, LISTBOX_THEME_USAGE, ListBoxListAppearance, ListBoxRowAppearance, ListBoxTheme,
    default_listbox_theme,
};

pub use crate::controls::button_family::ButtonKind as ListBoxKind;
pub use crate::controls::state::{CompositeItemState as ListBoxItemState, ControlFocusState};
pub use crate::theme::ControlSize as ListBoxSize;

use gpui::{Entity, SharedString};

pub type ListBox = Entity<ListBoxControl>;

pub fn new(id: impl Into<SharedString>) -> ListBoxBuilder {
    ListBoxBuilder::new(id)
}

pub fn single(id: impl Into<SharedString>) -> ListBoxBuilder {
    ListBoxBuilder::new(id).single()
}

pub fn multiple(id: impl Into<SharedString>) -> ListBoxBuilder {
    ListBoxBuilder::new(id).multiple()
}
