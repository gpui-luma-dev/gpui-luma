//! ListBox is a [`control_group`](crate::controls::control_group) preset with list-row chrome,
//! not a separate interaction engine.
//!
//! LMTP here is template/theme/item chrome plus thin builders in [`model`]. Interaction
//! lives on `ControlGroupControl` — there is no dedicated `control.rs`.

mod item_template;
mod model;
mod template;
mod theme;

pub use model::{ListBox, ListBoxItem, multiple, new, single, with_icons};
pub use item_template::{default_listbox_item_template, default_listbox_item_template_with_icons};
pub use template::{
    ThemedListBoxTemplate, default_listbox_row_item_element_template, default_listbox_template,
    listbox_row_item_element_template_with_theme, listbox_template_with_theme, listbox_template_with_theme_and_size,
    shared_listbox_template,
};
pub use theme::{
    DefaultListBoxTheme, ListBoxListLook, ListBoxRowLook, ListBoxRowPalette, ListBoxTheme, default_listbox_theme,
};

pub use crate::controls::control_group::{
    ControlFocusState, ControlGroupBuilder, ControlGroupControl, ControlGroupEvent, ControlGroupItemState,
    ControlGroupStateMode, ControlSelectionMode,
};
