//! Keyed collection state and input binding for host-composed selectable lists.
//!
//! The host owns rows, layout, viewport composition, and event delivery. An
//! optional scroll handle delivers reveal effects for host-composed stacks.
//! This module never chooses collection content or appearance.

mod binding;
mod drag_scroll;
mod model;
mod scroll;

pub use scroll::ListBoxScrollHandle;

pub use binding::{ListBoxAxis, ListBoxBinding};
pub use model::{
    ListBoxError, ListBoxEvent, ListBoxInput, ListBoxItemState, ListBoxNavigation, ListBoxSnapshot, ListBoxState,
    ListBoxUpdate, ListBoxVisibleItem, SelectionMode, SelectionPolicy, ListBoxSelectionModifiers,
};

#[cfg(test)]
mod selection_tests;
