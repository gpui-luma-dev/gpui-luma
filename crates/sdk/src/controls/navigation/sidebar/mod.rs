mod control;
/// Internal flush-list presentation engine (former `navigation_sidebar`).
mod engine;
mod model;
mod theme;

pub use control::{SidebarControl, SidebarEvent};
pub use model::{
    SidebarBuilder, SidebarContentBuilder, SidebarControlBuilder, SidebarControlModel, SidebarFooterBuilder,
    SidebarGroupBuilder, SidebarHeaderBuilder, SidebarMenuBuilder, SidebarMenuItemBuilder, SidebarMenuItemModel,
    SidebarMenuModel, SidebarMenuSubBuilder, SidebarPanelModel,
};
pub use theme::{SidebarMetricScale, SidebarPresentation};

// Look-bound flush theme / template surface (presentation engine).
pub use engine::{
    DefaultSidebarTheme, SidebarItemLook, SidebarSectionLook, SidebarPanelTemplate, SidebarPanelTemplateHandlers,
    SidebarPanelTemplateModifier, SidebarTheme, ThemedSidebarPanelTemplate, default_sidebar_panel_template,
    default_sidebar_theme,
};

use gpui::SharedString;

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
