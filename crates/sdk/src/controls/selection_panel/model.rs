use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString};

use crate::controls::icon::IconSource;
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};

use crate::controls::selection_panel::template::{SelectionPanelTemplate, default_selection_panel_template};
use crate::controls::selection_panel::theme::{SelectionPanelAppearance, default_selection_panel_appearance};
use crate::theme::{ControlSize, ThemeTokens};

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
pub struct SelectionPanelItemRenderModel<'a, T>
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

pub type SelectionPanelItemTemplate<T> =
    Arc<dyn for<'a> Fn(&SelectionPanelItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub fn make_selection_panel_item_template<T, F, E>(template: F) -> SelectionPanelItemTemplate<T>
where
    T: SelectionPanelItemLike + 'static,
    F: for<'a> Fn(&SelectionPanelItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, cx| template(model, cx).into_any_element())
}

pub type SelectionPanelAppearanceProvider =
    Arc<dyn Fn(ControlSize) -> SelectionPanelAppearance + Send + Sync + 'static>;

#[derive(Clone)]
pub struct SelectionPanelModel<T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub(crate) id: SharedString,
    pub(crate) panel_id: SharedString,
    pub(crate) items: Vec<T>,
    pub(crate) visible_indices: Vec<usize>,
    pub(crate) selected_source_index: Option<usize>,
    pub(crate) open: bool,
    pub(crate) enabled: bool,
    pub(crate) show_selection_marker: bool,
    pub(crate) scrolling: bool,
    pub(crate) min_visible_rows: usize,
    pub(crate) max_visible_rows: usize,
    pub(crate) size: ControlSize,
    pub(crate) item_template: Option<SelectionPanelItemTemplate<T>>,
    pub(crate) template: Arc<dyn SelectionPanelTemplate<T>>,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) appearance_provider: SelectionPanelAppearanceProvider,
}

pub(crate) fn default_selection_panel_model<T>(id: impl Into<SharedString>) -> SelectionPanelModel<T>
where
    T: SelectionPanelItemLike + 'static,
{
    let id = id.into();

    SelectionPanelModel {
        panel_id: format!("{}-panel", id).into(),
        id,
        items: Vec::new(),
        visible_indices: Vec::new(),
        selected_source_index: None,
        open: true,
        enabled: true,
        show_selection_marker: true,
        scrolling: true,
        min_visible_rows: 1,
        max_visible_rows: 7,
        size: ControlSize::Md,
        item_template: None,
        template: default_selection_panel_template(),
        scrollbar_template: default_scrollbar_template(),
        appearance_provider: Arc::new(|size| default_selection_panel_appearance(&ThemeTokens::default(), size)),
    }
}
