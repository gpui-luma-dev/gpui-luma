//! Controls for moving between destinations or sections.
//! Hierarchical record views belong to [`super::collection`].

pub mod accordion;
pub mod pager;
pub mod sidebar;
pub mod stepper;
pub mod tabs;
/// Compatibility export; Tree View now lives in [`super::collection`].
pub use super::collection::tree_view;
