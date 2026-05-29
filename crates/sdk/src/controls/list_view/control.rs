use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, ListState, MouseButton, MouseDownEvent,
    MouseUpEvent, Render, SharedString, Window, div, list, prelude::*, px,
};

use super::model::{
    ListSelectionMode, ListViewBuilder, ListViewHeaderTemplate, ListViewItemLike, ListViewItemRenderModel,
    ListViewModel, ListViewRenderModel, make_list_view_header_template, make_list_view_item_template,
};
use super::template::ListViewTemplate;
use super::theme::{ListViewRowAppearance, ListViewTheme};
use crate::controls::state::ControlFocusState;
use crate::keyhandling::{
    ActivateControl, ControlKeyProfile, DecreaseValueLarge, IncreaseValueLarge, SelectFirstItem, SelectLastItem,
    SelectNextItem, SelectPreviousItem,
};
use crate::theme::adorner::{adorner_oversize_extent, render_optional_adorner_with_focus_radius};
use crate::theme::{ControlSize, InteractionState};

#[derive(Clone, Debug)]
pub enum ListViewEvent {
    SelectionChanged { selected_indices: Vec<usize> },
    ActiveIndexChanged { active_index: Option<usize> },
}

pub struct ListViewControl<T>
where
    T: ListViewItemLike + 'static,
{
    model: ListViewModel<T>,
    list_state: ListState,
    focus_handle: gpui::FocusHandle,
    hovered_index: Option<usize>,
    pressed_index: Option<usize>,
}

impl<T> EventEmitter<ListViewEvent> for ListViewControl<T> where T: ListViewItemLike + 'static {}

impl<T> ListViewControl<T>
where
    T: ListViewItemLike + 'static,
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ListViewBuilder<T> {
        ListViewBuilder::new(id)
    }

    pub(crate) fn from_builder(mut builder: ListViewBuilder<T>, cx: &mut Context<Self>) -> Self {
        normalize_model(&mut builder.model);

        Self {
            list_state: ListState::new(builder.model.items.len(), builder.model.alignment, px(builder.model.overdraw)),
            focus_handle: cx.focus_handle().tab_stop(builder.model.enabled),
            model: builder.model,
            hovered_index: None,
            pressed_index: None,
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

    pub fn is_enabled(&self) -> bool {
        self.model.enabled
    }

    pub fn selected_items(&self) -> Vec<&T> {
        self.model.selected_indices.iter().filter_map(|index| self.model.items.get(*index)).collect()
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = T>, cx: &mut Context<Self>) {
        self.model.items = items.into_iter().collect();
        self.hovered_index = None;
        self.pressed_index = None;
        self.list_state.reset(self.model.items.len());
        normalize_model(&mut self.model);
        cx.notify();
    }

    pub fn set_selected_indices(&mut self, selected_indices: impl IntoIterator<Item = usize>, cx: &mut Context<Self>) {
        self.model.selected_indices = selected_indices.into_iter().collect();
        normalize_model(&mut self.model);
        cx.notify();
    }

    pub fn set_active_index(&mut self, active_index: Option<usize>, cx: &mut Context<Self>) {
        self.set_active_index_internal(active_index, cx);
    }

    pub fn set_selection_mode(&mut self, selection_mode: ListSelectionMode, cx: &mut Context<Self>) {
        if self.model.selection_mode == selection_mode {
            return;
        }

        self.model.selection_mode = selection_mode;
        normalize_model(&mut self.model);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);

        if !enabled {
            self.hovered_index = None;
            self.pressed_index = None;
        }

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

    pub fn set_header_template(&mut self, header_template: Option<ListViewHeaderTemplate>, cx: &mut Context<Self>) {
        self.model.header_template = header_template;
        cx.notify();
    }

    pub fn with_header_template<F, E>(&mut self, template: F, cx: &mut Context<Self>)
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

    pub fn with_item_template<F, E>(&mut self, template: F, cx: &mut Context<Self>)
    where
        F: for<'a> Fn(&ListViewItemRenderModel<'a, T>, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.item_template = Some(make_list_view_item_template(template));
        self.list_state.remeasure();
        cx.notify();
    }

    pub fn clear_item_template(&mut self, cx: &mut Context<Self>) {
        if self.model.item_template.is_some() {
            self.model.item_template = None;
            self.list_state.remeasure();
            cx.notify();
        }
    }

    fn render_model(&self, window: &Window) -> ListViewRenderModel<'_> {
        ListViewRenderModel {
            id: &self.model.id,
            item_count: self.model.items.len(),
            selection_mode: self.model.selection_mode,
            enabled: self.model.enabled,
            size: self.model.size,
            focus: ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window),
        }
    }

    fn render_row(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let Some(item) = self.model.items.get(index) else {
            return div().into_any_element();
        };

        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let enabled = self.model.enabled && item.is_enabled();
        let selected = self.model.selected_indices.contains(&index);
        let active = self.model.active_index == Some(index);
        let hovered = enabled && self.hovered_index == Some(index);
        let pressed = enabled && self.pressed_index == Some(index);
        let interaction =
            InteractionState { hovered, pressed, focused: enabled && focus.focused && active, disabled: !enabled };
        let appearance = self.model.theme.resolve_row(selected, interaction, self.model.size);
        let focused_probe_appearance = if enabled {
            let mut focused_probe_state = interaction;
            focused_probe_state.focused = true;
            Some(self.model.theme.resolve_row(selected, focused_probe_state, self.model.size))
        } else {
            None
        };
        let row_oversize_extent = adorner_oversize_extent(appearance.adorner)
            .max(focused_probe_appearance.as_ref().map(|probe| adorner_oversize_extent(probe.adorner)).unwrap_or(0.0));

        let content = if let Some(item_template) = self.model.item_template.as_ref() {
            item_template(
                &ListViewItemRenderModel {
                    list_id: &self.model.id,
                    item,
                    index,
                    sibling_count: self.model.items.len(),
                    selected,
                    active,
                    hovered,
                    pressed,
                    focused: focus.focused,
                    focus_visible: focus.focus_visible,
                    enabled,
                },
                window,
                cx,
            )
        } else {
            div().flex_1().min_w(px(0.0)).truncate().child(item.label().clone()).into_any_element()
        };

        let row = render_list_view_row_visual(format!("{}-row-{}", self.model.id, index), content, appearance, enabled)
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

    fn set_active_index_internal(&mut self, active_index: Option<usize>, cx: &mut Context<Self>) -> bool {
        let next = normalize_active_index(active_index, self.model.items.as_slice(), &self.model.selected_indices);
        if self.model.active_index == next {
            return false;
        }

        self.model.active_index = next;
        if let Some(index) = next {
            self.list_state.scroll_to_reveal_item(index);
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
        self.model.active_index = normalize_active_index(Some(index), self.model.items.as_slice(), &next);
        cx.emit(ListViewEvent::SelectionChanged { selected_indices: next });
        cx.notify();
        true
    }

    fn can_use_item(&self, index: usize) -> bool {
        self.model.enabled && self.model.items.get(index).is_some_and(ListViewItemLike::is_enabled)
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.can_use_item(index) {
            return;
        }

        if hovered {
            if self.hovered_index != Some(index) {
                self.hovered_index = Some(index);
                cx.notify();
            }
        } else if self.hovered_index == Some(index) {
            self.hovered_index = None;
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
            self.set_active_index_internal(Some(index), cx);
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

        self.set_active_index_internal(Some(index), cx);
        self.commit_select_index(index, cx);
    }

    fn move_active(&mut self, direction: ListDirection, cx: &mut Context<Self>) {
        if let Some(next_index) = next_enabled_index(self.model.items.as_slice(), self.model.active_index, direction) {
            self.set_active_index_internal(Some(next_index), cx);
        }
    }

    fn move_active_to_boundary(&mut self, first: bool, cx: &mut Context<Self>) {
        let next_index = if first {
            first_enabled_index(self.model.items.as_slice())
        } else {
            last_enabled_index(self.model.items.as_slice())
        };

        self.set_active_index_internal(next_index, cx);
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
        if let Some(index) = self.model.active_index {
            self.commit_select_index(index, cx);
        }
    }

    fn page_scroll_distance(&self) -> gpui::Pixels {
        let viewport_height = self.list_state.viewport_bounds().size.height;
        if viewport_height > px(0.0) {
            viewport_height * 0.9
        } else {
            px(240.0)
        }
    }

    fn handle_decrease_value_large(&mut self, _: &DecreaseValueLarge, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.list_state.scroll_by(-self.page_scroll_distance());
        cx.notify();
    }

    fn handle_increase_value_large(&mut self, _: &IncreaseValueLarge, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.list_state.scroll_by(self.page_scroll_distance());
        cx.notify();
    }
}

impl<T> Focusable for ListViewControl<T>
where
    T: ListViewItemLike + 'static,
{
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T> Render for ListViewControl<T>
where
    T: ListViewItemLike + 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let render_model = self.render_model(window);
        let header = self
            .model
            .header_template
            .as_ref()
            .map(|header_template| header_template(&render_model, window, cx));
        let body = list(self.list_state.clone(), cx.processor(Self::render_row)).size_full().into_any_element();

        div()
            .w_full()
            .h_full()
            .child(
                self.model
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
                    .on_action(cx.listener(Self::handle_activate_control)),
            )
            .into_any_element()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ListDirection {
    Previous,
    Next,
}

pub(crate) fn normalize_model<T>(model: &mut ListViewModel<T>)
where
    T: ListViewItemLike + 'static,
{
    model.selected_indices =
        normalize_selected_indices(model.selection_mode, model.items.as_slice(), &model.selected_indices);
    model.active_index = normalize_active_index(model.active_index, model.items.as_slice(), &model.selected_indices);
}

pub(crate) fn normalize_selected_indices<T>(
    selection_mode: ListSelectionMode,
    items: &[T],
    selected_indices: &[usize],
) -> Vec<usize>
where
    T: ListViewItemLike + 'static,
{
    let mut normalized = Vec::new();

    if selection_mode == ListSelectionMode::None {
        return normalized;
    }

    for index in selected_indices.iter().copied() {
        if index >= items.len() || !items[index].is_enabled() || normalized.contains(&index) {
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
    selected_indices: &[usize],
) -> Option<usize>
where
    T: ListViewItemLike + 'static,
{
    if let Some(active_index) = active_index
        && active_index < items.len()
        && items[active_index].is_enabled()
    {
        return Some(active_index);
    }

    if let Some(selected_index) =
        selected_indices.iter().copied().find(|index| *index < items.len() && items[*index].is_enabled())
    {
        return Some(selected_index);
    }

    items.iter().position(ListViewItemLike::is_enabled)
}

fn first_enabled_index<T>(items: &[T]) -> Option<usize>
where
    T: ListViewItemLike + 'static,
{
    items.iter().position(ListViewItemLike::is_enabled)
}

fn last_enabled_index<T>(items: &[T]) -> Option<usize>
where
    T: ListViewItemLike + 'static,
{
    items.iter().rposition(ListViewItemLike::is_enabled)
}

fn next_enabled_index<T>(items: &[T], current_index: Option<usize>, direction: ListDirection) -> Option<usize>
where
    T: ListViewItemLike + 'static,
{
    let len = items.len();
    if len == 0 {
        return None;
    }

    let step = match direction {
        ListDirection::Previous => len - 1,
        ListDirection::Next => 1,
    };

    let mut index = match current_index {
        Some(index) => (index + step) % len,
        None if direction == ListDirection::Previous => len - 1,
        None => 0,
    };

    for _ in 0..len {
        if items[index].is_enabled() {
            return Some(index);
        }
        index = (index + step) % len;
    }

    None
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

fn render_list_view_row_visual(
    id: impl Into<gpui::ElementId>,
    content: gpui::AnyElement,
    appearance: ListViewRowAppearance,
    enabled: bool,
) -> gpui::Stateful<gpui::Div> {
    let mut row = div()
        .id(id)
        .relative()
        .w_full()
        .min_h(px(appearance.min_height))
        .flex()
        .items_center()
        .rounded(px(appearance.radius))
        .px(px(appearance.padding_x))
        .py(px(appearance.padding_y))
        .bg(appearance.background)
        .text_color(appearance.label_color)
        .text_size(px(appearance.label_typography.size))
        .line_height(px(appearance.label_typography.line_height))
        .font_weight(appearance.label_typography.weight)
        .child(div().flex_1().min_w(px(0.0)).child(content));

    if enabled {
        row = row.cursor_pointer();
    } else {
        row = row.opacity(0.56);
    }

    if let Some(adorner) = render_optional_adorner_with_focus_radius(appearance.adorner, appearance.radius) {
        row = row.child(adorner);
    }

    row
}

#[cfg(test)]
mod tests {
    use gpui::SharedString;

    use super::{ListSelectionMode, compute_next_selected_indices, normalize_active_index, normalize_selected_indices};
    use crate::controls::list_view::{ListViewItem, ListViewItemLike};

    #[test]
    fn none_selection_mode_clears_selection() {
        let items = demo_items();
        assert_eq!(normalize_selected_indices(ListSelectionMode::None, &items, &[0, 1, 2]), Vec::<usize>::new());
    }

    #[test]
    fn single_selection_mode_keeps_first_enabled_item() {
        let items = vec![ListViewItem::new("one"), ListViewItem::new("two").enabled(false), ListViewItem::new("three")];

        assert_eq!(normalize_selected_indices(ListSelectionMode::Single, &items, &[1, 2, 0]), vec![2]);
    }

    #[test]
    fn active_index_falls_back_to_first_selected_then_first_enabled() {
        let items = vec![ListViewItem::new("one").enabled(false), ListViewItem::new("two"), ListViewItem::new("three")];

        assert_eq!(normalize_active_index(Some(0), &items, &[2]), Some(2));
        assert_eq!(normalize_active_index(None, &items, &[]), Some(1));
    }

    #[test]
    fn multiple_selection_toggles_index() {
        assert_eq!(compute_next_selected_indices(ListSelectionMode::Multiple, &[1, 3], 2), vec![1, 2, 3]);
        assert_eq!(compute_next_selected_indices(ListSelectionMode::Multiple, &[1, 2, 3], 2), vec![1, 3]);
    }

    fn demo_items() -> Vec<ListViewItem> {
        vec![
            ListViewItem::new(SharedString::from("one")),
            ListViewItem::new(SharedString::from("two")),
            ListViewItem::new(SharedString::from("three")),
        ]
    }

    #[test]
    fn item_label_round_trips() {
        let item = ListViewItem::new("alpha");
        assert_eq!(ListViewItemLike::label(&item), &SharedString::from("alpha"));
    }
}
