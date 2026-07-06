use gpui::{
    App, AppContext, ClickEvent, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, MouseDownEvent,
    MouseUpEvent, Render, ScrollWheelEvent, SharedString, Subscription, Window, div, prelude::*, px,
};

use crate::controls::popup_scroll_surface::PopupScrollSurface;
use crate::controls::scrollbar::{ScrollbarEvent, ScrollbarTemplate};
use crate::controls::selection_panel::item_template::{SelectionPanelItemTemplate, make_selection_panel_item_template};
use crate::controls::selection_panel::model::{
    SelectionPanelBuilder, SelectionPanelLookProvider, SelectionPanelItem, SelectionPanelItemLike, SelectionPanelModel,
    default_selection_panel_model,
};
use crate::controls::selection_panel::template::SelectionPanelTemplate;
use crate::controls::selection_panel::template::{
    SelectionPanelClickHandler, SelectionPanelHoverHandler, SelectionPanelMouseDownHandler,
    SelectionPanelMouseUpHandler, SelectionPanelRenderModel, render_selection_panel,
};
use crate::controls::state::ControlFocusState;
use crate::keyhandling::{
    ActivateControl, ControlKeyProfile, DecreaseValueLarge, IncreaseValueLarge, SelectFirstItem, SelectLastItem,
    SelectNextItem, SelectPreviousItem,
};
use crate::theme::ControlSize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectionPanelEvent {
    HoverChanged { visible_index: Option<usize> },
    ActivateRow { source_index: usize, visible_index: usize, item_id: SharedString },
    ActiveIndexChanged { visible_index: Option<usize> },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectionPanelState {
    active_visible_index: Option<usize>,
    hovered_visible_index: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionPanelStepDirection {
    Previous,
    Next,
}

pub struct SelectionPanelControl<T = SelectionPanelItem>
where
    T: SelectionPanelItemLike + 'static,
{
    model: SelectionPanelModel<T>,
    state: SelectionPanelState,
    pressed_visible_index: Option<usize>,
    focus_handle: FocusHandle,
    popup_surface: PopupScrollSurface,
    _subscriptions: Vec<Subscription>,
}

impl<T> EventEmitter<SelectionPanelEvent> for SelectionPanelControl<T> where T: SelectionPanelItemLike + 'static {}

impl SelectionPanelControl<SelectionPanelItem> {
    pub fn new(id: impl Into<SharedString>, cx: &mut impl AppContext) -> Entity<Self> {
        Self::new_typed(id, cx)
    }
}

impl<T> SelectionPanelControl<T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub fn new_typed(id: impl Into<SharedString>, cx: &mut impl AppContext) -> Entity<Self> {
        let model = default_selection_panel_model::<T>(id);
        cx.new(|cx| Self::from_model(model, None, cx))
    }

    pub(crate) fn from_builder(builder: SelectionPanelBuilder<T>, cx: &mut Context<Self>) -> Self {
        Self::from_model(builder.model, builder.initial_active_visible_index, cx)
    }

    pub(crate) fn from_model(
        mut model: SelectionPanelModel<T>,
        initial_active_visible_index: Option<usize>,
        cx: &mut Context<Self>,
    ) -> Self {
        if model.visible_indices.is_empty() {
            model.visible_indices = (0..model.items.len()).collect();
        }

        let mut popup_surface =
            PopupScrollSurface::new(format!("{}-scroll-surface", model.id), model.scrollbar_template.clone(), cx);
        popup_surface.set_scrolling_enabled(model.scrolling);
        popup_surface.set_snap_to_rows(true);

        let subscriptions =
            vec![cx.subscribe(&popup_surface.scrollbar(), |this, _, event: &ScrollbarEvent, cx| match event {
                ScrollbarEvent::Change { value } => {
                    this.popup_surface.set_vertical_offset(*value, cx);
                    cx.notify();
                }
            })];

        Self {
            focus_handle: cx.focus_handle().tab_stop(model.enabled),
            model,
            state: SelectionPanelState {
                active_visible_index: initial_active_visible_index,
                hovered_visible_index: None,
            },
            pressed_visible_index: None,
            popup_surface,
            _subscriptions: subscriptions,
        }
    }

    pub fn set_panel_id(&mut self, panel_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.panel_id = panel_id.into();
        cx.notify();
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = T>, cx: &mut Context<Self>) {
        self.model.items = items.into_iter().collect();
        self.model.visible_indices = (0..self.model.items.len()).collect();
        self.clamp_state();
        cx.notify();
    }

    pub fn set_visible_indices(&mut self, visible_indices: impl IntoIterator<Item = usize>, cx: &mut Context<Self>) {
        self.model.visible_indices = visible_indices.into_iter().collect();
        self.clamp_state();
        cx.notify();
    }

    pub fn set_selected_source_index(&mut self, selected_source_index: Option<usize>, cx: &mut Context<Self>) {
        if self.model.selected_source_index == selected_source_index {
            return;
        }

        self.model.selected_source_index = selected_source_index;
        cx.notify();
    }

    pub fn set_active_visible_index(&mut self, visible_index: Option<usize>, cx: &mut Context<Self>) {
        if self.state.set_active_visible_index(self.normalize_visible_index(visible_index)) {
            if let Some(active_visible_index) = self.state.active_visible_index() {
                self.popup_surface.ensure_item_visible(active_visible_index, cx);
            }
            cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
            cx.notify();
        }
    }

    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.model.open == open {
            return;
        }
        self.model.open = open;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        if !enabled {
            self.state.set_hovered_visible_index(None);
            self.pressed_visible_index = None;
        }
        cx.notify();
    }

    pub fn set_show_selection_marker(&mut self, show_selection_marker: bool, cx: &mut Context<Self>) {
        if self.model.show_selection_marker == show_selection_marker {
            return;
        }
        self.model.show_selection_marker = show_selection_marker;
        cx.notify();
    }

    pub fn set_scrolling(&mut self, scrolling: bool, cx: &mut Context<Self>) {
        if self.model.scrolling == scrolling {
            return;
        }
        self.model.scrolling = scrolling;
        self.popup_surface.set_scrolling_enabled(scrolling);
        cx.notify();
    }

    pub fn set_visible_row_limits(&mut self, min_visible_rows: usize, max_visible_rows: usize, cx: &mut Context<Self>) {
        let min_visible_rows = min_visible_rows.max(1);
        let max_visible_rows = max_visible_rows.max(min_visible_rows);

        if self.model.min_visible_rows == min_visible_rows && self.model.max_visible_rows == max_visible_rows {
            return;
        }

        self.model.min_visible_rows = min_visible_rows;
        self.model.max_visible_rows = max_visible_rows;
        cx.notify();
    }

    pub fn set_size(&mut self, size: ControlSize, cx: &mut Context<Self>) {
        if self.model.size == size {
            return;
        }
        self.model.size = size;
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn SelectionPanelTemplate<T>>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_item_template_fn<F, E>(&mut self, template: F, cx: &mut Context<Self>)
    where
        F: for<'a> Fn(&crate::controls::selection_panel::SelectionPanelItemRenderModel<'a, T>, &mut App) -> E
            + Send
            + Sync
            + 'static,
        E: IntoElement + 'static,
    {
        self.set_item_template(Some(make_selection_panel_item_template(template)), cx);
    }

    pub fn set_item_template(&mut self, item_template: Option<SelectionPanelItemTemplate<T>>, cx: &mut Context<Self>) {
        self.model.item_template = item_template;
        cx.notify();
    }

    pub fn clear_item_template(&mut self, cx: &mut Context<Self>) {
        self.set_item_template(None, cx);
    }

    pub fn set_scrollbar_template(&mut self, template: std::sync::Arc<dyn ScrollbarTemplate>, cx: &mut Context<Self>) {
        self.model.scrollbar_template = template;
        cx.notify();
    }

    pub fn set_look_provider(&mut self, provider: SelectionPanelLookProvider, cx: &mut Context<Self>) {
        self.model.look_provider = provider;
        cx.notify();
    }

    pub fn ensure_visible(&self, visible_index: usize, cx: &mut Context<Self>) {
        if self.model.visible_indices.is_empty() {
            return;
        }

        self.popup_surface
            .ensure_item_visible(visible_index.min(self.model.visible_indices.len().saturating_sub(1)), cx);
    }

    fn normalize_visible_index(&self, visible_index: Option<usize>) -> Option<usize> {
        visible_index.filter(|index| *index < self.model.visible_indices.len())
    }

    fn clamp_state(&mut self) {
        self.state.active_visible_index = self.normalize_visible_index(self.state.active_visible_index);
        self.state.hovered_visible_index = self.normalize_visible_index(self.state.hovered_visible_index);
        self.pressed_visible_index = self.normalize_visible_index(self.pressed_visible_index);

        if self.model.selected_source_index.is_some_and(|source_index| source_index >= self.model.items.len()) {
            self.model.selected_source_index = None;
        }
    }

    fn can_use_visible_index(&self, visible_index: usize) -> bool {
        let Some(source_index) = self.model.visible_indices.get(visible_index).copied() else {
            return false;
        };

        self.model.enabled && self.model.items.get(source_index).is_some_and(SelectionPanelItemLike::is_enabled)
    }

    fn emit_activate_for_visible_index(&mut self, visible_index: usize, cx: &mut Context<Self>) {
        let Some(source_index) = self.model.visible_indices.get(visible_index).copied() else {
            return;
        };
        let Some(item) = self.model.items.get(source_index) else {
            return;
        };

        if !self.model.enabled || !item.is_enabled() {
            return;
        }

        self.model.selected_source_index = Some(source_index);
        if self.state.set_active_visible_index(Some(visible_index)) {
            cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
        }

        self.popup_surface.ensure_item_visible(visible_index, cx);

        cx.emit(SelectionPanelEvent::ActivateRow { source_index, visible_index, item_id: item.id().clone() });
        cx.notify();
    }

    fn move_active(&mut self, direction: SelectionPanelStepDirection, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if self.state.step(self.model.visible_indices.len(), direction) {
            if let Some(active_visible_index) = self.state.active_visible_index() {
                self.popup_surface.ensure_item_visible(active_visible_index, cx);
            }
            cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
            cx.notify();
        }
    }

    fn move_active_to_boundary(&mut self, first: bool, cx: &mut Context<Self>) {
        if !self.model.enabled || self.model.visible_indices.is_empty() {
            return;
        }

        let next = if first {
            Some(0)
        } else {
            Some(self.model.visible_indices.len() - 1)
        };

        if self.state.set_active_visible_index(next) {
            if let Some(active_visible_index) = self.state.active_visible_index() {
                self.popup_surface.ensure_item_visible(active_visible_index, cx);
            }
            cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
            cx.notify();
        }
    }

    fn handle_item_hover(&mut self, visible_index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.can_use_visible_index(visible_index) {
            return;
        }

        let next = if hovered { Some(visible_index) } else { None };

        if self.state.set_hovered_visible_index(next) {
            if hovered {
                self.state.set_active_visible_index(Some(visible_index));
                self.popup_surface.ensure_item_visible(visible_index, cx);
                cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
            }

            cx.emit(SelectionPanelEvent::HoverChanged { visible_index: self.state.hovered_visible_index() });
            cx.notify();
        }
    }

    fn handle_item_mouse_down(
        &mut self,
        visible_index: usize,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_use_visible_index(visible_index) {
            return;
        }

        self.pressed_visible_index = Some(visible_index);
        self.state.set_active_visible_index(Some(visible_index));
        self.focus_handle.focus(window, cx);
        cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
        cx.notify();
    }

    fn handle_item_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.pressed_visible_index.take().is_some() {
            cx.notify();
        }
    }

    fn handle_item_click(
        &mut self,
        visible_index: usize,
        event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.is_keyboard() {
            return;
        }

        self.emit_activate_for_visible_index(visible_index, cx);
    }

    fn handle_scroll_wheel(&mut self, event: &ScrollWheelEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.popup_surface.scroll_wheel(event, cx) {
            cx.notify();
        }

        let delta_y = event.delta.pixel_delta(px(20.0)).y.as_f32();
        if delta_y.is_finite() && delta_y.abs() > f32::EPSILON {
            cx.stop_propagation();
        }
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(SelectionPanelStepDirection::Previous, cx);
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(SelectionPanelStepDirection::Next, cx);
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active_to_boundary(true, cx);
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active_to_boundary(false, cx);
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(active_visible_index) = self.state.active_visible_index() {
            self.emit_activate_for_visible_index(active_visible_index, cx);
        }
    }

    fn handle_page_down(&mut self, _: &IncreaseValueLarge, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || self.model.visible_indices.is_empty() {
            return;
        }

        let page = self.popup_surface.visible_row_count().max(1);
        let current = self.state.active_visible_index().unwrap_or(0);
        let next = (current + page).min(self.model.visible_indices.len() - 1);

        if self.state.set_active_visible_index(Some(next)) {
            self.popup_surface.ensure_item_visible(next, cx);
            cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
            cx.notify();
        }
    }

    fn handle_page_up(&mut self, _: &DecreaseValueLarge, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || self.model.visible_indices.is_empty() {
            return;
        }

        let page = self.popup_surface.visible_row_count().max(1);
        let current = self.state.active_visible_index().unwrap_or(0);
        let next = current.saturating_sub(page);

        if self.state.set_active_visible_index(Some(next)) {
            self.popup_surface.ensure_item_visible(next, cx);
            cx.emit(SelectionPanelEvent::ActiveIndexChanged { visible_index: self.state.active_visible_index() });
            cx.notify();
        }
    }

    #[allow(clippy::type_complexity)]
    fn template_handlers(
        &self,
        cx: &mut Context<Self>,
    ) -> (
        Vec<SelectionPanelHoverHandler>,
        Vec<SelectionPanelMouseDownHandler>,
        Vec<SelectionPanelMouseUpHandler>,
        Vec<SelectionPanelMouseUpHandler>,
        Vec<SelectionPanelClickHandler>,
    ) {
        let hover_handlers = (0..self.model.visible_indices.len())
            .map(|visible_index| {
                Box::new(cx.listener(move |this, hovered, _window, cx| {
                    this.handle_item_hover(visible_index, *hovered, cx);
                })) as SelectionPanelHoverHandler
            })
            .collect::<Vec<_>>();

        let mouse_down_handlers = (0..self.model.visible_indices.len())
            .map(|visible_index| {
                Box::new(cx.listener(move |this, event, window, cx| {
                    this.handle_item_mouse_down(visible_index, event, window, cx);
                })) as SelectionPanelMouseDownHandler
            })
            .collect::<Vec<_>>();

        let mouse_up_handlers = (0..self.model.visible_indices.len())
            .map(|_| Box::new(cx.listener(Self::handle_item_mouse_up)) as SelectionPanelMouseUpHandler)
            .collect::<Vec<_>>();

        let mouse_up_out_handlers = (0..self.model.visible_indices.len())
            .map(|_| Box::new(cx.listener(Self::handle_item_mouse_up)) as SelectionPanelMouseUpHandler)
            .collect::<Vec<_>>();

        let click_handlers = (0..self.model.visible_indices.len())
            .map(|visible_index| {
                Box::new(cx.listener(move |this, event, window, cx| {
                    this.handle_item_click(visible_index, event, window, cx);
                })) as SelectionPanelClickHandler
            })
            .collect::<Vec<_>>();

        (hover_handlers, mouse_down_handlers, mouse_up_handlers, mouse_up_out_handlers, click_handlers)
    }

    pub fn scrollbar(&self) -> Entity<crate::controls::scrollbar::Scrollbar> {
        self.popup_surface.scrollbar()
    }
}

impl SelectionPanelState {
    pub fn active_visible_index(&self) -> Option<usize> {
        self.active_visible_index
    }

    pub fn hovered_visible_index(&self) -> Option<usize> {
        self.hovered_visible_index
    }

    pub fn set_hovered_visible_index(&mut self, hovered_visible_index: Option<usize>) -> bool {
        if self.hovered_visible_index == hovered_visible_index {
            return false;
        }
        self.hovered_visible_index = hovered_visible_index;
        true
    }

    pub fn set_active_visible_index(&mut self, active_visible_index: Option<usize>) -> bool {
        if self.active_visible_index == active_visible_index {
            return false;
        }
        self.active_visible_index = active_visible_index;
        true
    }

    pub fn step(&mut self, visible_len: usize, direction: SelectionPanelStepDirection) -> bool {
        if visible_len == 0 {
            return false;
        }

        let Some(current) = self.active_visible_index else {
            return self.set_active_visible_index(Some(0));
        };

        let next = match direction {
            SelectionPanelStepDirection::Previous => current.saturating_sub(1),
            SelectionPanelStepDirection::Next => (current + 1).min(visible_len - 1),
        };

        self.set_active_visible_index(Some(next))
    }
}

impl<T> Focusable for SelectionPanelControl<T>
where
    T: SelectionPanelItemLike + 'static,
{
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T> Render for SelectionPanelControl<T>
where
    T: SelectionPanelItemLike + 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.clamp_state();

        let look = (self.model.look_provider)(self.model.size);
        let panel_min_width = look.min_width;

        let item_count = self.model.visible_indices.len();
        let min_visible_rows = self.model.min_visible_rows.max(1);
        let max_visible_rows = self.model.max_visible_rows.max(min_visible_rows);
        let visible_rows = item_count.max(1).clamp(min_visible_rows, max_visible_rows) as f32;
        let viewport_height = px(look.item_height * visible_rows);
        let row_height = px(look.item_height);
        let content_top_padding = px(0.0);
        let allow_scrolling = self.model.scrolling && item_count > max_visible_rows;

        self.popup_surface.set_scrolling_enabled(allow_scrolling);
        self.popup_surface.configure(item_count.max(1), row_height, content_top_padding, viewport_height);
        self.popup_surface.sync(cx);

        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let (item_hovers, item_mouse_downs, item_mouse_ups, item_mouse_up_outs, item_clicks) =
            self.template_handlers(cx);

        let rows = render_selection_panel(
            self.model.template.clone(),
            &SelectionPanelRenderModel {
                panel_id: &self.model.panel_id,
                control_id: &self.model.id,
                items: &self.model.items,
                visible_indices: &self.model.visible_indices,
                selected_source_index: self.model.selected_source_index,
                active_visible_index: self.state.active_visible_index,
                hovered_visible_index: self.state.hovered_visible_index,
                pressed_visible_index: self.pressed_visible_index,
                open: self.model.open,
                enabled: self.model.enabled,
                focus,
                item_template: self.model.item_template.as_ref(),
                look: look.clone(),
                show_selection_marker: self.model.show_selection_marker,
                show_panel_chrome: false,
            },
            item_hovers,
            item_mouse_downs,
            item_mouse_ups,
            item_mouse_up_outs,
            item_clicks,
            cx,
        );

        let rows_content = if allow_scrolling {
            self.popup_surface.render(rows.into_any_element())
        } else {
            rows.into_any_element()
        };

        let content_width = panel_min_width + if allow_scrolling { 16.0 } else { 0.0 };

        let panel_shell = div()
            .min_w(px(content_width))
            .p(px(look.padding))
            .bg(look.background)
            .border_1()
            .border_color(look.border)
            .rounded(px(look.radius))
            .shadow(look.shadow.clone())
            .occlude()
            .child(rows_content);

        div()
            .id(self.model.id.clone())
            .track_focus(&self.focus_handle)
            .key_context(ControlKeyProfile::Selector.context())
            .on_scroll_wheel(cx.listener(Self::handle_scroll_wheel))
            .on_action(cx.listener(Self::handle_select_previous_item))
            .on_action(cx.listener(Self::handle_select_next_item))
            .on_action(cx.listener(Self::handle_select_first_item))
            .on_action(cx.listener(Self::handle_select_last_item))
            .on_action(cx.listener(Self::handle_page_up))
            .on_action(cx.listener(Self::handle_page_down))
            .on_action(cx.listener(Self::handle_activate_control))
            .child(panel_shell)
    }
}
