//! Spawnable control families. Shared seams live in [`crate::infra`] and [`crate::motion`].

pub mod button;
pub mod choice;
pub mod layout;
pub mod navigation;
pub mod overlay;
pub mod range;
pub mod selection;
pub mod text;

pub use button::icon_button;
pub use button::family as button_family;
pub use button::split_button;

pub use choice::{checkbox, control_group, icon_group, listbox, radio_button, radio_group, switch, toggle, toolbar};

pub use text::{textarea, textfield};

pub use overlay::{context_menu, floating_menu, overlay_window, popover_button, popup_menu, slide_panel};

pub use selection::{autocomplete, combobox, table, search_selector, selection_panel, selector, selector_list};

pub use navigation::{accordion, pager, sidebar, stepper, tabs, tree_view};

pub use layout::{dock_splitter, popup_scroll_surface, resizable_panels, scroll_container, scrollbar, split_view};
pub use layout::{ScrollbarAutoHideActivate, ScrollbarPlacement, ScrollbarVisibility};

pub use range::{progress, slider};
