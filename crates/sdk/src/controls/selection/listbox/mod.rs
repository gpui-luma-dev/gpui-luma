//! Keyed collection state and input binding for host-composed selectable lists.
//!
//! The host owns rows, layout, scrolling, and event delivery. This module never
//! renders a collection or chooses its appearance.

mod binding;
mod model;

pub use binding::{ListBoxAxis, ListBoxBinding};
pub use model::{
    ListBoxError, ListBoxEvent, ListBoxInput, ListBoxItemState, ListBoxNavigation, ListBoxSnapshot, ListBoxState,
    ListBoxUpdate, ListBoxVisibleItem, SelectionMode,
};
