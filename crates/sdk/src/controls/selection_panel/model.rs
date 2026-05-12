use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString};

use crate::controls::icon::IconSource;

pub trait SelectionPanelItemLike {
    fn id(&self) -> &SharedString;

    fn label(&self) -> &SharedString {
        self.id()
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn icon(&self) -> Option<&IconSource> {
        None
    }

    fn is_selected(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub struct SelectionPanelItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) icon: Option<IconSource>,
    pub(crate) enabled: bool,
    pub(crate) selected: bool,
}

impl SelectionPanelItem {
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

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }
}

impl SelectionPanelItemLike for SelectionPanelItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn label(&self) -> &SharedString {
        &self.label
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn icon(&self) -> Option<&IconSource> {
        self.icon.as_ref()
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionPanelPath {
    Item(usize),
}

impl SelectionPanelPath {
    pub fn is_item(self, index: usize) -> bool {
        matches!(self, Self::Item(active) if active == index)
    }
}

#[derive(Clone, Debug)]
pub struct SelectionPanelPresenterModel<'a, T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub panel_id: &'a SharedString,
    pub control_id: &'a SharedString,
    pub item: &'a T,
    pub source_index: usize,
    pub visible_index: usize,
    pub selected: bool,
    pub active: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub enabled: bool,
    pub sibling_count: usize,
}

pub type SelectionPanelPresenter<T> =
    Arc<dyn for<'a> Fn(&SelectionPanelPresenterModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub fn make_selection_panel_presenter<T, F, E>(presenter: F) -> SelectionPanelPresenter<T>
where
    T: SelectionPanelItemLike + 'static,
    F: for<'a> Fn(&SelectionPanelPresenterModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, cx| presenter(model, cx).into_any_element())
}
