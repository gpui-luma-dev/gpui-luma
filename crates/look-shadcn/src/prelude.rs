//! Convenient imports for shadcn look styling.
//!
//! ```ignore
//! use gpui_luma_look_shadcn::prelude::*;
//!
//! with_look(&look, || {
//!     div()
//!         .bg_cn(ShadcnToken::Background)
//!         .text_cn(ShadcnToken::Foreground)
//! });
//! ```

pub use crate::context::with_look;
pub use crate::ext::ShadcnElementExt;
pub use crate::look::ShadcnLook;
pub use crate::tokens::{ShadcnFont, ShadcnRadius, ShadcnShadow, ShadcnStyle, ShadcnTextSize, ShadcnToken};
