//! Convenient imports for shadcn look styling and control construction.
//!
//! ```ignore
//! use gpui_luma_look_shadcn::prelude::*;
//!
//! let submit = look.primary_button("submit").label("Submit").spawn(cx);
//! with_look(&look, || {
//!     div()
//!         .bg_cn(ShadcnToken::Background)
//!         .text_cn(ShadcnToken::Foreground)
//! });
//! ```

pub use crate::context::with_look;
pub use crate::controls::{
    ShadcnButtonStyle, ShadcnButtonStyleExt, ShadcnCheckboxStyleExt, ShadcnLookControlExt, ShadcnSwitchStyleExt,
    ShadcnTextAreaExt, ShadcnTextFieldExt,
};
pub use crate::elements::{Badge, BadgeIconPlacement, BadgeVariant};
pub use crate::ext::{LumaTypographyExt, ShadcnElementExt};
pub use crate::look::ShadcnLook;
pub use crate::tokens::{ShadcnFont, ShadcnRadius, ShadcnShadow, ShadcnStyle, ShadcnTextRole, ShadcnTextSize, ShadcnToken};
