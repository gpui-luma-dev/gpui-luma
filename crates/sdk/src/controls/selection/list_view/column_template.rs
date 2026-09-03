use std::sync::Arc;

use gpui::{AnyElement, App, FontWeight, SharedString, Window, div, prelude::*};

use super::model::{ListViewColumnCellTemplate, ListViewRowRenderModel};

pub type ListViewColumnRenderModel<'a, T> = ListViewRowRenderModel<'a, T>;

pub fn default_text_column_template<T, F, S>(value_fn: F) -> ListViewColumnCellTemplate<T>
where
    T: 'static,
    F: Fn(&T) -> S + Send + Sync + 'static,
    S: Into<SharedString> + 'static,
{
    Arc::new(move |model, _window, _cx| {
        let value = value_fn(model.row);
        div().truncate().child(value.into()).into_any_element()
    })
}

pub fn default_muted_column_template<T, F, S>(value_fn: F) -> ListViewColumnCellTemplate<T>
where
    T: 'static,
    F: Fn(&T) -> S + Send + Sync + 'static,
    S: Into<SharedString> + 'static,
{
    Arc::new(move |model, _window, _cx| {
        let value = value_fn(model.row);
        div().opacity(0.65).truncate().child(value.into()).into_any_element()
    })
}

pub fn default_emphasis_column_template<T, F, S>(value_fn: F) -> ListViewColumnCellTemplate<T>
where
    T: 'static,
    F: Fn(&T) -> S + Send + Sync + 'static,
    S: Into<SharedString> + 'static,
{
    Arc::new(move |model, _window, _cx| {
        let value = value_fn(model.row);
        div().font_weight(FontWeight::SEMIBOLD).truncate().child(value.into()).into_any_element()
    })
}

pub fn default_numeric_column_template<T, F, S>(value_fn: F) -> ListViewColumnCellTemplate<T>
where
    T: 'static,
    F: Fn(&T) -> S + Send + Sync + 'static,
    S: Into<SharedString> + 'static,
{
    Arc::new(move |model, _window, _cx| {
        let value = value_fn(model.row);
        div().w_full().flex().justify_end().truncate().child(value.into()).into_any_element()
    })
}

pub fn column_template_with_modifier<T, F>(
    base: ListViewColumnCellTemplate<T>,
    modifier: F,
) -> ListViewColumnCellTemplate<T>
where
    T: 'static,
    F: Fn(AnyElement, &ListViewColumnRenderModel<'_, T>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
{
    Arc::new(move |model, window, cx| {
        let cell = base(model, window, cx);
        modifier(cell, model, window, cx)
    })
}
