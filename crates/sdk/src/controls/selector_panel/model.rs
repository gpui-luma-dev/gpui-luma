use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString};

use crate::controls::icon::IconSource;

pub trait SelectorItemLike {
    fn id(&self) -> &SharedString;

    fn is_enabled(&self) -> bool {
        true
    }

    fn is_selected(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub struct SelectorItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) icon: Option<IconSource>,
    pub(crate) enabled: bool,
    pub(crate) selected: bool,
}

impl SelectorItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self { label: id.clone(), id, icon: None, enabled: true, selected: false }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<IconSource>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }

    pub fn icon_ref(&self) -> Option<&IconSource> {
        self.icon.as_ref()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }
}

impl SelectorItemLike for SelectorItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

pub fn normalize_selector_items<T: SelectorItemLike>(items: impl IntoIterator<Item = T>) -> Vec<T> {
    items.into_iter().filter(SelectorItemLike::is_enabled).collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorPath {
    Item(usize),
}

impl SelectorPath {
    pub fn is_item(self, index: usize) -> bool {
        matches!(self, Self::Item(active) if active == index)
    }
}

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
    T: SelectorItemLike + 'static,
    F: for<'a> Fn(&SelectorItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, cx| template(model, cx).into_any_element())
}
