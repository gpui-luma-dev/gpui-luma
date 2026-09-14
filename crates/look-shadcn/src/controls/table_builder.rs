//! Look-owned table builder. Spawn synthesizes the SDK [`luma::controls::table::Table`].

use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, ListAlignment, SharedString, Window};
use luma::controls::table::{
    TableSelectionMode, TableBuilder, TableColumn, TableControl, TableRowRenderModel, TableTemplate, TableTheme,
};
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a table: Shadcn theme/template plus SDK options, until `.spawn(cx)`.
pub struct Table<T>
where
    T: 'static,
{
    look: Option<ShadcnLook>,
    builder: TableBuilder<T>,
    custom_template: bool,
    custom_theme: bool,
    size: ShadcnSize,
}

impl<T> Table<T>
where
    T: 'static,
{
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            look: None,
            builder: luma::controls::table::new_typed(id),
            custom_template: false,
            custom_theme: false,
            size: ShadcnSize::Md,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn item(mut self, item: T) -> Self {
        self.builder = self.builder.item(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    pub fn selection_mode(mut self, selection_mode: TableSelectionMode) -> Self {
        self.builder = self.builder.selection_mode(selection_mode);
        self
    }

    pub fn single(mut self) -> Self {
        self.builder = self.builder.single();
        self
    }

    pub fn multiple(mut self) -> Self {
        self.builder = self.builder.multiple();
        self
    }

    pub fn no_selection(mut self) -> Self {
        self.builder = self.builder.no_selection();
        self
    }

    pub fn select_on_row_click(mut self, select_on_row_click: bool) -> Self {
        self.builder = self.builder.select_on_row_click(select_on_row_click);
        self
    }

    pub fn selected_index(mut self, selected_index: usize) -> Self {
        self.builder = self.builder.selected_index(selected_index);
        self
    }

    pub fn selected_indices(mut self, selected_indices: impl IntoIterator<Item = usize>) -> Self {
        self.builder = self.builder.selected_indices(selected_indices);
        self
    }

    pub fn active_index(mut self, active_index: usize) -> Self {
        self.builder = self.builder.active_index(active_index);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn alignment(mut self, alignment: ListAlignment) -> Self {
        self.builder = self.builder.alignment(alignment);
        self
    }

    pub fn overdraw(mut self, overdraw: impl Into<f64>) -> Self {
        self.builder = self.builder.overdraw(overdraw);
        self
    }

    pub fn paged(mut self, page_size: usize) -> Self {
        self.builder = self.builder.paged(page_size);
        self
    }

    pub fn scroll_snap(mut self, enabled: bool) -> Self {
        self.builder = self.builder.scroll_snap(enabled);
        self
    }

    pub fn visible_rows(mut self, count: usize) -> Self {
        self.builder = self.builder.visible_rows(count);
        self
    }

    pub fn fill_height(mut self) -> Self {
        self.builder = self.builder.fill_height();
        self
    }

    pub fn visible_row_height(mut self, height: impl Into<f32>) -> Self {
        self.builder = self.builder.visible_row_height(height);
        self
    }

    pub fn row_label<F, S>(mut self, label: F) -> Self
    where
        F: Fn(&T) -> S + Send + Sync + 'static,
        S: Into<SharedString>,
    {
        self.builder = self.builder.row_label(label);
        self
    }

    pub fn row_enabled<F>(mut self, enabled: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        self.builder = self.builder.row_enabled(enabled);
        self
    }

    pub fn template(mut self, template: Arc<dyn TableTemplate>) -> Self {
        self.custom_template = true;
        self.builder = self.builder.template(template);
        self
    }

    pub fn theme(mut self, theme: Arc<dyn TableTheme>) -> Self {
        self.custom_theme = true;
        self.builder = self.builder.theme(theme);
        self
    }

    pub fn with_row_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&TableRowRenderModel<'a, T>, AnyElement, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.builder = self.builder.with_row_template(template);
        self
    }

    pub fn grid_view(mut self, columns: impl IntoIterator<Item = TableColumn<T>>) -> Self {
        self.builder = self.builder.grid_view(columns);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<TableControl<T>> {
        let look = resolve_look_from(self.look.as_ref(), cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> TableBuilder<T> {
        let mut builder = self.builder.size(self.size.control_size());
        if !self.custom_theme {
            builder = builder.theme(look.table_theme());
        }
        if !self.custom_template {
            builder = builder.template(look.table_template());
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Table::new("ok")
            .look(&look)
            .items([SharedString::from("one")])
            .row_label(|row| row.clone())
            .into_sdk_builder(look);
    }
}
