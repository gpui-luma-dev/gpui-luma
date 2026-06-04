use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Div, Entity, ListAlignment, SharedString, Stateful, Window, div, prelude::*, px};

use super::control::ListViewControl;
use super::template::{ListViewTemplate, default_list_view_template, list_view_template_with_modifier};
use super::theme::{ListViewAppearance, ListViewRowAppearance, ListViewTheme, default_list_view_theme};
use crate::controls::state::ControlFocusState;
use crate::theme::ControlSize;

pub type ListViewLabelFn<T> = Arc<dyn Fn(&T) -> SharedString + Send + Sync + 'static>;
pub type ListViewEnabledFn<T> = Arc<dyn Fn(&T) -> bool + Send + Sync + 'static>;

#[derive(Clone, Debug)]
pub struct ListViewLabel {
    pub(crate) label: SharedString,
    pub(crate) enabled: bool,
}

impl ListViewLabel {
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ListScrollMode {
    /// Smooth pixel-by-pixel scrolling (default).
    #[default]
    ScrollSmooth,
    /// Smooth scrolling with snap-to-row boundaries on release.
    ScrollSnap,
    /// Paginated layout displaying N items at a time with page selectors.
    Paged { page_size: usize },
}

impl ListScrollMode {
    pub fn is_paged(self) -> bool {
        matches!(self, Self::Paged { .. })
    }
}

pub struct ListViewRenderModel<'a> {
    pub id: &'a SharedString,
    pub appearance: ListViewAppearance,
    pub row_appearance: ListViewRowAppearance,
    pub row_count: usize,
    pub selection_mode: ListSelectionMode,
    pub scroll_mode: ListScrollMode,
    pub visible_rows: Option<usize>,
    pub body_rows_height: Option<f32>,
    pub shell_height: Option<f32>,
    pub has_header: bool,
    pub current_page: usize,
    pub page_size: Option<usize>,
    pub page_count: usize,
    pub selected_count: usize,
    pub enabled: bool,
    pub size: ControlSize,
    pub focus: ControlFocusState,
}

pub type ListViewAppearanceOverride = Arc<dyn Fn(ListViewAppearance) -> ListViewAppearance + Send + Sync + 'static>;

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
pub struct ListViewRowRenderModel<'a, T>
where
    T: 'static,
{
    pub list_id: &'a SharedString,
    pub row: &'a T,
    pub index: usize,
    pub sibling_count: usize,
    pub selected: bool,
    pub active: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub enabled: bool,
    pub appearance: ListViewRowAppearance,
}

pub type ListViewRowTemplate<T> = Arc<
    dyn for<'a> Fn(&ListViewRowRenderModel<'a, T>, AnyElement, &mut Window, &mut App) -> AnyElement
        + Send
        + Sync
        + 'static,
>;

pub fn make_list_view_row_template<T, F, E>(template: F) -> ListViewRowTemplate<T>
where
    T: 'static,
    F: for<'a> Fn(&ListViewRowRenderModel<'a, T>, AnyElement, &mut Window, &mut App) -> E + Send + Sync + 'static,
    E: gpui::IntoElement + 'static,
{
    Arc::new(move |model, cells, window, cx| template(model, cells, window, cx).into_any_element())
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ListViewColumnWidth {
    Fixed(f32),
    #[default]
    Fill,
}

/// Horizontal padding and overflow behavior for a grid column cell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListViewColumnCellLayout {
    /// Text columns: horizontal inset and ellipsis overflow.
    #[default]
    Text,
    /// Embedded controls (checkbox, icon button): centered, no truncate.
    Control,
}

pub type ListViewColumnCellTemplate<T> =
    Arc<dyn for<'a> Fn(&ListViewRowRenderModel<'a, T>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static>;

pub trait IntoListViewColumnCellTemplate<T> {
    fn into_cell_template(self) -> ListViewColumnCellTemplate<T>;
}

impl<T> IntoListViewColumnCellTemplate<T> for ListViewColumnCellTemplate<T>
where
    T: 'static,
{
    fn into_cell_template(self) -> ListViewColumnCellTemplate<T> {
        self
    }
}

impl<T, F, E> IntoListViewColumnCellTemplate<T> for F
where
    T: 'static,
    F: Fn(&T) -> E + Send + Sync + 'static,
    E: gpui::IntoElement + 'static,
{
    fn into_cell_template(self) -> ListViewColumnCellTemplate<T> {
        Arc::new(move |model, _window, _cx| (self)(model.row).into_any_element())
    }
}

pub struct ListViewColumn<T>
where
    T: 'static,
{
    header: SharedString,
    width: ListViewColumnWidth,
    cell_layout: ListViewColumnCellLayout,
    cell_template: ListViewColumnCellTemplate<T>,
}

impl<T> Clone for ListViewColumn<T>
where
    T: 'static,
{
    fn clone(&self) -> Self {
        Self {
            header: self.header.clone(),
            width: self.width,
            cell_layout: self.cell_layout,
            cell_template: self.cell_template.clone(),
        }
    }
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
        Self { header: header.into(), width, cell_layout: ListViewColumnCellLayout::Text, cell_template }
    }

    pub fn fixed(
        header: impl Into<SharedString>,
        width: f32,
        cell_template: impl IntoListViewColumnCellTemplate<T>,
    ) -> Self {
        Self::fixed_with_layout(header, width, ListViewColumnCellLayout::Text, cell_template)
    }

    /// Fixed-width column for embedded controls (checkboxes, icon buttons).
    pub fn fixed_control(
        header: impl Into<SharedString>,
        width: f32,
        cell_template: impl IntoListViewColumnCellTemplate<T>,
    ) -> Self {
        Self::fixed_with_layout(header, width, ListViewColumnCellLayout::Control, cell_template)
    }

    pub fn fixed_with_layout(
        header: impl Into<SharedString>,
        width: f32,
        cell_layout: ListViewColumnCellLayout,
        cell_template: impl IntoListViewColumnCellTemplate<T>,
    ) -> Self {
        Self {
            header: header.into(),
            width: ListViewColumnWidth::Fixed(width),
            cell_layout,
            cell_template: cell_template.into_cell_template(),
        }
    }

    pub fn fill(header: impl Into<SharedString>, cell_template: impl IntoListViewColumnCellTemplate<T>) -> Self {
        Self {
            header: header.into(),
            width: ListViewColumnWidth::Fill,
            cell_layout: ListViewColumnCellLayout::Text,
            cell_template: cell_template.into_cell_template(),
        }
    }

    pub fn cell_layout(&self) -> ListViewColumnCellLayout {
        self.cell_layout
    }

    pub fn header(&self) -> &SharedString {
        &self.header
    }

    pub fn width(&self) -> ListViewColumnWidth {
        self.width
    }

    pub fn render_cell(&self, model: &ListViewRowRenderModel<'_, T>, window: &mut Window, cx: &mut App) -> AnyElement {
        (self.cell_template)(model, window, cx)
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
    /// When `false`, row clicks update the active row only; selection changes via [`ListViewControl::set_selected_indices`] / [`ListViewControl::toggle_selected_index`].
    pub(crate) select_on_row_click: bool,
    pub(crate) enabled: bool,
    pub(crate) size: ControlSize,
    pub(crate) alignment: ListAlignment,
    pub(crate) overdraw: f32,
    pub(crate) row_label: ListViewLabelFn<T>,
    pub(crate) row_enabled: ListViewEnabledFn<T>,
    pub(crate) template: Arc<dyn ListViewTemplate>,
    pub(crate) header_template: Option<ListViewHeaderTemplate>,
    pub(crate) row_template: Option<ListViewRowTemplate<T>>,
    pub(crate) theme: Arc<dyn ListViewTheme>,
    pub(crate) appearance_override: Option<ListViewAppearanceOverride>,
    pub(crate) columns: Vec<ListViewColumn<T>>,
    pub(crate) scroll_mode: ListScrollMode,
    pub(crate) visible_rows: Option<usize>,
    pub(crate) visible_row_height: Option<f32>,
}

impl<T> ListViewModel<T>
where
    T: 'static,
{
    pub(crate) fn label_for_row(&self, row: &T) -> SharedString {
        (self.row_label)(row)
    }

    pub(crate) fn row_is_enabled(&self, row: &T) -> bool {
        (self.row_enabled)(row)
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

impl ListViewBuilder<ListViewLabel> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self::new_typed(id).row_label(|row| row.label.clone()).row_enabled(|row| row.enabled)
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
                select_on_row_click: true,
                enabled: true,
                size: ControlSize::Md,
                alignment: ListAlignment::Top,
                overdraw: 480.0,
                row_label: Arc::new(|_| SharedString::default()),
                row_enabled: Arc::new(|_| true),
                template: default_list_view_template(),
                header_template: None,
                row_template: None,
                theme: default_list_view_theme(),
                appearance_override: None,
                columns: Vec::new(),
                scroll_mode: ListScrollMode::ScrollSmooth,
                visible_rows: None,
                visible_row_height: None,
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

    /// Data-table style: keep [`ListSelectionMode::Multiple`] but toggle selection only from embedded controls (e.g. row checkboxes).
    pub fn select_on_row_click(mut self, select_on_row_click: bool) -> Self {
        self.model.select_on_row_click = select_on_row_click;
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

    pub fn scroll_mode(mut self, mode: ListScrollMode) -> Self {
        self.model.scroll_mode = mode;
        self
    }

    pub fn paged(self, page_size: usize) -> Self {
        self.scroll_mode(ListScrollMode::Paged { page_size: page_size.max(1) })
    }

    pub fn scroll_snap(self, enabled: bool) -> Self {
        if enabled {
            self.scroll_mode(ListScrollMode::ScrollSnap)
        } else {
            self.scroll_mode(ListScrollMode::ScrollSmooth)
        }
    }

    pub fn visible_rows(mut self, count: usize) -> Self {
        self.model.visible_rows = Some(count.max(1));
        self
    }

    /// Overrides the per-row height used for [`Self::visible_rows`] shell sizing.
    ///
    /// Use when a custom [`Self::row_template`] applies vertical padding that differs
    /// from the default row chrome (`min_height + 2 * padding_y`).
    pub fn visible_row_height(mut self, height: impl Into<f32>) -> Self {
        self.model.visible_row_height = Some(height.into().max(1.0));
        self
    }

    pub fn row_label<F, S>(mut self, label: F) -> Self
    where
        F: Fn(&T) -> S + Send + Sync + 'static,
        S: Into<SharedString>,
    {
        self.model.row_label = Arc::new(move |row| label(row).into());
        self
    }

    pub fn row_enabled<F>(mut self, enabled: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        self.model.row_enabled = Arc::new(enabled);
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

    pub fn appearance_override<F>(mut self, override_fn: F) -> Self
    where
        F: Fn(ListViewAppearance) -> ListViewAppearance + Send + Sync + 'static,
    {
        self.model.appearance_override = Some(Arc::new(override_fn));
        self
    }

    pub fn square_corners(self) -> Self {
        self.appearance_override(|mut appearance| {
            appearance.radius = 0.0;
            appearance
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

    pub fn row_template(mut self, row_template: ListViewRowTemplate<T>) -> Self {
        self.model.row_template = Some(row_template);
        self
    }

    pub fn with_row_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&ListViewRowRenderModel<'a, T>, AnyElement, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.model.row_template = Some(make_list_view_row_template(template));
        self
    }

    pub fn grid_view(mut self, columns: impl IntoIterator<Item = ListViewColumn<T>>) -> Self {
        let columns: Vec<ListViewColumn<T>> = columns.into_iter().collect();
        let header_columns = columns.clone();
        self.model.columns = columns;

        self.model.header_template =
            Some(make_list_view_header_template(move |_model, _window, _cx| render_grid_view_header(&header_columns)));
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
        row = row.child(render_grid_view_header_column_slot(
            column.width(),
            column.cell_layout(),
            column.header().clone().into_any_element(),
        ));
    }

    row.into_any_element()
}

pub(crate) fn render_grid_view_cells<T>(
    model: &ListViewRowRenderModel<'_, T>,
    columns: &[ListViewColumn<T>],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement
where
    T: 'static,
{
    let mut cells = div().w_full().flex().items_center();

    for column in columns {
        cells = cells.child(render_grid_view_column_slot(
            column.width(),
            column.cell_layout(),
            column.render_cell(model, window, cx),
        ));
    }

    cells.into_any_element()
}

fn render_grid_view_column_slot(
    width: ListViewColumnWidth,
    cell_layout: ListViewColumnCellLayout,
    content: AnyElement,
) -> AnyElement {
    match (width, cell_layout) {
        (ListViewColumnWidth::Fixed(width), ListViewColumnCellLayout::Control) => div()
            .w(px(width))
            .min_w(px(width))
            .max_w(px(width))
            .flex()
            .items_center()
            .justify_center()
            .child(content)
            .into_any_element(),
        (ListViewColumnWidth::Fixed(width), ListViewColumnCellLayout::Text) => div()
            .w(px(width))
            .min_w(px(width))
            .max_w(px(width))
            .child(div().w_full().min_w(px(0.0)).px(px(12.0)).truncate().child(content))
            .into_any_element(),
        (ListViewColumnWidth::Fill, ListViewColumnCellLayout::Control) => div()
            .flex_1()
            .min_w(px(0.0))
            .flex()
            .items_center()
            .justify_center()
            .child(content)
            .into_any_element(),
        (ListViewColumnWidth::Fill, ListViewColumnCellLayout::Text) => div()
            .flex_1()
            .min_w(px(0.0))
            .child(div().w_full().min_w(px(0.0)).px(px(12.0)).truncate().child(content))
            .into_any_element(),
    }
}

fn render_grid_view_header_column_slot(
    width: ListViewColumnWidth,
    cell_layout: ListViewColumnCellLayout,
    content: AnyElement,
) -> AnyElement {
    match (width, cell_layout) {
        (ListViewColumnWidth::Fixed(width), ListViewColumnCellLayout::Control) => div()
            .w(px(width))
            .min_w(px(width))
            .max_w(px(width))
            .flex()
            .items_center()
            .justify_center()
            .child(content)
            .into_any_element(),
        (ListViewColumnWidth::Fixed(width), ListViewColumnCellLayout::Text) => div()
            .w(px(width))
            .min_w(px(width))
            .max_w(px(width))
            .flex()
            .items_center()
            .justify_center()
            .child(grid_view_header_label_slot(content))
            .into_any_element(),
        (ListViewColumnWidth::Fill, ListViewColumnCellLayout::Control) => div()
            .flex_1()
            .min_w(px(0.0))
            .flex()
            .items_center()
            .justify_center()
            .child(content)
            .into_any_element(),
        (ListViewColumnWidth::Fill, ListViewColumnCellLayout::Text) => div()
            .flex_1()
            .min_w(px(0.0))
            .flex()
            .items_center()
            .justify_center()
            .child(grid_view_header_label_slot(content))
            .into_any_element(),
    }
}

fn grid_view_header_label_slot(content: AnyElement) -> impl IntoElement {
    div()
        .w_full()
        .min_w(px(0.0))
        .px(px(12.0))
        .flex()
        .items_center()
        .justify_center()
        .truncate()
        .child(content)
}
