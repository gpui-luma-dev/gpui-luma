//! Table selection and row identity.
//!
//! `Single` (the default) selects one row. `Multiple` independently toggles rows
//! on click or Space/Enter. `Extended` replaces selection on plain click or
//! Space/Enter, toggles with Ctrl/Cmd, and extends from a fixed anchor with Shift.
//! Ctrl/Cmd+Shift adds a range. `None` retains navigation without selection.
//! Arrows and Home/End move the active row without changing selection;
//! Shift+arrows/Home/End selects ranges in Extended mode. Ctrl/Cmd+A selects all
//! enabled rows across pages; adding Shift clears selection in both multi modes.
//! Ranges follow current row order, skip disabled rows, and can cross pages.
//!
//! Controls own their selection by default. Owners can replace it with
//! [`TableControl::set_selected_indices`] or [`TableControl::set_selected_keys`]
//! without moving focus, the active row, or scroll position. These setters reset
//! the range anchor only for different membership and emit changes once. This permits
//! subscribing to events and feeding owner state back without an event loop.
//!
//! After spawning, configure [`TableControl::set_row_key`] before replacing or
//! sorting rows. Keys must be unique and stable for a record. [`TableControl::set_items`]
//! returns an error for duplicate keys; successful replacements retain selected,
//! active and anchor identities, dropping removed/disabled selections. Unkeyed
//! tables retain positional selection and cannot preserve identity across sorting.
//! Selection indices are returned in row order. The legacy index event can fire
//! on a reorder; [`TableEvent::SelectedKeysChanged`] fires only for different key
//! membership. Mode changes and row replacement also emit reconciliation events.
//!
//! [`TableSelectionMode::Extended`] and the fallible `set_items` return value are
//! additions to the original API; exhaustive mode matches and row replacement
//! callers should handle them.

mod control;
mod column_template;
mod layout;
mod macros;
mod model;
mod paging;
mod row;
mod template;
mod theme;

pub use control::{TableControl, TableEvent, TableSelectionError};
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
