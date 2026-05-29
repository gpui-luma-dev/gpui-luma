mod control;
mod macros;
mod model;
mod row_chrome;
mod template;
mod theme;

pub use control::{ListViewControl, ListViewEvent};
pub use model::{
    ListSelectionMode, ListViewAppearanceOverride, ListViewBuilder, ListViewColumn, ListViewColumnWidth,
    ListViewEnabledFn, ListViewHeaderTemplate, ListViewGridColumnsBuilder, ListViewLabel, ListViewLabelFn,
    ListViewRenderModel, ListViewRowRenderModel, ListViewRowTemplate, make_list_view_header_template,
    make_list_view_row_template,
};
pub use template::{
    DefaultListViewShellTemplate, ListViewTemplate, ListViewTemplateModifier, default_list_view_template,
    list_view_template_with_modifier, list_view_template_with_theme,
};
pub use theme::{DefaultListViewTheme, ListViewAppearance, ListViewRowAppearance, ListViewTheme, default_list_view_theme};

use gpui::{Entity, SharedString};

pub type ListView<T> = Entity<ListViewControl<T>>;

pub fn new(id: impl Into<SharedString>) -> ListViewBuilder<ListViewLabel> {
    ListViewBuilder::new(id)
}

pub fn new_typed<T>(id: impl Into<SharedString>) -> ListViewBuilder<T>
where
    T: 'static,
{
    ListViewBuilder::new_typed(id)
}
