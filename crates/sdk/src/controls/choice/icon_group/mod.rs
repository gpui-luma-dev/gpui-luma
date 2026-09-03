//! Icon toolbar preset over [`control_group`](crate::controls::control_group).
//!
//! Not a separate LMTP engine — types and builders are aliases/presets over
//! [`ControlGroupControl`]. Prefer this module for themed icon toolbars;
//! use `control_group` directly for custom layouts.

mod model;

pub use model::{
    IconGroup, IconGroupBuilder, IconGroupEvent, IconGroupItem, IconGroupItemLike, horizontal, icon_toolbar,
    icon_toolbar_multiple, new,
};
