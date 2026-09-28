//! Flush presentation engine formerly known as `navigation_sidebar`.
//!
//! Kept as an internal implementation detail of [`super::SidebarControl`].
//! Theme/template types are re-exported from [`super`] for look-crate binding.

#![allow(dead_code)]
#![allow(unused_imports)]

mod control;
mod model;
mod template;
mod theme;

pub(crate) use control::{SidebarPanelEngine, SidebarPanelEngineEvent};
pub(crate) use model::{
    NavHostedContent, NavNode, NavNodeKind, NavNodeState, NavPresenter, SidebarPanelEngineBuilder,
    SidebarPanelEngineModel, SidebarPanelEngineRenderModel, RenderedNavNode, RenderedRailSubmenu, entity_presenter,
};
pub use template::{
    SidebarPanelTemplate, SidebarPanelTemplateHandlers, SidebarPanelTemplateModifier, ThemedSidebarPanelTemplate,
    default_sidebar_panel_template,
};
pub(crate) use template::modified_sidebar_panel_template;
pub use theme::{DefaultSidebarTheme, SidebarItemLook, SidebarSectionLook, SidebarTheme, default_sidebar_theme};
