use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Entity, ListAlignment, SharedString, Window};

use super::control::ListViewControl;
use super::template::{ListViewTemplate, default_list_view_template};
use super::theme::{ListViewTheme, default_list_view_theme};
use crate::controls::state::ControlFocusState;
use crate::theme::ControlSize;

pub trait ListViewItemLike {
    fn label(&self) -> &SharedString;

    fn is_enabled(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub struct ListViewItem {
    pub(crate) label: SharedString,
    pub(crate) enabled: bool,
}

impl ListViewItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into(), enabled: true }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl ListViewItemLike for ListViewItem {
    fn label(&self) -> &SharedString {
        &self.label
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ListSelectionMode {
    None,
    #[default]
    Single,
    Multiple,
}

pub struct ListViewRenderModel<'a> {
    pub id: &'a SharedString,
    pub item_count: usize,
    pub selection_mode: ListSelectionMode,
    pub enabled: bool,
    pub size: ControlSize,
    pub focus: ControlFocusState,
}

pub type ListViewHeaderTemplate =
    Arc<dyn for<'a> Fn(&ListViewRenderModel<'a>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static>;

pub fn make_list_view_header_template<F, E>(template: F) -> ListViewHeaderTemplate
where
    F: for<'a> Fn(&ListViewRenderModel<'a>, &mut Window, &mut App) -> E + Send + Sync + 'static,
    E: gpui::IntoElement + 'static,
{
    Arc::new(move |model, window, cx| template(model, window, cx).into_any_element())
}

#[derive(Clone, Debug)]
pub struct ListViewItemRenderModel<'a, T>
where
    T: ListViewItemLike + 'static,
{
    pub list_id: &'a SharedString,
    pub item: &'a T,
    pub index: usize,
    pub sibling_count: usize,
    pub selected: bool,
    pub active: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub enabled: bool,
}

pub type ListViewItemTemplate<T> =
    Arc<dyn for<'a> Fn(&ListViewItemRenderModel<'a, T>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static>;

pub fn make_list_view_item_template<T, F, E>(template: F) -> ListViewItemTemplate<T>
where
    T: ListViewItemLike + 'static,
    F: for<'a> Fn(&ListViewItemRenderModel<'a, T>, &mut Window, &mut App) -> E + Send + Sync + 'static,
    E: gpui::IntoElement + 'static,
{
    Arc::new(move |model, window, cx| template(model, window, cx).into_any_element())
}

#[derive(Clone)]
pub struct ListViewModel<T>
where
    T: ListViewItemLike + 'static,
{
    pub(crate) id: SharedString,
    pub(crate) items: Vec<T>,
    pub(crate) selected_indices: Vec<usize>,
    pub(crate) active_index: Option<usize>,
    pub(crate) selection_mode: ListSelectionMode,
    pub(crate) enabled: bool,
    pub(crate) size: ControlSize,
    pub(crate) alignment: ListAlignment,
    pub(crate) overdraw: f32,
    pub(crate) template: Arc<dyn ListViewTemplate>,
    pub(crate) header_template: Option<ListViewHeaderTemplate>,
    pub(crate) item_template: Option<ListViewItemTemplate<T>>,
    pub(crate) theme: Arc<dyn ListViewTheme>,
}

pub struct ListViewBuilder<T>
where
    T: ListViewItemLike + 'static,
{
    pub(crate) model: ListViewModel<T>,
}

impl<T> ListViewBuilder<T>
where
    T: ListViewItemLike + 'static,
{
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: ListViewModel {
                id: id.into(),
                items: Vec::new(),
                selected_indices: Vec::new(),
                active_index: None,
                selection_mode: ListSelectionMode::Single,
                enabled: true,
                size: ControlSize::Md,
                alignment: ListAlignment::Top,
                overdraw: 480.0,
                template: default_list_view_template(),
                header_template: None,
                item_template: None,
                theme: default_list_view_theme(),
            },
        }
    }

    pub fn item(mut self, item: T) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn selection_mode(mut self, selection_mode: ListSelectionMode) -> Self {
        self.model.selection_mode = selection_mode;
        self
    }

    pub fn single(mut self) -> Self {
        self.model.selection_mode = ListSelectionMode::Single;
        self
    }

    pub fn multiple(mut self) -> Self {
        self.model.selection_mode = ListSelectionMode::Multiple;
        self
    }

    pub fn no_selection(mut self) -> Self {
        self.model.selection_mode = ListSelectionMode::None;
        self
    }

    pub fn selected_index(mut self, selected_index: usize) -> Self {
        self.model.selected_indices = vec![selected_index];
        self
    }

    pub fn selected_indices(mut self, selected_indices: impl IntoIterator<Item = usize>) -> Self {
        self.model.selected_indices = selected_indices.into_iter().collect();
        self
    }

    pub fn active_index(mut self, active_index: usize) -> Self {
        self.model.active_index = Some(active_index);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn alignment(mut self, alignment: ListAlignment) -> Self {
        self.model.alignment = alignment;
        self
    }

    pub fn overdraw(mut self, overdraw: impl Into<f64>) -> Self {
        self.model.overdraw = overdraw.into().max(0.0) as f32;
        self
    }

    pub fn template(mut self, template: Arc<dyn ListViewTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn theme(mut self, theme: Arc<dyn ListViewTheme>) -> Self {
        self.model.theme = theme;
        self
    }

    pub fn header_template(mut self, header_template: ListViewHeaderTemplate) -> Self {
        self.model.header_template = Some(header_template);
        self
    }

    pub fn with_header_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&ListViewRenderModel<'a>, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.model.header_template = Some(make_list_view_header_template(template));
        self
    }

    pub fn item_template(mut self, item_template: ListViewItemTemplate<T>) -> Self {
        self.model.item_template = Some(item_template);
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&ListViewItemRenderModel<'a, T>, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.model.item_template = Some(make_list_view_item_template(template));
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ListViewControl<T>> {
        cx.new(|cx| ListViewControl::from_builder(self, cx))
    }
}
