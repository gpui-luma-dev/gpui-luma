//! Convenient imports for Radix-themed control construction.
//!
//! ```ignore
//! use gpui_luma::theme::radix::prelude::*;
//!
//! let submit = radix_theme.primary_button("submit").label("Submit").spawn(cx);
//! let cancel = Button::new("cancel").label("Cancel").secondary(&radix_theme).spawn(cx);
//! ```

pub use super::controls::{
    RadixButtonStyleExt, RadixCheckboxStyleExt, RadixSwitchStyleExt, RadixTextFieldExt, RadixThemeControlExt,
};
pub use super::{RadixButtonStyle, RadixTheme};
