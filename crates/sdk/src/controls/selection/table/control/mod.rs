mod input;
mod render;
mod selection;
mod viewport;

use gpui::{
    App, Context, EventEmitter, Focusable, IntoElement, ListOffset, ListState, SharedString, Subscription, Window, px,
};

use super::layout::{clamp_page, page_after_size_change, page_count};
use super::model::{
    TableBuilder, TableHeaderTemplate, TableLabel, TableLookOverride, TableModel, TableRenderModel,
    TableRowRenderModel, TableScrollMode, TableSelectionMode, make_table_header_template, make_table_row_template,
};
use super::template::TableTemplate;
use super::theme::TableTheme;
use crate::theme::ControlSize;
use crate::theme::observe_theme_revision;

use self::selection::normalize_model;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum TableEvent {
    SelectionChanged { selected_indices: Vec<usize> },
    ActiveIndexChanged { active_index: Option<usize> },
    PageChanged { page: usize },
    PageSizeChanged { page_size: usize },
    ScrollChanged { top_index: usize },
    FocusChanged { focused: bool },
    RowHoverChanged { index: usize, hovered: bool },
    EnabledChanged { enabled: bool },
}

pub struct TableControl<T>
where
    T: 'static,
{
    model: TableModel<T>,
    list_state: ListState,
    focus_handle: gpui::FocusHandle,
    focus_in_subscription: Option<Subscription>,
    focus_out_subscription: Option<Subscription>,
    hovered_index: Option<usize>,
    pressed_index: Option<usize>,
    current_page: usize,
    emitted_scroll_top_index: usize,
    emitted_focused: bool,
    /// When fill-height + paged, row height stretched so `page_size` rows fill the viewport.
    fill_row_height: Option<f32>,
}

impl<T> EventEmitter<TableEvent> for TableControl<T> where T: 'static {}

impl TableControl<TableLabel> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> TableBuilder<TableLabel> {
        TableBuilder::new(id)
    }
}

impl<T> TableControl<T>
where
    T: 'static,
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new_typed(id: impl Into<SharedString>) -> TableBuilder<T> {
        TableBuilder::new_typed(id)
    }

    pub(crate) fn from_builder(mut builder: TableBuilder<T>, cx: &mut Context<Self>) -> Self {
        normalize_model(&mut builder.model);
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();

        let visible_count = Self::visible_list_item_count(&builder.model, 0);
        let mut list_state = ListState::new(visible_count, builder.model.alignment, px(builder.model.overdraw));
        if builder.model.visible_rows.is_some() || builder.model.fill_height {
            // Fixed-viewport / fill-height lists still scroll through the full item set.
            // Without measuring every row, GPUI's internal height sum stays at the visible
            // slice and scroll_max collapses to zero when overdraw is small.
            list_state = list_state.measure_all();
        }
        Self {
            list_state,
            focus_handle: cx.focus_handle().tab_stop(builder.model.enabled),
            focus_in_subscription: None,
            focus_out_subscription: None,
            model: builder.model,
            hovered_index: None,
            pressed_index: None,
            current_page: 0,
            emitted_scroll_top_index: 0,
            emitted_focused: false,
            fill_row_height: None,
        }
    }

    pub fn items(&self) -> &[T] {
        &self.model.items
    }

    pub fn selected_indices(&self) -> &[usize] {
        &self.model.selected_indices
    }

    pub fn active_index(&self) -> Option<usize> {
        self.model.active_index
    }

    pub fn selection_mode(&self) -> TableSelectionMode {
        self.model.selection_mode
    }

    pub fn scroll_mode(&self) -> TableScrollMode {
        self.model.scroll_mode
    }

    pub fn fills_height(&self) -> bool {
        self.model.fill_height
    }

    pub fn is_enabled(&self) -> bool {
        self.model.enabled
    }

    pub fn current_page(&self) -> usize {
        self.current_page
    }

    pub fn page_size(&self) -> Option<usize> {
        match self.model.scroll_mode {
            TableScrollMode::Paged { page_size } => Some(page_size.max(1)),
            _ => None,
        }
    }

    pub fn page_count(&self) -> usize {
        page_count(self.model.items.len(), self.page_size().unwrap_or(1))
    }

    pub fn selected_items(&self) -> Vec<&T> {
        self.model.selected_indices.iter().filter_map(|index| self.model.items.get(*index)).collect()
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = T>, cx: &mut Context<Self>) {
        self.model.items = items.into_iter().collect();
        self.hovered_index = None;
        self.pressed_index = None;
        self.current_page = clamp_page(self.current_page, self.model.items.len(), self.page_size().unwrap_or(1));
        self.sync_list_state();
        normalize_model(&mut self.model);
        cx.notify();
    }

    pub fn set_selected_indices(&mut self, selected_indices: impl IntoIterator<Item = usize>, cx: &mut Context<Self>) {
        let previous = self.model.selected_indices.clone();
        self.model.selected_indices = selected_indices.into_iter().collect();
        normalize_model(&mut self.model);
        let next = self.model.selected_indices.clone();
        if next != previous {
            cx.emit(TableEvent::SelectionChanged { selected_indices: next });
        }
        cx.notify();
    }

    /// Toggles selection for `index` using the same rules as a row click.
    pub fn toggle_selected_index(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        self.commit_select_index(index, cx)
    }

    pub fn set_active_index(&mut self, active_index: Option<usize>, cx: &mut Context<Self>) {
        self.set_active_index_internal(active_index, None, cx);
    }

    pub fn set_selection_mode(&mut self, selection_mode: TableSelectionMode, cx: &mut Context<Self>) {
        if self.model.selection_mode == selection_mode {
            return;
        }

        self.model.selection_mode = selection_mode;
        normalize_model(&mut self.model);
        cx.notify();
    }

    pub fn set_scroll_mode(&mut self, scroll_mode: TableScrollMode, cx: &mut Context<Self>) {
        if self.model.scroll_mode == scroll_mode {
            return;
        }

        self.model.scroll_mode = scroll_mode;
        self.current_page = clamp_page(self.current_page, self.model.items.len(), self.page_size().unwrap_or(1));
        self.sync_list_state();
        cx.notify();
    }

    pub fn set_page(&mut self, page: usize, cx: &mut Context<Self>) {
        if !self.is_paged() {
            return;
        }

        let next = clamp_page(page, self.model.items.len(), self.page_size().unwrap_or(1));
        if self.current_page == next {
            return;
        }

        self.current_page = next;
        self.sync_list_state();
        cx.emit(TableEvent::PageChanged { page: next });
        cx.notify();
    }

    pub fn next_page(&mut self, cx: &mut Context<Self>) {
        self.set_page(self.current_page.saturating_add(1), cx);
    }

    pub fn prev_page(&mut self, cx: &mut Context<Self>) {
        self.set_page(self.current_page.saturating_sub(1), cx);
    }

    pub fn first_page(&mut self, cx: &mut Context<Self>) {
        self.set_page(0, cx);
    }

    pub fn last_page(&mut self, cx: &mut Context<Self>) {
        let last = self.page_count().saturating_sub(1);
        self.set_page(last, cx);
    }

    pub fn set_page_size(&mut self, page_size: usize, cx: &mut Context<Self>) {
        let page_size = page_size.max(1);
        if self.page_size() == Some(page_size) {
            return;
        }

        let old_page_size = self.page_size().unwrap_or(1);
        let keep_in_view = self.focus_index_for_page_size_change();
        let next_page =
            page_after_size_change(self.current_page, old_page_size, page_size, self.model.items.len(), keep_in_view);
        self.model.scroll_mode = TableScrollMode::Paged { page_size };
        self.current_page = next_page;
        self.sync_list_state();
        cx.emit(TableEvent::PageSizeChanged { page_size });
        cx.emit(TableEvent::PageChanged { page: self.current_page });
        cx.notify();
    }

    /// Row to keep visible across a page-size change: active when selected (or when nothing is
    /// selected), otherwise the first selected index.
    fn focus_index_for_page_size_change(&self) -> Option<usize> {
        if let Some(active) = self.model.active_index
            && (self.model.selected_indices.is_empty() || self.model.selected_indices.contains(&active))
        {
            return Some(active);
        }
        self.model.selected_indices.first().copied()
    }

    pub fn scroll_to_row(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.is_paged() {
            let page_size = self.page_size().unwrap_or(1);
            self.set_page(index / page_size, cx);
            self.set_active_index_internal(Some(index), None, cx);
            return;
        }

        if self.is_scroll_snap() {
            self.list_state.scroll_to(ListOffset { item_ix: index, offset_in_item: px(0.0) });
        } else {
            self.list_state.scroll_to_reveal_item(index);
        }
        self.emit_scroll_changed_if_needed(cx);
        cx.notify();
    }

    pub fn scroll_by_pixels(&mut self, pixels: f32, cx: &mut Context<Self>) {
        if self.is_paged() || !pixels.is_finite() {
            return;
        }

        self.list_state.scroll_by(px(pixels));
        if self.is_scroll_snap() {
            self.snap_scroll_position();
        }
        self.emit_scroll_changed_if_needed(cx);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        self.focus_in_subscription = None;
        self.focus_out_subscription = None;

        if !enabled {
            self.clear_hovered_index(cx);
            self.pressed_index = None;
            self.emit_focus_changed(false, cx);
        }

        cx.emit(TableEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_size(&mut self, size: ControlSize, cx: &mut Context<Self>) {
        if self.model.size == size {
            return;
        }

        self.model.size = size;
        self.list_state.remeasure();
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn TableTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: std::sync::Arc<dyn TableTheme>, cx: &mut Context<Self>) {
        self.model.theme = theme;
        self.list_state.remeasure();
        cx.notify();
    }

    pub fn set_look_override(&mut self, look_override: Option<TableLookOverride>, cx: &mut Context<Self>) {
        self.model.look_override = look_override;
        self.list_state.remeasure();
        cx.notify();
    }

    pub fn set_header_template(&mut self, header_template: Option<TableHeaderTemplate>, cx: &mut Context<Self>) {
        self.model.header_template = header_template;
        cx.notify();
    }

    pub fn set_header_template_fn<F, E>(&mut self, template: F, cx: &mut Context<Self>)
    where
        F: for<'a> Fn(&TableRenderModel<'a>, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.header_template = Some(make_table_header_template(template));
        cx.notify();
    }

    pub fn clear_header_template(&mut self, cx: &mut Context<Self>) {
        if self.model.header_template.is_some() {
            self.model.header_template = None;
            cx.notify();
        }
    }

    pub fn set_row_template_fn<F, E>(&mut self, template: F, cx: &mut Context<Self>)
    where
        F: for<'a> Fn(&TableRowRenderModel<'a, T>, gpui::AnyElement, &mut Window, &mut App) -> E
            + Send
            + Sync
            + 'static,
        E: IntoElement + 'static,
    {
        self.model.row_template = Some(make_table_row_template(template));
        self.list_state.remeasure();
        cx.notify();
    }

    pub fn clear_row_template(&mut self, cx: &mut Context<Self>) {
        if self.model.row_template.is_some() {
            self.model.row_template = None;
            self.list_state.remeasure();
            cx.notify();
        }
    }
}

impl<T> Focusable for TableControl<T>
where
    T: 'static,
{
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}
