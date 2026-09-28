//! Primary consumer facade for GPUI-Luma.
//!
//! Re-exports the control SDK (`gpui-luma-core`), including shared look provenance.
//! Pick **exactly one** look crate in your app:
//! `gpui-luma-look-shadcn` or `gpui-luma-look-radix`.

pub use sdk::theme::provenance::*;
pub use sdk::*;
