//! Keyed collection state and input binding for host-composed selectable lists.
//!
//! Hosts and looks own content, viewport composition, and event delivery. An
//! optional persistent control combines state and bindings; fixed-item geometry
//! and content-template contracts support look-owned builders. This module never
//! chooses appearance or depends on a particular look.

mod binding;
mod control;
mod drag_scroll;
mod item_template;
mod layout;
mod model;
mod scroll;

pub use scroll::ListBoxScrollHandle;
pub use control::{ListBoxControl, ListBoxInputHandler, ListBoxRenderParts};
pub use item_template::{ListBoxItemRenderModel, ListBoxItemTemplate, make_listbox_item_template};
pub use layout::{ListBoxFlow, ListBoxLayout};

pub use binding::{ListBoxAxis, ListBoxBinding};
pub use model::{
    ListBoxError, ListBoxEvent, ListBoxInput, ListBoxItemState, ListBoxNavigation, ListBoxSnapshot, ListBoxState,
    ListBoxUpdate, ListBoxVisibleItem, SelectionMode, SelectionPolicy, ListBoxSelectionModifiers,
};

#[cfg(test)]
mod selection_tests;
