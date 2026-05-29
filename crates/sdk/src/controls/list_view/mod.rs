mod control;
mod model;
mod template;
mod theme;

pub use control::{ListViewControl, ListViewEvent};
pub use model::{
    ListSelectionMode, ListViewBuilder, ListViewHeaderTemplate, ListViewItem, ListViewItemLike,
    ListViewItemRenderModel, ListViewItemTemplate, ListViewRenderModel, make_list_view_header_template,
    make_list_view_item_template,
};
pub use template::{ListViewTemplate, ThemedListViewTemplate, default_list_view_template, list_view_template_with_theme};
pub use theme::{
    DefaultListViewTheme, ListViewListAppearance, ListViewRowAppearance, ListViewTheme, default_list_view_theme,
};

use gpui::{Entity, SharedString};

pub type ListView<T> = Entity<ListViewControl<T>>;

pub fn new<T>(id: impl Into<SharedString>) -> ListViewBuilder<T>
where
    T: ListViewItemLike + 'static,
{
    ListViewBuilder::new(id)
}
