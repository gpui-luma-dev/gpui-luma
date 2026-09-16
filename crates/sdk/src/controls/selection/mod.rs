//! Selection family: spawnable pickers plus shared list chrome.
//!
//! Three roles (do not collapse these names):
//! - [`selector`] — popup dropdown control (trigger + anchored list)
//! - [`selection_panel`] — listbox-in-a-panel control (always-visible option list)
//! - [`selector_list`] — shared item-list template/chrome used by selector, combobox,
//!   and autocomplete; **not** a spawnable control
//!
//! Related spawnables in this family: [`combobox`], [`autocomplete`], [`search_selector`],
//! [`table`].

mod behavior;
mod text_selection;

pub mod autocomplete;
pub mod combobox;
pub mod table;
pub mod search_selector;
pub mod selection_panel;
pub mod selector;
pub mod selector_list;
