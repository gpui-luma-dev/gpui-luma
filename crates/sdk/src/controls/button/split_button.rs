//! Popup-menu preset: single-focus action face plus menu trigger.
//!
//! [`SplitButton`], [`SplitButtonBuilder`], and [`SplitButtonEvent`] are aliases of
//! [`crate::controls::popup_menu::PopupMenu`], [`crate::controls::popup_menu::PopupMenuBuilder`],
//! and [`crate::controls::popup_menu::PopupMenuEvent`]. This is not a separate LMTP control.

pub use crate::controls::popup_menu::{PopupMenu as SplitButton, PopupMenuBuilder as SplitButtonBuilder};
pub use crate::controls::popup_menu::PopupMenuEvent as SplitButtonEvent;
