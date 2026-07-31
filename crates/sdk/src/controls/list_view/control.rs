use gpui::{
    App, ClickEvent, Context, EventEmitter, FocusOutEvent, Focusable, IntoElement, ListOffset, ListState, MouseButton,
    MouseDownEvent, MouseUpEvent, Render, ScrollWheelEvent, SharedString, Subscription, TouchPhase, Window, div, list,
    prelude::*, px,
};

use super::layout::{
    clamp_page, compute_shell_height, distributed_row_height, effective_visible_rows, page_after_size_change,
    page_count, page_start, rows_for_viewport_height_border_box, visible_item_count, visible_row_height,
};
use super::model::{
    ListScrollMode, ListSelectionMode, ListViewBuilder, ListViewHeaderTemplate, ListViewLabel, ListViewModel,
    ListViewRenderModel, ListViewRowRenderModel, make_list_view_header_template, make_list_view_row_template,
};
use super::row::render_list_view_row;
use super::template::ListViewTemplate;
use super::theme::{ListViewLook, ListViewTheme};
use super::model::ListViewLookOverride;
use crate::controls::state::ControlFocusState;
use crate::keyhandling::{
    ActivateControl, ControlKeyProfile, DecreaseValueLarge, IncreaseValueLarge, SelectFirstItem, SelectLastItem,
    SelectNextItem, SelectPreviousItem,
};
use crate::theme::adorner::adorner_oversize_extent;
use crate::theme::{ControlSize, InteractionState, LayoutCacheKey, ListRowScale, LumaLayoutCacheExt};
use crate::theme::observe_theme_revision;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ListViewEvent {
    SelectionChanged { selected_indices: Vec<usize> },
    ActiveIndexChanged { active_index: Option<usize> },
    PageChanged { page: usize },
    PageSizeChanged { page_size: usize },
    ScrollChanged { top_index: usize },
    FocusChanged { focused: bool },
    RowHoverChanged { index: usize, hovered: bool },
    EnabledChanged { enabled: bool },
}

pub struct ListViewControl<T>
where
    T: 'static,
{
    model: ListViewModel<T>,
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

impl<T> EventEmitter<ListViewEvent> for ListViewControl<T> where T: 'static {}

impl ListViewControl<ListViewLabel> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ListViewBuilder<ListViewLabel> {
        ListViewBuilder::new(id)
    }
}

impl<T> ListViewControl<T>
where
    T: 'static,
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new_typed(id: impl Into<SharedString>) -> ListViewBuilder<T> {
        ListViewBuilder::new_typed(id)
    }

    pub(crate) fn from_builder(mut builder: ListViewBuilder<T>, cx: &mut Context<Self>) -> Self {
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

    pub fn selection_mode(&self) -> ListSelectionMode {
        self.model.selection_mode
    }

    pub fn scroll_mode(&self) -> ListScrollMode {
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
            ListScrollMode::Paged { page_size } => Some(page_size.max(1)),
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
            cx.emit(ListViewEvent::SelectionChanged { selected_indices: next });
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

    pub fn set_selection_mode(&mut self, selection_mode: ListSelectionMode, cx: &mut Context<Self>) {
        if self.model.selection_mode == selection_mode {
            return;
        }

        self.model.selection_mode = selection_mode;
        normalize_model(&mut self.model);
        cx.notify();
    }

    pub fn set_scroll_mode(&mut self, scroll_mode: ListScrollMode, cx: &mut Context<Self>) {
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
        cx.emit(ListViewEvent::PageChanged { page: next });
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
        self.model.scroll_mode = ListScrollMode::Paged { page_size };
        self.current_page = next_page;
        self.sync_list_state();
        cx.emit(ListViewEvent::PageSizeChanged { page_size });
        cx.emit(ListViewEvent::PageChanged { page: self.current_page });
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

        cx.emit(ListViewEvent::EnabledChanged { enabled });
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

    pub fn set_template(&mut self, template: std::sync::Arc<dyn ListViewTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: std::sync::Arc<dyn ListViewTheme>, cx: &mut Context<Self>) {
        self.model.theme = theme;
        self.list_state.remeasure();
        cx.notify();
    }

    pub fn set_look_override(&mut self, look_override: Option<ListViewLookOverride>, cx: &mut Context<Self>) {
        self.model.look_override = look_override;
        self.list_state.remeasure();
        cx.notify();
    }

    pub fn set_header_template(&mut self, header_template: Option<ListViewHeaderTemplate>, cx: &mut Context<Self>) {
        self.model.header_template = header_template;
        cx.notify();
    }

    pub fn set_header_template_fn<F, E>(&mut self, template: F, cx: &mut Context<Self>)
    where
        F: for<'a> Fn(&ListViewRenderModel<'a>, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.header_template = Some(make_list_view_header_template(template));
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
        F: for<'a> Fn(&ListViewRowRenderModel<'a, T>, gpui::AnyElement, &mut Window, &mut App) -> E
            + Send
            + Sync
            + 'static,
        E: IntoElement + 'static,
    {
        self.model.row_template = Some(make_list_view_row_template(template));
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

    fn is_paged(&self) -> bool {
        matches!(self.model.scroll_mode, ListScrollMode::Paged { .. })
    }

    fn is_scroll_snap(&self) -> bool {
        matches!(self.model.scroll_mode, ListScrollMode::ScrollSnap)
    }

    fn visible_list_item_count(model: &ListViewModel<T>, current_page: usize) -> usize {
        if let ListScrollMode::Paged { page_size } = model.scroll_mode {
            visible_item_count(model.items.len(), current_page, page_size)
        } else {
            model.items.len()
        }
    }

    fn sync_list_state(&mut self) {
        let count = Self::visible_list_item_count(&self.model, self.current_page);
        if self.list_state.item_count() != count {
            self.list_state.reset(count);
        }
    }

    fn local_to_global_index(&self, local_index: usize) -> usize {
        if self.is_paged() {
            page_start(self.current_page, self.page_size().unwrap_or(1)) + local_index
        } else {
            local_index
        }
    }

    fn ensure_page_for_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if !self.is_paged() {
            return;
        }

        let page_size = self.page_size().unwrap_or(1);
        let page = index / page_size;
        if page != self.current_page {
            self.set_page(page, cx);
        }
    }

    fn row_scroll_increment(&self, row_look: &super::theme::ListViewRowLook) -> f32 {
        self.fill_row_height.unwrap_or_else(|| visible_row_height(row_look, self.model.visible_row_height))
    }

    fn token_row_height(&self, scale_factor: f32) -> f32 {
        visible_row_height(&self.resolve_row_look_for_metrics(scale_factor), self.model.visible_row_height)
    }

    fn visible_row_capacity(&self) -> usize {
        if let Some(count) =
            effective_visible_rows(self.model.visible_rows, self.model.scroll_mode, self.model.fill_height)
        {
            return count;
        }

        if self.should_sync_page_size_to_viewport() {
            return self.page_size().unwrap_or(1);
        }

        self.viewport_page_size(1.0).unwrap_or(1)
    }

    fn should_sync_page_size_to_viewport(&self) -> bool {
        self.model.fill_height && self.is_paged() && self.model.visible_rows.is_none()
    }

    fn viewport_page_size(&self, scale_factor: f32) -> Option<usize> {
        let viewport_height = self.list_state.viewport_bounds().size.height.as_f32();
        // Fill-height rows paint dividers as border_t inside `.h(...)`.
        rows_for_viewport_height_border_box(viewport_height, self.token_row_height(scale_factor))
    }

    /// When fill-height + paged, match `page_size` to the viewport and stretch rows to fill it.
    fn sync_fill_height_page_size(&mut self, window: &Window, cx: &mut Context<Self>) {
        if !self.should_sync_page_size_to_viewport() {
            return;
        }
        let viewport_height = self.list_state.viewport_bounds().size.height.as_f32();
        let Some(page_size) = self.viewport_page_size(window.scale_factor()) else {
            return;
        };
        let Some(row_height) = distributed_row_height(viewport_height, page_size) else {
            return;
        };

        let height_changed = self.fill_row_height.is_none_or(|current| (current - row_height).abs() > 0.05);
        self.fill_row_height = Some(row_height);

        if self.page_size() != Some(page_size) {
            self.set_page_size(page_size, cx);
            self.list_state.remeasure();
        } else if height_changed {
            self.list_state.remeasure();
            cx.notify();
        }
    }

    fn is_active_index_in_viewport(&self, index: usize) -> bool {
        let scroll_top = self.list_state.logical_scroll_top();
        let visible_rows = self.visible_row_capacity().max(1);
        let first = scroll_top.item_ix;
        let last = first.saturating_add(visible_rows.saturating_sub(1));
        index >= first && index <= last
    }

    /// Keep the active row rendered inside the viewport. When moving up within the
    /// first page, pin scroll at row 0 so the list top stays anchored.
    fn scroll_active_into_view(&mut self, index: usize, direction: Option<ListDirection>, cx: &mut Context<Self>) {
        let visible_rows = self.visible_row_capacity().max(1);

        if direction == Some(ListDirection::Previous) && index < visible_rows {
            self.list_state.scroll_to(ListOffset { item_ix: 0, offset_in_item: px(0.0) });
            self.emit_scroll_changed_if_needed(cx);
            return;
        }

        if self.is_active_index_in_viewport(index) {
            return;
        }

        if index < visible_rows {
            self.list_state.scroll_to(ListOffset { item_ix: 0, offset_in_item: px(0.0) });
        } else {
            self.list_state.scroll_to_reveal_item(index);
        }
        self.emit_scroll_changed_if_needed(cx);
    }

    fn snap_scroll_position(&mut self) {
        let row_look = self.resolve_row_look_for_metrics(1.0);
        let row_height = self.row_scroll_increment(&row_look);
        let offset = self.list_state.logical_scroll_top();
        let mut item_ix = offset.item_ix;
        if row_height > 0.0 && offset.offset_in_item >= px(row_height * 0.5) {
            item_ix += 1;
        }
        self.list_state.scroll_to(ListOffset { item_ix, offset_in_item: px(0.0) });
    }

    fn resolve_look(&self, window: &Window) -> ListViewLook {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let mut look = self.model.theme.resolve_look(self.model.enabled, focus.focused, self.model.size);
        if let Some(override_fn) = &self.model.look_override {
            look = override_fn(look);
        }
        look
    }

    fn resolve_row_look(&self, window: &Window, cx: &mut Context<Self>) -> super::theme::ListViewRowLook {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.model.theme.metrics(),
            LayoutCacheKey { size: self.model.size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| ListRowScale::compute(self.model.size, metrics, scale_factor),
        );
        self.model.theme.resolve_row_look(
            false,
            InteractionState { focused: focus.focused, ..Default::default() },
            self.model.size,
            &scale,
        )
    }

    fn resolve_row_look_for_metrics(&self, scale_factor: f32) -> super::theme::ListViewRowLook {
        let scale = ListRowScale::compute(self.model.size, &self.model.theme.metrics(), scale_factor);
        self.model.theme.resolve_row_look(false, InteractionState::default(), self.model.size, &scale)
    }

    fn render_model(&self, window: &Window, cx: &mut Context<Self>) -> ListViewRenderModel<'_> {
        let look = self.resolve_look(window);
        let row_look = self.resolve_row_look(window, cx);
        let has_header = self.model.header_template.is_some();
        let visible_rows =
            effective_visible_rows(self.model.visible_rows, self.model.scroll_mode, self.model.fill_height);
        let row_height = visible_row_height(&row_look, self.model.visible_row_height);
        let body_rows_height = visible_rows.map(|count| super::layout::body_rows_height(count, row_height));
        let shell_height = visible_rows.map(|count| compute_shell_height(count, row_height, &look, has_header));

        ListViewRenderModel {
            id: &self.model.id,
            look,
            row_look,
            row_count: self.model.items.len(),
            selection_mode: self.model.selection_mode,
            scroll_mode: self.model.scroll_mode,
            visible_rows,
            body_rows_height,
            shell_height,
            has_header,
            current_page: self.current_page,
            page_size: self.page_size(),
            page_count: self.page_count(),
            selected_count: self.model.selected_indices.len(),
            enabled: self.model.enabled,
            size: self.model.size,
            focus: ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window),
        }
    }

    fn render_row(&mut self, local_index: usize, window: &mut Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let index = self.local_to_global_index(local_index);
        let Some(item) = self.model.items.get(index) else {
            return div().into_any_element();
        };

        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let enabled = self.model.enabled && self.model.row_is_enabled(item);
        let selected = self.model.selected_indices.contains(&index);
        let active = self.model.active_index == Some(index);
        let hovered = enabled && self.hovered_index == Some(index);
        let pressed = enabled && self.pressed_index == Some(index);
        let keyboard_active = active && focus.focused;
        let interaction =
            InteractionState { hovered, pressed, focused: keyboard_active, disabled: !enabled && !keyboard_active };
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.model.theme.metrics(),
            LayoutCacheKey { size: self.model.size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| ListRowScale::compute(self.model.size, metrics, scale_factor),
        );
        let mut look = self.model.theme.resolve_row_look(selected, interaction, self.model.size, &scale);
        if let Some(fill_height) = self.fill_row_height {
            look.min_height = fill_height;
        }
        let focused_probe_look = if enabled || active {
            let mut focused_probe_state = interaction;
            focused_probe_state.focused = true;
            Some(self.model.theme.resolve_row_look(selected, focused_probe_state, self.model.size, &scale))
        } else {
            None
        };
        let row_oversize_extent = adorner_oversize_extent(look.adorner)
            .max(focused_probe_look.as_ref().map(|probe| adorner_oversize_extent(probe.adorner)).unwrap_or(0.0));

        let row_model = ListViewRowRenderModel {
            list_id: &self.model.id,
            row: item,
            index,
            sibling_count: self.model.items.len(),
            selected,
            active,
            hovered,
            pressed,
            focused: focus.focused,
            focus_visible: focus.focus_visible,
            enabled,
            look: look.clone(),
        };

        let cells = if !self.model.columns.is_empty() {
            super::model::render_grid_view_cells(&row_model, &self.model.columns, window, cx)
        } else {
            div().flex_1().min_w(px(0.0)).truncate().child(self.model.label_for_row(item)).into_any_element()
        };

        let is_custom = self.model.row_template.is_some();
        let content = if let Some(row_template) = self.model.row_template.as_ref() {
            row_template(&row_model, cells, window, cx)
        } else {
            cells
        };

        let row = render_list_view_row(
            format!("{}-row-{}", self.model.id, index),
            content,
            look,
            enabled,
            local_index > 0,
            is_custom,
            self.fill_row_height,
        )
        .on_hover(cx.listener(move |this, hovered, _window, cx| {
            this.handle_item_hover(index, *hovered, cx);
        }))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, event, window, cx| {
                this.handle_item_mouse_down(index, event, window, cx);
            }),
        )
        .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_item_mouse_up))
        .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_item_mouse_up))
        .on_click(cx.listener(move |this, event, _window, cx| {
            this.handle_item_click(index, event, cx);
        }));

        if row_oversize_extent > 0.0 {
            div().relative().w_full().p(px(row_oversize_extent)).child(row).into_any_element()
        } else {
            row.into_any_element()
        }
    }

    fn set_active_index_internal(
        &mut self,
        active_index: Option<usize>,
        scroll_direction: Option<ListDirection>,
        cx: &mut Context<Self>,
    ) -> bool {
        let next = match active_index {
            Some(index) if index < self.model.items.len() => Some(index),
            _ => normalize_active_index(
                active_index,
                self.model.items.as_slice(),
                self.model.row_enabled.as_ref(),
                &self.model.selected_indices,
            ),
        };
        if self.model.active_index == next {
            return false;
        }

        self.model.active_index = next;
        if let Some(index) = next {
            self.ensure_page_for_index(index, cx);
            if self.is_paged() {
                // All rows on the current page are visible without scrolling.
            } else {
                self.scroll_active_into_view(index, scroll_direction, cx);
            }
        }
        cx.emit(ListViewEvent::ActiveIndexChanged { active_index: next });
        cx.notify();
        true
    }

    fn commit_select_index(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        if !self.can_use_item(index) {
            return false;
        }

        let next = compute_next_selected_indices(self.model.selection_mode, &self.model.selected_indices, index);
        if next == self.model.selected_indices {
            return false;
        }

        self.model.selected_indices = next.clone();
        self.model.active_index =
            normalize_active_index(Some(index), self.model.items.as_slice(), self.model.row_enabled.as_ref(), &next);
        cx.emit(ListViewEvent::SelectionChanged { selected_indices: next });
        cx.notify();
        true
    }

    fn can_use_item(&self, index: usize) -> bool {
        self.model.enabled && self.model.items.get(index).is_some_and(|item| self.model.row_is_enabled(item))
    }

    fn clear_pointer_interaction(&mut self, cx: &mut Context<Self>) {
        if self.hovered_index.is_none() && self.pressed_index.is_none() {
            return;
        }

        self.clear_hovered_index(cx);
        self.pressed_index = None;
        cx.notify();
    }

    fn clear_hovered_index(&mut self, cx: &mut Context<Self>) {
        if let Some(index) = self.hovered_index.take() {
            cx.emit(ListViewEvent::RowHoverChanged { index, hovered: false });
        }
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.can_use_item(index) {
            return;
        }

        if hovered {
            if self.hovered_index != Some(index) {
                self.clear_hovered_index(cx);
                self.hovered_index = Some(index);
                cx.emit(ListViewEvent::RowHoverChanged { index, hovered: true });
                cx.notify();
            }
        } else if self.hovered_index == Some(index) {
            self.clear_hovered_index(cx);
            if self.pressed_index == Some(index) {
                self.pressed_index = None;
            }
            cx.notify();
        }
    }

    fn handle_item_mouse_down(
        &mut self,
        index: usize,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.can_use_item(index) {
            self.pressed_index = Some(index);
            self.set_active_index_internal(Some(index), None, cx);
            self.focus_handle.focus(window, cx);
            cx.notify();
        }
    }

    fn handle_item_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.pressed_index.is_some() {
            self.pressed_index = None;
            cx.notify();
        }
    }

    fn handle_item_click(&mut self, index: usize, event: &ClickEvent, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        self.set_active_index_internal(Some(index), None, cx);
        if self.model.select_on_row_click {
            self.commit_select_index(index, cx);
        }
    }

    fn move_active(&mut self, direction: ListDirection, cx: &mut Context<Self>) {
        self.clear_pointer_interaction(cx);
        if let Some(next_index) = next_enabled_index(
            self.model.items.as_slice(),
            self.model.row_enabled.as_ref(),
            self.model.active_index,
            direction,
        ) {
            self.set_active_index_internal(Some(next_index), Some(direction), cx);
        }
    }

    fn move_active_to_boundary(&mut self, first: bool, cx: &mut Context<Self>) {
        self.clear_pointer_interaction(cx);
        let len = self.model.items.len();
        if len == 0 {
            return;
        }

        let next_index = if first { 0 } else { len - 1 };
        self.set_active_index_internal(Some(next_index), None, cx);
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(ListDirection::Previous, cx);
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(ListDirection::Next, cx);
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active_to_boundary(true, cx);
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active_to_boundary(false, cx);
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_pointer_interaction(cx);
        if self.model.select_on_row_click
            && let Some(index) = self.model.active_index
        {
            self.commit_select_index(index, cx);
        }
    }

    fn page_scroll_distance(&self) -> gpui::Pixels {
        let viewport_height = self.list_state.viewport_bounds().size.height;
        if viewport_height > px(0.0) {
            viewport_height * 0.9
        } else {
            px(self.row_scroll_increment(&self.resolve_row_look_for_metrics(1.0)))
        }
    }

    fn handle_decrease_value_large(&mut self, _: &DecreaseValueLarge, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.clear_pointer_interaction(cx);

        if self.is_paged() {
            self.prev_page(cx);
        } else {
            self.list_state.scroll_by(-self.page_scroll_distance());
            if self.is_scroll_snap() {
                self.snap_scroll_position();
            }
            self.emit_scroll_changed_if_needed(cx);
            cx.notify();
        }
    }

    fn handle_increase_value_large(&mut self, _: &IncreaseValueLarge, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.clear_pointer_interaction(cx);

        if self.is_paged() {
            self.next_page(cx);
        } else {
            self.list_state.scroll_by(self.page_scroll_distance());
            if self.is_scroll_snap() {
                self.snap_scroll_position();
            }
            self.emit_scroll_changed_if_needed(cx);
            cx.notify();
        }
    }

    fn handle_scroll_wheel(&mut self, event: &ScrollWheelEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_paged() || !self.model.enabled {
            return;
        }

        let delta_y = event.delta.pixel_delta(px(20.0)).y;
        let is_scroll_end = matches!(event.touch_phase, TouchPhase::Ended);
        if delta_y == px(0.0) && !is_scroll_end {
            return;
        }

        cx.stop_propagation();

        if !self.is_scroll_snap() {
            if self.emit_scroll_changed_if_needed(cx) {
                cx.notify();
            }
            return;
        }

        if event.delta.precise() {
            // Trackpad: let GPUI's list scroll smoothly during the gesture, then snap once
            // when the gesture ends. Snapping on every delta fought partial scroll progress
            // and felt erratic (especially with natural scrolling).
            if is_scroll_end {
                self.snap_scroll_position();
                self.emit_scroll_changed_if_needed(cx);
                cx.notify();
            }
            return;
        }

        // Mouse wheel (line deltas): GPUI scrolls first in bubble order; align to the nearest row.
        self.snap_scroll_position();
        self.emit_scroll_changed_if_needed(cx);
        cx.notify();
    }

    fn handle_focus_in(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled && self.emit_focus_changed(true, cx) {
            cx.notify();
        }
    }

    fn handle_focus_out(&mut self, _: FocusOutEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.emit_focus_changed(false, cx) {
            cx.notify();
        }
    }

    fn emit_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) -> bool {
        if self.emitted_focused == focused {
            return false;
        }

        self.emitted_focused = focused;
        cx.emit(ListViewEvent::FocusChanged { focused });
        true
    }

    fn emit_scroll_changed_if_needed(&mut self, cx: &mut Context<Self>) -> bool {
        let top_index = self.list_state.logical_scroll_top().item_ix;
        if self.emitted_scroll_top_index == top_index {
            return false;
        }

        self.emitted_scroll_top_index = top_index;
        cx.emit(ListViewEvent::ScrollChanged { top_index });
        true
    }
}

impl<T> Focusable for ListViewControl<T>
where
    T: 'static,
{
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T> Render for ListViewControl<T>
where
    T: 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_in_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.focus_in_subscription = Some(cx.on_focus(&focus_handle, window, Self::handle_focus_in));
        }
        if self.focus_out_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.focus_out_subscription = Some(cx.on_focus_out(&focus_handle, window, Self::handle_focus_out));
        }

        let render_model = self.render_model(window, cx);
        let header = self
            .model
            .header_template
            .as_ref()
            .map(|header_template| header_template(&render_model, window, cx));
        let mut list_element = list(self.list_state.clone(), cx.processor(Self::render_row));
        if let Some(body_rows_height) = render_model.body_rows_height {
            list_element = list_element.h(px(body_rows_height)).w_full();
        } else {
            list_element = list_element.size_full();
        }
        let body = list_element.into_any_element();

        let mut shell = self
            .model
            .template
            .render(&render_model, header, body, window, cx)
            .track_focus(&self.focus_handle)
            .key_context(ControlKeyProfile::Selector.context())
            .on_action(cx.listener(Self::handle_select_previous_item))
            .on_action(cx.listener(Self::handle_select_next_item))
            .on_action(cx.listener(Self::handle_decrease_value_large))
            .on_action(cx.listener(Self::handle_increase_value_large))
            .on_action(cx.listener(Self::handle_select_first_item))
            .on_action(cx.listener(Self::handle_select_last_item))
            .on_action(cx.listener(Self::handle_activate_control));

        if !self.is_paged() {
            shell = shell.on_scroll_wheel(cx.listener(Self::handle_scroll_wheel));
        }

        let sync_page_size = self.should_sync_page_size_to_viewport();
        let entity = cx.entity();
        let mut root = div().w_full();
        if render_model.shell_height.is_none() {
            root = root.h_full();
        }
        if sync_page_size {
            root = root.on_children_prepainted(move |_child_bounds, window, cx| {
                entity.update(cx, |this, cx| {
                    this.sync_fill_height_page_size(window, cx);
                });
            });
        }
        root.child(shell).into_any_element()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ListDirection {
    Previous,
    Next,
}

pub(crate) fn normalize_model<T>(model: &mut ListViewModel<T>)
where
    T: 'static,
{
    model.selected_indices = normalize_selected_indices(
        model.selection_mode,
        model.items.as_slice(),
        model.row_enabled.as_ref(),
        &model.selected_indices,
    );
    model.active_index = normalize_active_index(
        model.active_index,
        model.items.as_slice(),
        model.row_enabled.as_ref(),
        &model.selected_indices,
    );
}

pub(crate) fn normalize_selected_indices<T>(
    selection_mode: ListSelectionMode,
    items: &[T],
    row_enabled: &dyn Fn(&T) -> bool,
    selected_indices: &[usize],
) -> Vec<usize> {
    let mut normalized = Vec::new();

    if selection_mode == ListSelectionMode::None {
        return normalized;
    }

    for index in selected_indices.iter().copied() {
        if index >= items.len() || !row_enabled(&items[index]) || normalized.contains(&index) {
            continue;
        }

        normalized.push(index);
        if selection_mode == ListSelectionMode::Single {
            break;
        }
    }

    normalized
}

pub(crate) fn normalize_active_index<T>(
    active_index: Option<usize>,
    items: &[T],
    row_enabled: &dyn Fn(&T) -> bool,
    selected_indices: &[usize],
) -> Option<usize> {
    if let Some(active_index) = active_index
        && active_index < items.len()
        && row_enabled(&items[active_index])
    {
        return Some(active_index);
    }

    if let Some(selected_index) =
        selected_indices.iter().copied().find(|index| *index < items.len() && row_enabled(&items[*index]))
    {
        return Some(selected_index);
    }

    items.iter().position(row_enabled)
}

fn first_enabled_index<T>(items: &[T], row_enabled: &dyn Fn(&T) -> bool) -> Option<usize> {
    items.iter().position(row_enabled)
}

fn last_enabled_index<T>(items: &[T], row_enabled: &dyn Fn(&T) -> bool) -> Option<usize> {
    items.iter().rposition(row_enabled)
}

fn next_enabled_index<T>(
    items: &[T],
    row_enabled: &dyn Fn(&T) -> bool,
    current_index: Option<usize>,
    direction: ListDirection,
) -> Option<usize> {
    let len = items.len();
    if len == 0 {
        return None;
    }

    match current_index {
        None => match direction {
            ListDirection::Next => first_enabled_index(items, row_enabled),
            ListDirection::Previous => last_enabled_index(items, row_enabled),
        },
        Some(start) => match direction {
            ListDirection::Next => (start + 1..len)
                .find(|&index| row_enabled(&items[index]))
                .or_else(|| (start + 1 < len).then(|| start + 1)),
            ListDirection::Previous => {
                (0..start).rfind(|&index| row_enabled(&items[index])).or_else(|| (start > 0).then(|| start - 1))
            }
        },
    }
}

fn compute_next_selected_indices(
    selection_mode: ListSelectionMode,
    current: &[usize],
    toggled_index: usize,
) -> Vec<usize> {
    match selection_mode {
        ListSelectionMode::None => Vec::new(),
        ListSelectionMode::Single => vec![toggled_index],
        ListSelectionMode::Multiple => {
            let mut next = current.to_vec();
            if let Some(position) = next.iter().position(|index| *index == toggled_index) {
                next.remove(position);
            } else {
                next.push(toggled_index);
                next.sort_unstable();
            }
            next
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui::SharedString;

    use super::{
        ListDirection, ListSelectionMode, compute_next_selected_indices, next_enabled_index, normalize_active_index,
        normalize_selected_indices,
    };
    use crate::controls::list_view::ListViewLabel;
    use crate::controls::list_view::layout::{clamp_page, page_count, visible_item_count};

    fn row_enabled(row: &ListViewLabel) -> bool {
        row.enabled
    }

    #[test]
    fn none_selection_mode_clears_selection() {
        let items = demo_items();
        assert_eq!(
            normalize_selected_indices(ListSelectionMode::None, &items, &row_enabled, &[0, 1, 2]),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn single_selection_mode_keeps_first_enabled_item() {
        let items =
            vec![ListViewLabel::new("one"), ListViewLabel::new("two").enabled(false), ListViewLabel::new("three")];

        assert_eq!(normalize_selected_indices(ListSelectionMode::Single, &items, &row_enabled, &[1, 2, 0]), vec![2]);
    }

    #[test]
    fn active_index_falls_back_to_first_selected_then_first_enabled() {
        let items =
            vec![ListViewLabel::new("one").enabled(false), ListViewLabel::new("two"), ListViewLabel::new("three")];

        assert_eq!(normalize_active_index(Some(0), &items, &row_enabled, &[2]), Some(2));
        assert_eq!(normalize_active_index(None, &items, &row_enabled, &[]), Some(1));
    }

    #[test]
    fn multiple_selection_toggles_index() {
        assert_eq!(compute_next_selected_indices(ListSelectionMode::Multiple, &[1, 3], 2), vec![1, 2, 3]);
        assert_eq!(compute_next_selected_indices(ListSelectionMode::Multiple, &[1, 2, 3], 2), vec![1, 3]);
    }

    #[test]
    fn paging_helpers() {
        assert_eq!(page_count(25, 10), 3);
        assert_eq!(visible_item_count(25, 2, 10), 5);
        assert_eq!(clamp_page(9, 25, 10), 2);
    }

    #[test]
    fn next_enabled_index_does_not_wrap() {
        let items = demo_items();

        assert_eq!(next_enabled_index(&items, &row_enabled, Some(0), ListDirection::Previous), None);
        assert_eq!(next_enabled_index(&items, &row_enabled, Some(2), ListDirection::Next), None);
        assert_eq!(next_enabled_index(&items, &row_enabled, Some(0), ListDirection::Next), Some(1));
        assert_eq!(next_enabled_index(&items, &row_enabled, Some(2), ListDirection::Previous), Some(1));
    }

    #[test]
    fn next_enabled_index_steps_onto_adjacent_disabled_row_at_boundary() {
        let items =
            vec![ListViewLabel::new("zero").enabled(false), ListViewLabel::new("one"), ListViewLabel::new("two")];

        assert_eq!(next_enabled_index(&items, &row_enabled, Some(1), ListDirection::Previous), Some(0));
        assert_eq!(next_enabled_index(&items, &row_enabled, Some(0), ListDirection::Next), Some(1));
        assert_eq!(next_enabled_index(&items, &row_enabled, Some(0), ListDirection::Previous), None);
    }

    fn demo_items() -> Vec<ListViewLabel> {
        vec![
            ListViewLabel::new(SharedString::from("one")),
            ListViewLabel::new(SharedString::from("two")),
            ListViewLabel::new(SharedString::from("three")),
        ]
    }

    #[test]
    fn row_label_round_trips() {
        let row = ListViewLabel::new("alpha");
        assert_eq!(row.label, SharedString::from("alpha"));
    }
}
