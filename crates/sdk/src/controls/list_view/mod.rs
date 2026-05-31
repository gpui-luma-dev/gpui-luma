mod control;
mod column_template;
mod layout;
mod macros;
mod model;
mod row;
mod template;
mod theme;

pub use control::{ListViewControl, ListViewEvent};
pub use column_template::{
    ListViewColumnRenderModel, column_template_with_modifier, default_emphasis_column_template,
    default_muted_column_template, default_numeric_column_template, default_text_column_template,
};
pub use layout::{
    ROW_DIVIDER_WIDTH, body_rows_height, compute_shell_height, default_row_height, effective_visible_rows, page_count,
    visible_row_height,
};
pub use model::{
    IntoListViewColumnCellTemplate, ListScrollMode, ListSelectionMode, ListViewAppearanceOverride, ListViewBuilder,
    ListViewColumn, ListViewColumnCellTemplate, ListViewColumnWidth, ListViewEnabledFn, ListViewHeaderTemplate,
    ListViewGridColumnsBuilder, ListViewLabel, ListViewLabelFn, ListViewRenderModel, ListViewRowRenderModel,
    ListViewRowTemplate, make_list_view_header_template, make_list_view_row_template,
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
