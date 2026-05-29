use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Div, Entity, ListAlignment, SharedString, Stateful, Window, div, prelude::*, px};

use super::control::ListViewControl;
use super::template::{ListViewTemplate, default_list_view_template, list_view_template_with_modifier};
use super::theme::{ListViewListAppearance, ListViewTheme, default_list_view_theme};
use crate::controls::state::ControlFocusState;
use crate::theme::ControlSize;

pub type ListViewLabelFn<T> = Arc<dyn Fn(&T) -> SharedString + Send + Sync + 'static>;
pub type ListViewEnabledFn<T> = Arc<dyn Fn(&T) -> bool + Send + Sync + 'static>;

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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ListSelectionMode {
    None,
    #[default]
    Single,
    Multiple,
}

pub struct ListViewRenderModel<'a> {
    pub id: &'a SharedString,
    pub list: ListViewListAppearance,
    pub item_count: usize,
    pub selection_mode: ListSelectionMode,
    pub enabled: bool,
    pub size: ControlSize,
    pub focus: ControlFocusState,
}

pub type ListViewListAppearanceOverride =
    Arc<dyn Fn(ListViewListAppearance) -> ListViewListAppearance + Send + Sync + 'static>;

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
    T: 'static,
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
    T: 'static,
    F: for<'a> Fn(&ListViewItemRenderModel<'a, T>, &mut Window, &mut App) -> E + Send + Sync + 'static,
    E: gpui::IntoElement + 'static,
{
    Arc::new(move |model, window, cx| template(model, window, cx).into_any_element())
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ListViewColumnWidth {
    Fixed(f32),
    #[default]
    Fill,
}

pub type ListViewColumnCellTemplate<T> = Arc<dyn Fn(&T) -> AnyElement + Send + Sync + 'static>;

#[derive(Clone)]
pub struct ListViewColumn<T>
where
    T: 'static,
{
    header: SharedString,
    width: ListViewColumnWidth,
    cell_template: ListViewColumnCellTemplate<T>,
}

impl<T> ListViewColumn<T>
where
    T: 'static,
{
    pub fn new(
        header: impl Into<SharedString>,
        width: ListViewColumnWidth,
        cell_template: ListViewColumnCellTemplate<T>,
    ) -> Self {
        Self { header: header.into(), width, cell_template }
    }

    pub fn fixed<F, E>(header: impl Into<SharedString>, width: f32, cell_template: F) -> Self
    where
        F: Fn(&T) -> E + Send + Sync + 'static,
        E: gpui::IntoElement + 'static,
    {
        Self {
            header: header.into(),
            width: ListViewColumnWidth::Fixed(width),
            cell_template: Arc::new(move |item| cell_template(item).into_any_element()),
        }
    }

    pub fn fill<F, E>(header: impl Into<SharedString>, cell_template: F) -> Self
    where
        F: Fn(&T) -> E + Send + Sync + 'static,
        E: gpui::IntoElement + 'static,
    {
        Self {
            header: header.into(),
            width: ListViewColumnWidth::Fill,
            cell_template: Arc::new(move |item| cell_template(item).into_any_element()),
        }
    }

    pub fn header(&self) -> &SharedString {
        &self.header
    }

    pub fn width(&self) -> ListViewColumnWidth {
        self.width
    }

    pub fn render_cell(&self, item: &T) -> AnyElement {
        (self.cell_template)(item)
    }
}

#[derive(Clone)]
pub struct ListViewModel<T>
where
    T: 'static,
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
    pub(crate) item_label: ListViewLabelFn<T>,
    pub(crate) item_enabled: ListViewEnabledFn<T>,
    pub(crate) template: Arc<dyn ListViewTemplate>,
    pub(crate) header_template: Option<ListViewHeaderTemplate>,
    pub(crate) item_template: Option<ListViewItemTemplate<T>>,
    pub(crate) theme: Arc<dyn ListViewTheme>,
    pub(crate) list_appearance_override: Option<ListViewListAppearanceOverride>,
}

impl<T> ListViewModel<T>
where
    T: 'static,
{
    pub(crate) fn label_for_item(&self, item: &T) -> SharedString {
        (self.item_label)(item)
    }

    pub(crate) fn item_is_enabled(&self, item: &T) -> bool {
        (self.item_enabled)(item)
    }
}

pub struct ListViewBuilder<T>
where
    T: 'static,
{
    pub(crate) model: ListViewModel<T>,
}

pub struct ListViewGridColumnsBuilder<T>
where
    T: 'static,
{
    builder: ListViewBuilder<T>,
    columns: Vec<ListViewColumn<T>>,
}

impl ListViewBuilder<ListViewItem> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self::new_typed(id).item_label(|item| item.label.clone()).item_enabled(|item| item.enabled)
    }
}

impl<T> ListViewBuilder<T>
where
    T: 'static,
{
    pub fn new_typed(id: impl Into<SharedString>) -> Self {
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
                item_label: Arc::new(|_| SharedString::default()),
                item_enabled: Arc::new(|_| true),
                template: default_list_view_template(),
                header_template: None,
                item_template: None,
                theme: default_list_view_theme(),
                list_appearance_override: None,
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

    pub fn item_label<F, S>(mut self, label: F) -> Self
    where
        F: Fn(&T) -> S + Send + Sync + 'static,
        S: Into<SharedString>,
    {
        self.model.item_label = Arc::new(move |item| label(item).into());
        self
    }

    pub fn item_enabled<F>(mut self, enabled: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        self.model.item_enabled = Arc::new(enabled);
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

    pub fn list_appearance_override<F>(mut self, override_fn: F) -> Self
    where
        F: Fn(ListViewListAppearance) -> ListViewListAppearance + Send + Sync + 'static,
    {
        self.model.list_appearance_override = Some(Arc::new(override_fn));
        self
    }

    pub fn square_corners(self) -> Self {
        self.list_appearance_override(|mut list| {
            list.radius = 0.0;
            list
        })
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ListViewRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.model.template = list_view_template_with_modifier(std::sync::Arc::clone(&self.model.template), modifier);
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

    pub fn grid_view(mut self, columns: impl IntoIterator<Item = ListViewColumn<T>>) -> Self {
        let columns: Arc<[ListViewColumn<T>]> = columns.into_iter().collect::<Vec<_>>().into();
        let header_columns = Arc::clone(&columns);
        let item_columns = Arc::clone(&columns);

        self.model.header_template = Some(make_list_view_header_template(move |_model, _window, _cx| {
            render_grid_view_header(header_columns.as_ref())
        }));
        self.model.item_template = Some(make_list_view_item_template(move |model, _window, _cx| {
            render_grid_view_row(model.item, item_columns.as_ref())
        }));
        self
    }

    pub fn grid_columns(self) -> ListViewGridColumnsBuilder<T> {
        ListViewGridColumnsBuilder { builder: self, columns: Vec::new() }
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ListViewControl<T>> {
        cx.new(|cx| ListViewControl::from_builder(self, cx))
    }
}

impl<T> ListViewGridColumnsBuilder<T>
where
    T: 'static,
{
    pub fn column_fixed<F, E>(mut self, header: impl Into<SharedString>, width: f32, cell_template: F) -> Self
    where
        F: Fn(&T) -> E + Send + Sync + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.columns.push(ListViewColumn::fixed(header, width, cell_template));
        self
    }

    pub fn column_fill<F, E>(mut self, header: impl Into<SharedString>, cell_template: F) -> Self
    where
        F: Fn(&T) -> E + Send + Sync + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.columns.push(ListViewColumn::fill(header, cell_template));
        self
    }

    pub fn finish(self) -> ListViewBuilder<T> {
        self.builder.grid_view(self.columns)
    }
}

fn render_grid_view_header<T>(columns: &[ListViewColumn<T>]) -> AnyElement
where
    T: 'static,
{
    let mut row = div().w_full().flex().items_center();

    for column in columns {
        row =
            row.child(render_grid_view_header_column_slot(column.width(), column.header().clone().into_any_element()));
    }

    row.into_any_element()
}

fn render_grid_view_row<T>(item: &T, columns: &[ListViewColumn<T>]) -> AnyElement
where
    T: 'static,
{
    let mut row = div().w_full().flex().items_center();

    for column in columns {
        row = row.child(render_grid_view_column_slot(column.width(), column.render_cell(item)));
    }

    row.into_any_element()
}

fn render_grid_view_column_slot(width: ListViewColumnWidth, content: AnyElement) -> AnyElement {
    match width {
        ListViewColumnWidth::Fixed(width) => div()
            .w(px(width))
            .min_w(px(width))
            .max_w(px(width))
            .child(div().w_full().min_w(px(0.0)).px(px(12.0)).truncate().child(content))
            .into_any_element(),
        ListViewColumnWidth::Fill => div()
            .flex_1()
            .min_w(px(0.0))
            .child(div().w_full().min_w(px(0.0)).px(px(12.0)).truncate().child(content))
            .into_any_element(),
    }
}

fn render_grid_view_header_column_slot(width: ListViewColumnWidth, content: AnyElement) -> AnyElement {
    match width {
        ListViewColumnWidth::Fixed(width) => div()
            .w(px(width))
            .min_w(px(width))
            .max_w(px(width))
            .child(div().w_full().min_w(px(0.0)).px(px(12.0)).truncate().child(content))
            .into_any_element(),
        ListViewColumnWidth::Fill => div()
            .flex_1()
            .min_w(px(0.0))
            .child(div().w_full().min_w(px(0.0)).px(px(12.0)).truncate().child(content))
            .into_any_element(),
    }
}
