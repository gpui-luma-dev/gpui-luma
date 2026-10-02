//! Overlay and menu controls.
//!
//! Spawnable: [`popup_menu`], [`context_menu`], [`popover_button`], [`overlay_window`],
//! [`slide_panel`].
//!
//! Not spawnable: [`floating_menu`] is shared menu **renderer** chrome (model/template/theme)
//! used by popup/context menus — there is no `FloatingMenu` entity.

pub mod context_menu;
pub mod floating_menu;
pub mod overlay_window;
pub mod popover_button;
pub mod popup_menu;
pub mod slide_panel;

pub mod tooltip;
