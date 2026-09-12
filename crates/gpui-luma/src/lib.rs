//! Primary consumer facade for GPUI-Luma.
//!
//! Re-exports the control SDK (`gpui-luma-core`) and look-agnostic contracts
//! (`gpui-luma-look-core`). Pick **exactly one** look crate in your app:
//! `gpui-luma-look-shadcn` or `gpui-luma-look-radix`.

pub use luma_look_core::*;
pub use sdk::*;
