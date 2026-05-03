//! Popup selector control.
//!
//! # Examples
//!
//! Default built-in items:
//! ```ignore
//! use gpui_luma::controls::popup_selector::{PopupSelector, SelectorItem};
//!
//! let selector = PopupSelector::new("status")
//!     .items([
//!         SelectorItem::new("todo").label("To Do"),
//!         SelectorItem::new("doing").label("In Progress"),
//!         SelectorItem::new("done").label("Done"),
//!     ])
//!     .selected_id("doing");
//! ```
//!
//! Typed custom items:
//! ```ignore
//! use gpui::SharedString;
//! use gpui_luma::controls::popup_selector::{PopupSelector, SelectorItemLike};
//!
//! #[derive(Clone)]
//! struct SwatchItem {
//!     id: SharedString,
//!     label: SharedString,
//!     enabled: bool,
//! }
//!
//! impl SelectorItemLike for SwatchItem {
//!     fn id(&self) -> &SharedString { &self.id }
//!     fn is_enabled(&self) -> bool { self.enabled }
//!     fn label_text(&self) -> &SharedString { &self.label }
//! }
//!
//! let selector = PopupSelector::<SwatchItem>::new_typed("palette");
//! ```
mod control;
mod model;
mod template;

pub use control::{PopupSelector, PopupSelectorEvent};
pub use model::{
    PopupSelectorBuilder, PopupSelectorItemRenderModel, PopupSelectorItemTemplate, PopupSelectorModel,
    PopupSelectorPlacement, PopupSelectorRenderModel, SelectorItem, SelectorItemIcon, SelectorItemLike,
};
pub use template::{
    PopupSelectorTemplate, PopupSelectorTemplateHandlers, ThemedPopupSelectorTemplate, default_popup_selector_template,
};

pub use crate::controls::state::{ControlFocusState, MenuPath};
pub use crate::theme::InteractionState as PopupSelectorState;
