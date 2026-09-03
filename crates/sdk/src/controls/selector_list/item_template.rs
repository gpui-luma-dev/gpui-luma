use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString};

pub struct SelectorItemRenderModel<'a, T> {
    pub selector_id: &'a SharedString,
    pub item: &'a T,
    pub index: usize,
    pub selected: bool,
    pub active: bool,
    pub open: bool,
    pub enabled: bool,
}

pub type SelectorItemTemplate<T> =
    Arc<dyn for<'a> Fn(&SelectorItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub fn make_selector_item_template<T, F, E>(template: F) -> SelectorItemTemplate<T>
where
    F: for<'a> Fn(&SelectorItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, cx| template(model, cx).into_any_element())
}
