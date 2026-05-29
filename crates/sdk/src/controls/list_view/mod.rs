mod control;
mod macros;
mod model;
mod row_template;
mod template;
mod theme;

pub use control::{ListViewControl, ListViewEvent};
pub use model::{
    ListSelectionMode, ListViewBuilder, ListViewColumn, ListViewColumnWidth, ListViewEnabledFn, ListViewHeaderTemplate,
    ListViewGridColumnsBuilder, ListViewItem, ListViewItemRenderModel, ListViewItemTemplate, ListViewLabelFn,
    ListViewListAppearanceOverride, ListViewRenderModel, make_list_view_header_template, make_list_view_item_template,
};
pub use template::{
    DefaultListViewShellTemplate, ListViewTemplate, ListViewTemplateModifier, default_list_view_template,
    list_view_template_with_modifier, list_view_template_with_theme,
};
pub use theme::{
    DefaultListViewTheme, ListViewListAppearance, ListViewRowAppearance, ListViewTheme, default_list_view_theme,
};

use gpui::{Entity, SharedString};

pub type ListView<T> = Entity<ListViewControl<T>>;

pub fn new(id: impl Into<SharedString>) -> ListViewBuilder<ListViewItem> {
    ListViewBuilder::new(id)
}

pub fn new_typed<T>(id: impl Into<SharedString>) -> ListViewBuilder<T>
where
    T: 'static,
{
    ListViewBuilder::new_typed(id)
}
