mod control;
/// Internal flush-list presentation engine (former `navigation_sidebar`).
mod engine;
mod model;
mod template;
mod theme;

pub use control::{SidebarControl, SidebarEvent};
pub use model::{
    SidebarBuilder, SidebarContentBuilder, SidebarControlBuilder, SidebarControlModel, SidebarFooterBuilder,
    SidebarGroupBuilder, SidebarHeaderBuilder, SidebarInsetBuilder, SidebarInsetModel, SidebarMenuBuilder,
    SidebarMenuItemBuilder, SidebarMenuItemModel, SidebarMenuModel, SidebarMenuSubBuilder, SidebarPaneRender,
    SidebarPanelModel, SidebarRailBuilder, pane_render,
};
pub use template::{
    DefaultSidebarTemplate, SidebarRenderModel, SidebarTemplate, default_sidebar_template, render_inset_column,
};
pub use theme::{SidebarCollapsible, SidebarMetricScale, SidebarVariant};

// Look-bound flush theme / template surface (presentation engine).
pub use engine::{
    DefaultSidebarTheme, SidebarContainerLook, SidebarItemLook, SidebarSectionLook, SidebarPanelTemplate,
    SidebarPanelTemplateHandlers, SidebarPanelTemplateModifier, SidebarTheme, ThemedSidebarPanelTemplate,
    default_sidebar_panel_template, default_sidebar_theme,
};

use gpui::{Entity, SharedString};

pub type SidebarControlEntity = Entity<SidebarControl>;

pub fn new(id: impl Into<SharedString>) -> SidebarControlBuilder {
    SidebarControlBuilder::new(id)
}

pub fn sidebar(id: impl Into<SharedString>) -> SidebarBuilder {
    SidebarBuilder::new(id)
}

pub fn sidebar_header() -> SidebarHeaderBuilder {
    SidebarHeaderBuilder::new()
}

pub fn sidebar_content() -> SidebarContentBuilder {
    SidebarContentBuilder::new()
}

pub fn sidebar_group() -> SidebarGroupBuilder {
    SidebarGroupBuilder::new()
}

pub fn sidebar_menu(id: impl Into<SharedString>) -> SidebarMenuBuilder {
    SidebarMenuBuilder::new(id)
}

pub fn sidebar_menu_item(id: impl Into<SharedString>, label: impl Into<SharedString>) -> SidebarMenuItemBuilder {
    SidebarMenuItemBuilder::new(id, label)
}

pub fn sidebar_menu_sub() -> SidebarMenuSubBuilder {
    SidebarMenuSubBuilder::new()
}

pub fn sidebar_footer() -> SidebarFooterBuilder {
    SidebarFooterBuilder::new()
}

pub fn sidebar_rail() -> SidebarRailBuilder {
    SidebarRailBuilder::new()
}

pub fn sidebar_inset() -> SidebarInsetBuilder {
    SidebarInsetBuilder::new()
}
