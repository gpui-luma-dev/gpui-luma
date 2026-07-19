mod control;
mod model;
mod template;
mod theme;

pub use control::{Toolbar, ToolbarEvent, ToolbarValue, horizontal_arrow_policy};
pub use model::{
    ToolbarBuilder, ToolbarChangeHandler, ToolbarClickHandler, ToolbarItem, ToolbarItemKind, ToolbarItemRenderModel,
    ToolbarItemSource, ToolbarRenderModel,
};
pub use template::{
    ThemedToolbarTemplate, ToolbarRenderedItem, ToolbarTemplate, ToolbarTemplateModifier, default_toolbar_template,
    toolbar_template_with_theme,
};
pub use theme::{DefaultToolbarTheme, ToolbarLook, ToolbarTheme, ToolbarVariant, default_toolbar_theme};

use gpui::{Entity, SharedString};

pub type ToolbarControl = Entity<Toolbar>;

pub fn new(id: impl Into<SharedString>) -> ToolbarBuilder {
    ToolbarBuilder::new(id)
}
