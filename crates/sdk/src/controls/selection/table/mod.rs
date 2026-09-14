mod control;
mod column_template;
mod layout;
mod macros;
mod model;
mod paging;
mod row;
mod template;
mod theme;

pub use control::{TableControl, TableEvent};
pub use paging::{PagingTable, PagingTableControl, PagingTableBuilder};
pub use column_template::{
    TableColumnRenderModel, column_template_with_modifier, default_emphasis_column_template,
    default_muted_column_template, default_numeric_column_template, default_text_column_template,
};
pub use layout::{
    ROW_DIVIDER_WIDTH, body_rows_height, compute_shell_height, default_row_height, distributed_row_height,
    effective_visible_rows, page_after_size_change, page_count, rows_for_viewport_height,
    rows_for_viewport_height_border_box, visible_row_height,
};
pub use model::{
    IntoTableColumnCellTemplate, TableScrollMode, TableSelectionMode, TableLookOverride, TableBuilder, TableColumn,
    TableColumnCellLayout, TableColumnCellTemplate, TableColumnWidth, TableEnabledFn, TableHeaderTemplate,
    TableGridColumnsBuilder, TableLabel, TableLabelFn, TableRenderModel, TableRowRenderModel, TableRowTemplate,
    make_table_header_template, make_table_row_template,
};
pub use template::{
    DefaultTableShellTemplate, TableTemplate, TableTemplateModifier, default_table_template, table_template_with_theme,
};
pub use theme::{DefaultTableTheme, TableLook, TableRowLook, TableRowPalette, TableTheme, default_table_theme};

use gpui::{Entity, SharedString};

pub type Table<T> = Entity<TableControl<T>>;

pub fn new(id: impl Into<SharedString>) -> TableBuilder<TableLabel> {
    TableBuilder::new(id)
}

pub fn new_typed<T>(id: impl Into<SharedString>) -> TableBuilder<T>
where
    T: 'static,
{
    TableBuilder::new_typed(id)
}
