use std::{collections::HashMap, sync::Arc, time::Duration};

use gpui::{
    Bounds, Context, CursorStyle, DragMoveEvent, IntoElement, KeyDownEvent, KeystrokeEvent, MouseButton, MouseUpEvent,
    Focusable, Pixels, Point, Render, SharedString, Subscription, Window, div, point, prelude::*, px,
};
use luma::controls::control_group::{ControlGroupEvent, ControlGroupItemRenderModel, ControlGroupItemTemplate};
use luma::controls::listbox::ListBox as SdkListBox;
use luma::infra::ElementExt;
use luma::VisualTransition;
use luma_look_shadcn::{ListBox, ShadcnLook, ShadcnRadius};

#[derive(Clone)]
struct SortableItem {
    id: SharedString,
    label: SharedString,
}

fn accent_for(id: &str) -> gpui::Hsla {
    let hue = match id {
        "research" => 0.58,
        "wireframes" => 0.08,
        "prototype" => 0.75,
        "qa" => 0.32,
        "docs" => 0.16,
        "discovery" => 0.42,
        "spec" => 0.67,
        "priorities" => 0.92,
        "review" => 0.22,
        "release" => 0.05,
        "measure" => 0.48,
        _ => 0.0,
    };
    gpui::hsla(hue, 0.55, 0.52, 1.0)
}

#[derive(Clone)]
struct SortableRow {
    label: SharedString,
    items: Vec<SortableItem>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DropTarget {
    row: usize,
    index: usize,
}

#[derive(Clone)]
struct ReorderableDrag {
    row: usize,
    item_ids: Vec<SharedString>,
    label: SharedString,
    accent: gpui::Hsla,
    look: Arc<ShadcnLook>,
    cursor_offset: Point<Pixels>,
}

struct DropAnimation {
    transition: VisualTransition,
    offsets: HashMap<SharedString, Pixels>,
}

fn apply_reorder_move(
    rows: &mut [SortableRow],
    source_row: usize,
    item_ids: &[SharedString],
    mut target: DropTarget,
) -> bool {
    if item_ids.is_empty() || source_row >= rows.len() || target.row >= rows.len() {
        return false;
    }

    let source_indices: Vec<_> = rows[source_row]
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| item_ids.iter().any(|id| id == &item.id).then_some(index))
        .collect();
    if source_indices.len() != item_ids.len() {
        return false;
    }

    let mut moved_items = Vec::with_capacity(source_indices.len());
    for index in source_indices.iter().rev() {
        moved_items.push(rows[source_row].items.remove(*index));
    }
    moved_items.reverse();

    if target.row == source_row {
        let removed_before_target = source_indices.iter().filter(|index| **index < target.index).count();
        target.index = target.index.saturating_sub(removed_before_target);
    }
    target.index = target.index.min(rows[target.row].items.len());
    for (offset, item) in moved_items.into_iter().enumerate() {
        rows[target.row].items.insert(target.index + offset, item);
    }

    true
}

fn painted_item_positions(
    rows: &[SortableRow],
    item_bounds: &[Vec<Bounds<Pixels>>],
    item_bounds_valid: &[Vec<bool>],
) -> HashMap<SharedString, Pixels> {
    let mut positions = HashMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let Some(bounds) = item_bounds.get(row_index) else {
            continue;
        };
        let Some(valid) = item_bounds_valid.get(row_index) else {
            continue;
        };
        for (item_index, item) in row.items.iter().enumerate() {
            if valid.get(item_index).copied().unwrap_or(false)
                && let Some(bounds) = bounds.get(item_index)
                && !bounds.is_empty()
            {
                positions.insert(item.id.clone(), bounds.origin.y);
            }
        }
    }
    positions
}

fn drop_animation_offsets(
    from_positions: &HashMap<SharedString, Pixels>,
    rows: &[SortableRow],
    item_bounds: &[Vec<Bounds<Pixels>>],
    item_bounds_valid: &[Vec<bool>],
) -> HashMap<SharedString, Pixels> {
    let mut offsets = HashMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let Some(bounds) = item_bounds.get(row_index) else {
            continue;
        };
        let Some(valid) = item_bounds_valid.get(row_index) else {
            continue;
        };
        for (item_index, item) in row.items.iter().enumerate() {
            let Some(from_y) = from_positions.get(&item.id) else {
                continue;
            };
            if !valid.get(item_index).copied().unwrap_or(false) {
                continue;
            }
            let Some(bounds) = bounds.get(item_index) else { continue };
            if bounds.is_empty() {
                continue;
            }
            let offset = *from_y - bounds.origin.y;
            if offset.as_f32().abs() >= 0.5 {
                offsets.insert(item.id.clone(), offset);
            }
        }
    }
    offsets
}

fn select_drag_items(selected_ids: &mut [Vec<SharedString>; 2], row: usize, item_ids: &[SharedString]) {
    for (other_row, ids) in selected_ids.iter_mut().enumerate() {
        if other_row != row {
            ids.clear();
        }
    }
    selected_ids[row] = item_ids.to_vec();
}

const SORTABLE_LISTBOX_VISIBLE_ITEMS: usize = 4;
const SORTABLE_LISTBOX_ITEM_HEIGHT: f32 = 68.0;
const SORTABLE_LISTBOX_HEIGHT: f32 = SORTABLE_LISTBOX_VISIBLE_ITEMS as f32 * SORTABLE_LISTBOX_ITEM_HEIGHT;
const REORDERABLE_EDGE_SCROLL_ZONE: f32 = 56.0;
const REORDERABLE_EDGE_SCROLL_STEP: f32 = 14.0;

#[derive(Clone, Copy)]
pub struct ReorderableCollectionOptions {
    pub allow_same_list_reordering: bool,
    pub source_drag_overlay: Option<gpui::Hsla>,
    pub drop_selection_behavior: DropSelectionBehavior,
}

impl Default for ReorderableCollectionOptions {
    fn default() -> Self {
        Self {
            allow_same_list_reordering: true,
            source_drag_overlay: Some(gpui::hsla(0.0, 0.0, 0.52, 0.58)),
            drop_selection_behavior: DropSelectionBehavior::KeepDroppedItemsOnly,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropSelectionBehavior {
    KeepDroppedItemsOnly,
    ClearAll,
    PreserveExisting,
}

impl Render for ReorderableDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let card = self.look.token_color("card").unwrap_or(self.look.chrome().panel_background);
        let foreground = self.look.token_color("foreground").unwrap_or(self.look.chrome().body_text);
        let focus = self.look.token_color("ring").unwrap_or(self.look.chrome().border);
        let label = if self.item_ids.len() > 1 {
            format!("{} (+{} more)", self.label, self.item_ids.len() - 1)
        } else {
            self.label.to_string()
        };
        div().relative().w(self.cursor_offset.x + px(180.0)).h(px(52.0)).child(
            div()
                .absolute()
                .left(self.cursor_offset.x - px(12.0))
                .top(self.cursor_offset.y - px(12.0))
                .w(px(180.0))
                .px(px(14.0))
                .py(px(10.0))
                .rounded(px(8.0))
                .bg(card)
                .border_1()
                .border_color(focus)
                .shadow_lg()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(self.accent))
                        .child(div().text_sm().text_color(foreground).child(label)),
                ),
        )
    }
}

pub struct ReorderableCollection {
    look: Arc<ShadcnLook>,
    rows: Vec<SortableRow>,
    item_bounds: Vec<Vec<Bounds<Pixels>>>,
    item_bounds_valid: Vec<Vec<bool>>,
    list_bounds: Vec<Bounds<Pixels>>,
    dragging: Option<ReorderableDrag>,
    drop_target: Option<DropTarget>,
    options: ReorderableCollectionOptions,
    selected_ids: [Vec<SharedString>; 2],
    listboxes: [Option<SdkListBox>; 2],
    _subscriptions: Vec<Subscription>,
    drag_position: Option<Point<Pixels>>,
    auto_scroll: Option<(usize, Pixels)>,
    auto_scroll_scheduled: bool,
    drop_animation: Option<DropAnimation>,
    pending_drop_animation: Option<HashMap<SharedString, Pixels>>,
    pending_drop_animation_scheduled: bool,
    suppress_next_click: bool,
    suppress_next_change: bool,
    drag_selection_before: Option<[Vec<SharedString>; 2]>,
    pending_scroll_offsets: Option<[Pixels; 2]>,
    pending_scroll_restore_scheduled: bool,
    pending_scroll_restore_attempts: u8,
    drag_cancel_subscription: Option<Subscription>,
}

impl ReorderableCollection {
    pub fn new(look: Arc<ShadcnLook>) -> Self {
        Self::with_options(look, ReorderableCollectionOptions::default())
    }

    pub fn with_options(look: Arc<ShadcnLook>, options: ReorderableCollectionOptions) -> Self {
        let item = |id: &str, label: &str| SortableItem { id: id.into(), label: label.into() };
        Self {
            look,
            rows: vec![
                SortableRow {
                    label: "Backlog".into(),
                    items: vec![
                        item("research", "Research"),
                        item("wireframes", "Wireframes"),
                        item("discovery", "Discovery"),
                        item("spec", "Specification"),
                        item("priorities", "Priorities"),
                    ],
                },
                SortableRow {
                    label: "In progress".into(),
                    items: vec![
                        item("prototype", "Prototype"),
                        item("qa", "QA pass"),
                        item("docs", "Docs"),
                        item("review", "Design review"),
                        item("release", "Release plan"),
                        item("measure", "Measure results"),
                    ],
                },
            ],
            item_bounds: Vec::new(),
            item_bounds_valid: Vec::new(),
            list_bounds: Vec::new(),
            dragging: None,
            drop_target: None,
            options,
            selected_ids: [Vec::new(), Vec::new()],
            listboxes: [None, None],
            _subscriptions: Vec::new(),
            drag_position: None,
            auto_scroll: None,
            auto_scroll_scheduled: false,
            drop_animation: None,
            pending_drop_animation: None,
            pending_drop_animation_scheduled: false,
            suppress_next_click: false,
            suppress_next_change: false,
            drag_selection_before: None,
            pending_scroll_offsets: None,
            pending_scroll_restore_scheduled: false,
            pending_scroll_restore_attempts: 0,
            drag_cancel_subscription: None,
        }
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.capture_scroll_offsets(cx);
        self.look = look;
        self.listboxes = [None, None];
        self._subscriptions.clear();
        cx.notify();
    }

    fn begin_drag(&mut self, drag: ReorderableDrag, window: &mut Window, cx: &mut Context<Self>) {
        self.drag_selection_before = Some(self.selected_ids.clone());
        self.clear_other_selections_for_active_list(drag.row, cx);
        if self.options.drop_selection_behavior == DropSelectionBehavior::PreserveExisting {
            self.selected_ids[drag.row] = drag.item_ids.clone();
        } else {
            select_drag_items(&mut self.selected_ids, drag.row, &drag.item_ids);
        }
        if let Some(listbox) = self.listboxes[drag.row].clone() {
            listbox.update(cx, |listbox, cx| listbox.focus_handle(cx).focus(window, cx));
        }
        self.dragging = Some(drag.clone());
        self.drop_target = None;
        self.drag_position = None;
        self.auto_scroll = None;
        self.suppress_next_click = false;
        self.suppress_next_change = false;
        cx.notify();
    }

    fn update_drop_target(
        &mut self,
        event: &DragMoveEvent<ReorderableDrag>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let drag = event.drag(cx);
        let Some(session) = self.dragging.as_ref() else { return };
        if drag.row != session.row {
            return;
        }
        self.update_drop_target_at(event.event.position, window, cx);
    }

    fn update_drop_target_at(&mut self, position: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        let Some((session_row, session_item_ids)) =
            self.dragging.as_ref().map(|session| (session.row, session.item_ids.clone()))
        else {
            return;
        };
        self.drag_position = Some(position);
        self.auto_scroll = self.edge_scroll_target(position);
        if self.auto_scroll.is_some() {
            self.schedule_auto_scroll(window, cx);
        }
        let mut target = self.item_bounds.iter().enumerate().find_map(|(row, bounds)| {
            let list_bounds = self.list_bounds.get(row)?;
            if !list_bounds.contains(&position) {
                return None;
            }
            let valid = self.item_bounds_valid.get(row)?;
            let painted: Vec<_> = bounds
                .iter()
                .enumerate()
                .filter_map(|(index, bounds)| valid.get(index).copied().unwrap_or(false).then_some((index, *bounds)))
                .filter(|(_, bounds)| !bounds.is_empty())
                .collect();
            if painted.is_empty() {
                if self.rows.get(row).is_some_and(|row| row.items.is_empty()) {
                    return Some(DropTarget { row, index: 0 });
                }
                return None;
            }
            let (_, first) = painted.first()?;
            let (last_index, last) = painted.last()?;
            if position.x < first.origin.x - px(8.0) || position.x > first.right() + px(8.0) {
                return None;
            }
            if position.y < first.origin.y {
                return Some(DropTarget { row, index: 0 });
            }
            if position.y > last.bottom() {
                return Some(DropTarget { row, index: last_index + 1 });
            }
            let index = painted
                .iter()
                .find(|(_, bounds)| position.y < bounds.center().y)
                .map(|(index, _)| *index)
                .unwrap_or(last_index + 1);
            Some(DropTarget { row, index })
        });
        if !self.options.allow_same_list_reordering && target.is_some_and(|target| target.row == session_row) {
            target = None;
        }
        if let Some(candidate) = target
            && candidate.row == session_row
        {
            let selected_indices: Vec<_> = self.rows[session_row]
                .items
                .iter()
                .enumerate()
                .filter_map(|(index, item)| session_item_ids.iter().any(|id| id == &item.id).then_some(index))
                .collect();
            let contiguous = selected_indices.windows(2).all(|indices| indices[1] == indices[0] + 1);
            if contiguous
                && let (Some(first), Some(last)) = (selected_indices.first(), selected_indices.last())
                && candidate.index >= *first
                && candidate.index <= *last + 1
            {
                target = None;
            }
        }
        cx.set_active_drag_cursor_style(
            if target.is_some() {
                CursorStyle::DragCopy
            } else {
                CursorStyle::OperationNotAllowed
            },
            window,
        );
        let target_changed = self.drop_target != target;
        if target_changed {
            self.drop_target = target;
        }
        if target_changed {
            cx.notify();
        }
    }

    fn edge_scroll_target(&self, position: Point<Pixels>) -> Option<(usize, Pixels)> {
        self.list_bounds.iter().enumerate().find_map(|(row, bounds)| {
            if !bounds.contains(&position) {
                return None;
            }

            let edge = px(REORDERABLE_EDGE_SCROLL_ZONE);
            if position.y <= bounds.origin.y + edge {
                Some((row, px(-REORDERABLE_EDGE_SCROLL_STEP)))
            } else if position.y >= bounds.bottom() - edge {
                Some((row, px(REORDERABLE_EDGE_SCROLL_STEP)))
            } else {
                None
            }
        })
    }

    fn schedule_auto_scroll(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.auto_scroll_scheduled {
            return;
        }
        self.auto_scroll_scheduled = true;
        cx.on_next_frame(window, |this, window, cx| {
            this.auto_scroll_scheduled = false;
            let Some((row, delta)) = this.auto_scroll else { return };
            let Some(listbox) = this.listboxes[row].clone() else {
                return;
            };
            let moved = listbox.update(cx, |listbox, cx| listbox.scroll_vertical_by(delta, cx));
            if moved {
                cx.notify();
                if let Some(position) = this.drag_position {
                    this.update_drop_target_at(position, window, cx);
                }
            } else {
                this.auto_scroll = None;
            }
        });
    }

    fn finish_drag(&mut self, _: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.drag_position = None;
        self.auto_scroll = None;
        let Some(session) = self.dragging.take() else { return };
        self.suppress_gesture_selection(window, cx);
        let Some(target) = self.drop_target.take() else {
            self.restore_drag_selection(cx);
            cx.notify();
            return;
        };
        if session.row >= self.rows.len() || target.row >= self.rows.len() {
            self.restore_drag_selection(cx);
            cx.notify();
            return;
        }

        let old_positions = painted_item_positions(&self.rows, &self.item_bounds, &self.item_bounds_valid);
        if !apply_reorder_move(&mut self.rows, session.row, &session.item_ids, target) {
            self.restore_drag_selection(cx);
            cx.notify();
            return;
        }

        self.drag_selection_before = None;
        self.pending_drop_animation = Some(old_positions);
        match self.options.drop_selection_behavior {
            DropSelectionBehavior::KeepDroppedItemsOnly => {
                self.selected_ids.iter_mut().for_each(Vec::clear);
                self.selected_ids[target.row] = session.item_ids.clone();
            }
            DropSelectionBehavior::ClearAll => {
                self.selected_ids.iter_mut().for_each(Vec::clear);
            }
            DropSelectionBehavior::PreserveExisting => {
                if target.row != session.row {
                    self.selected_ids[session.row].retain(|id| !session.item_ids.iter().any(|moved_id| moved_id == id));
                    self.selected_ids[target.row].extend(session.item_ids.iter().cloned());
                }
            }
        }
        self.sync_listboxes(cx);
        for (row, listbox) in self.listboxes.iter().enumerate() {
            if let Some(listbox) = listbox {
                let selected_ids = self.selected_ids[row].clone();
                listbox.update(cx, |listbox, cx| listbox.set_selected_ids(selected_ids, cx));
            }
        }
        cx.notify();
    }

    fn start_drop_animation(&mut self, offsets: HashMap<SharedString, Pixels>) {
        if !offsets.is_empty() {
            let mut transition = VisualTransition::new(1.0, Duration::from_millis(180));
            transition.snap_to(0.0);
            transition.set_target(1.0);
            self.drop_animation = Some(DropAnimation { transition, offsets });
        } else {
            self.drop_animation = None;
        }
    }

    fn schedule_pending_drop_animation(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.pending_drop_animation.is_none() || self.pending_drop_animation_scheduled {
            return;
        }
        self.pending_drop_animation_scheduled = true;
        cx.on_next_frame(window, |this, window, cx| {
            this.pending_drop_animation_scheduled = false;
            if this.rows.iter().all(|row| row.items.is_empty()) {
                this.pending_drop_animation = None;
                return;
            }
            if !this.item_bounds_valid.iter().flatten().any(|valid| *valid) {
                this.schedule_pending_drop_animation(window, cx);
                return;
            }
            let Some(from_positions) = this.pending_drop_animation.take() else {
                return;
            };
            let offsets =
                drop_animation_offsets(&from_positions, &this.rows, &this.item_bounds, &this.item_bounds_valid);
            this.start_drop_animation(offsets);
            cx.notify();
        });
    }

    fn drop_animation_offset(&self, item_id: &SharedString) -> Option<Pixels> {
        let animation = self.drop_animation.as_ref()?;
        let offset = animation.offsets.get(item_id)?;
        Some(px(animation.transition.interpolate(offset.as_f32(), 0.0)))
    }

    fn cancel_drag_session(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.dragging.is_none() {
            return false;
        }
        cx.stop_active_drag(window);
        self.dragging.take();
        self.drag_position = None;
        self.auto_scroll = None;
        window.prevent_default();
        self.drop_target = None;
        self.suppress_gesture_selection(window, cx);
        self.restore_drag_selection(cx);
        cx.notify();
        true
    }

    fn cancel_drag(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key.as_str() != "escape" || !self.cancel_drag_session(window, cx) {
            return;
        }
        cx.stop_propagation();
    }

    fn intercept_drag_cancel(&mut self, event: &KeystrokeEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key.as_str() == "escape" && self.cancel_drag_session(window, cx) {
            cx.stop_propagation();
        }
    }

    fn restore_drag_selection(&mut self, cx: &mut Context<Self>) {
        let Some(selected_ids) = self.drag_selection_before.take() else {
            return;
        };
        self.selected_ids = selected_ids;
        for (row, listbox) in self.listboxes.iter().enumerate() {
            if let Some(listbox) = listbox {
                let selected_ids = self.selected_ids[row].clone();
                listbox.update(cx, |listbox, cx| listbox.set_selected_ids(selected_ids, cx));
            }
        }
    }

    fn consume_suppressed_click(&mut self) -> bool {
        let suppressed = self.suppress_next_click;
        self.suppress_next_click = false;
        suppressed
    }

    fn suppress_gesture_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.suppress_next_click = true;
        self.suppress_next_change = true;
        cx.on_next_frame(window, |this, _, _| {
            this.suppress_next_click = false;
            this.suppress_next_change = false;
        });
    }

    fn sync_listboxes(&mut self, cx: &mut Context<Self>) {
        self.capture_scroll_offsets(cx);
        for (row, listbox) in self.listboxes.iter().enumerate() {
            if let Some(listbox) = listbox {
                listbox.update(cx, |listbox, cx| {
                    listbox.set_items(
                        self.rows[row].items.iter().map(|item| {
                            luma::controls::listbox::ListBoxItem::new(item.id.clone(), item.id.clone())
                                .label(item.label.clone())
                        }),
                        cx,
                    );
                });
            }
        }
    }

    fn capture_scroll_offsets(&mut self, cx: &mut Context<Self>) {
        self.pending_scroll_offsets = Some(std::array::from_fn(|row| {
            self.listboxes[row].as_ref().map_or(px(0.0), |listbox| listbox.read(cx).vertical_scroll_offset())
        }));
        self.pending_scroll_restore_attempts = 3;
    }

    fn schedule_pending_scroll_restore(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.pending_scroll_offsets.is_none() || self.pending_scroll_restore_scheduled {
            return;
        }
        self.pending_scroll_restore_scheduled = true;
        cx.on_next_frame(window, |this, window, cx| {
            this.pending_scroll_restore_scheduled = false;
            let Some(offsets) = this.pending_scroll_offsets.take() else {
                return;
            };
            let mut retry_offsets = [px(0.0); 2];
            let mut retry = false;
            for (row, offset) in offsets.into_iter().enumerate() {
                let Some(listbox) = this.listboxes[row].clone() else {
                    continue;
                };
                listbox.update(cx, |listbox, cx| listbox.set_vertical_scroll_offset(offset, cx));
                let max_offset = listbox.read(cx).vertical_scroll_max_offset();
                if offset > px(0.5)
                    && max_offset <= px(0.0)
                    && this.rows[row].items.len() > SORTABLE_LISTBOX_VISIBLE_ITEMS
                {
                    retry_offsets[row] = offset;
                    retry = true;
                }
            }
            if retry && this.pending_scroll_restore_attempts > 0 {
                this.pending_scroll_restore_attempts -= 1;
                this.pending_scroll_offsets = Some(retry_offsets);
                this.schedule_pending_scroll_restore(window, cx);
            }
        });
    }

    fn clear_other_selections(&mut self, selected_row: usize, cx: &mut Context<Self>) {
        for row in 0..self.selected_ids.len() {
            if row == selected_row || self.selected_ids[row].is_empty() {
                continue;
            }
            self.selected_ids[row].clear();
            if let Some(listbox) = self.listboxes[row].clone() {
                listbox.update(cx, |listbox, cx| listbox.set_selected_ids(Vec::<SharedString>::new(), cx));
            }
        }
    }

    fn clear_other_selections_for_active_list(&mut self, selected_row: usize, cx: &mut Context<Self>) {
        if self.options.drop_selection_behavior != DropSelectionBehavior::PreserveExisting {
            self.clear_other_selections(selected_row, cx);
        }
    }

    fn create_listbox(&mut self, row_index: usize, cx: &mut Context<Self>) -> SdkListBox {
        let prototype = cx.entity();
        let item_prototype = prototype.clone();
        let item_template: ControlGroupItemTemplate<luma::controls::listbox::ListBoxItem> =
            Arc::new(move |item: &ControlGroupItemRenderModel<'_, _>, _, app| {
                let item_look = item_prototype.read(app).look.clone();
                let accent = accent_for(item.item.id().as_ref());
                let foreground = item_look.token_color("foreground").unwrap_or(item_look.chrome().body_text);
                let muted = item_look.token_color("muted-foreground").unwrap_or(item_look.chrome().muted_text);
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(accent))
                    .child(
                        div()
                            .flex_1()
                            .child(div().text_sm().text_color(foreground).child(item.item.label_text().clone()))
                            .child(
                                div().text_xs().text_color(muted).child(format!("{} · drag to move", item.item.id())),
                            ),
                    )
                    .into_any_element()
            });
        let bounds_host_seed = prototype.clone();
        let start_host_seed = prototype.clone();
        let finish_host_seed = prototype.clone();
        let finish_host_out_seed = prototype.clone();
        let item_element_template = Arc::new(
            move |item: &ControlGroupItemRenderModel<'_, luma::controls::listbox::ListBoxItem>,
                  item_template: Option<&ControlGroupItemTemplate<luma::controls::listbox::ListBoxItem>>,
                  window: &mut Window,
                  app: &mut gpui::App| {
                let current_look = prototype.read(app).look.clone();
                let item_id = item.item.id().clone();
                let item_index = item.index;
                let sibling_count = item.sibling_count;
                let bounds_host = bounds_host_seed.clone();
                let start_host = start_host_seed.clone();
                let selection_host = start_host_seed.clone();
                let click_host = prototype.clone();
                let finish_host = finish_host_seed.clone();
                let finish_host_out = finish_host_out_seed.clone();
                let drag = ReorderableDrag {
                    row: row_index,
                    item_ids: vec![item_id.clone()],
                    label: item.item.label_text().clone(),
                    accent: accent_for(item_id.as_ref()),
                    look: current_look.clone(),
                    cursor_offset: point(px(0.0), px(0.0)),
                };
                let drop_target = prototype.read(app).drop_target;
                let (source_drag_overlay, is_drag_source) = {
                    let state = prototype.read(app);
                    (
                        state.options.source_drag_overlay,
                        state.dragging.as_ref().is_some_and(|drag| drag.item_ids.iter().any(|id| id == &item_id)),
                    )
                };
                let target_before = drop_target == Some(DropTarget { row: row_index, index: item_index });
                let target_at_end = drop_target == Some(DropTarget { row: row_index, index: item_index + 1 })
                    && item_index + 1 == sibling_count;
                let card = current_look.token_color("card").unwrap_or(current_look.chrome().panel_background);
                let border = current_look.token_color("border").unwrap_or(current_look.chrome().border);
                let selection_border = current_look.token_color("ring").unwrap_or(border);
                let content = item_template
                    .map(|template| template(item, window, app))
                    .unwrap_or_else(|| div().into_any_element());
                let mut element = div()
                    .id(format!("sortable-listbox-item-{}", item_id))
                    .relative()
                    .w_full()
                    .h(px(SORTABLE_LISTBOX_ITEM_HEIGHT))
                    .flex_none()
                    .px(px(18.0))
                    .py(px(12.0))
                    .rounded(px(current_look.radius(ShadcnRadius::Lg)))
                    .bg(card)
                    .border_1()
                    .border_color(if item.selected { selection_border } else { border })
                    .cursor_grab()
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        selection_host.update(cx, |this, cx| {
                            this.clear_other_selections_for_active_list(row_index, cx);
                        });
                    })
                    .on_click(move |_, _, cx| {
                        if click_host.update(cx, |this, _| this.consume_suppressed_click()) {
                            cx.stop_propagation();
                        }
                    })
                    .on_mouse_up(MouseButton::Left, move |event, window, cx| {
                        finish_host.update(cx, |this, cx| this.finish_drag(event, window, cx));
                    })
                    .on_mouse_up_out(MouseButton::Left, move |event, window, cx| {
                        finish_host_out.update(cx, |this, cx| this.finish_drag(event, window, cx));
                    })
                    .on_prepaint(move |bounds, _, cx| {
                        bounds_host.update(cx, |this, _| {
                            if let Some(slot) =
                                this.item_bounds.get_mut(row_index).and_then(|bounds| bounds.get_mut(item_index))
                            {
                                *slot = bounds;
                            }
                            if let Some(slot) =
                                this.item_bounds_valid.get_mut(row_index).and_then(|valid| valid.get_mut(item_index))
                            {
                                *slot = true;
                            }
                        });
                    })
                    .child(content);
                if let Some(offset) = prototype.read(app).drop_animation_offset(&item_id) {
                    element = element.top(offset);
                }
                let selection_host = start_host_seed.clone();
                element = element.on_drag(drag, move |payload, cursor_offset, window, cx| {
                    let selected_ids = {
                        let state = selection_host.read(cx);
                        if state.selected_ids[row_index].iter().any(|id| id == &payload.item_ids[0]) {
                            state.selected_ids[row_index].clone()
                        } else {
                            vec![payload.item_ids[0].clone()]
                        }
                    };
                    let mut payload = payload.clone();
                    payload.item_ids = selected_ids.clone();
                    start_host.update(cx, |this, cx| {
                        if let Some(listbox) = this.listboxes[row_index].clone() {
                            listbox.update(cx, |listbox, cx| listbox.set_selected_ids(selected_ids.clone(), cx));
                        }
                        this.begin_drag(payload.clone(), window, cx);
                    });
                    let mut preview = payload;
                    preview.cursor_offset = cursor_offset;
                    cx.new(|_| preview)
                });
                if target_before || target_at_end {
                    let marker =
                        div().absolute().left(px(0.0)).right(px(0.0)).h(px(4.0)).rounded(px(2.0)).bg(prototype
                            .read(app)
                            .look
                            .token_color("ring")
                            .unwrap_or(prototype.read(app).look.chrome().border));
                    element = if target_before {
                        element.child(marker.top(px(-3.0)))
                    } else {
                        element.child(marker.bottom(px(-3.0)))
                    };
                }
                if is_drag_source && let Some(overlay) = source_drag_overlay {
                    element = element.child(
                        div().absolute().inset_0().rounded(px(current_look.radius(ShadcnRadius::Lg))).bg(overlay),
                    );
                }
                element
            },
        );
        let items = self.rows[row_index].items.iter().map(|item| {
            luma::controls::listbox::ListBoxItem::new(item.id.clone(), item.id.clone()).label(item.label.clone())
        });
        ListBox::multiple(format!("developer-sortable-listbox-{row_index}"))
            .look(&self.look)
            .template(self.look.listbox_template())
            .with_template_modifier(|root, _| root.h(px(SORTABLE_LISTBOX_HEIGHT)))
            .items(items)
            .item_template(item_template)
            .item_element_template(item_element_template)
            .spawn(cx)
    }
}

impl Render for ReorderableCollection {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.drag_cancel_subscription.is_none() {
            let listener = cx.listener(Self::intercept_drag_cancel);
            self.drag_cancel_subscription = Some(cx.intercept_keystrokes(listener));
        }
        if let Some(animation) = self.drop_animation.as_mut() {
            let animating = animation.transition.sync();
            if animating {
                animation.transition.schedule_frame(window, cx);
            } else {
                self.drop_animation = None;
            }
        }
        if self.listboxes[0].is_none() {
            for row_index in 0..self.rows.len() {
                let listbox = self.create_listbox(row_index, cx);
                let subscription = cx.subscribe(&listbox, move |this, _, event, cx| {
                    if let ControlGroupEvent::Change { selected_ids, .. } = event {
                        if this.suppress_next_change {
                            this.suppress_next_change = false;
                            return;
                        }
                        this.selected_ids[row_index] = selected_ids.clone();
                        if !selected_ids.is_empty() {
                            this.clear_other_selections_for_active_list(row_index, cx);
                        }
                        cx.notify();
                    }
                });
                self._subscriptions.push(subscription);
                self.listboxes[row_index] = Some(listbox);
            }
        }
        self.schedule_pending_scroll_restore(window, cx);
        self.item_bounds = self.rows.iter().map(|row| vec![Bounds::default(); row.items.len()]).collect();
        self.item_bounds_valid = self.rows.iter().map(|row| vec![false; row.items.len()]).collect();
        self.schedule_pending_drop_animation(window, cx);
        self.list_bounds = vec![Bounds::default(); self.rows.len()];
        let background = self.look.token_color("background").unwrap_or(self.look.chrome().app_background);
        let foreground = self.look.token_color("foreground").unwrap_or(self.look.chrome().body_text);
        let muted = self.look.token_color("muted-foreground").unwrap_or(self.look.chrome().muted_text);
        let drop_target = self.drop_target;
        let drop_marker = self.look.token_color("ring").unwrap_or(self.look.chrome().border);
        let listboxes: Vec<_> = self
            .listboxes
            .iter()
            .zip(&self.rows)
            .enumerate()
            .map(|(row_index, (listbox, row))| {
                let bounds_host = cx.entity();
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(foreground)
                                    .child(row.label.clone()),
                            )
                            .child(div().text_xs().text_color(muted).child(format!("{} items", row.items.len()))),
                    )
                    .child({
                        let empty_drop_target =
                            row.items.is_empty() && drop_target == Some(DropTarget { row: row_index, index: 0 });
                        let mut viewport = div()
                            .relative()
                            .h(px(SORTABLE_LISTBOX_HEIGHT))
                            .capture_key_down(cx.listener(Self::cancel_drag))
                            .on_prepaint(move |bounds, _, cx| {
                                bounds_host.update(cx, |this, _| {
                                    if let Some(slot) = this.list_bounds.get_mut(row_index) {
                                        *slot = bounds;
                                    }
                                });
                            })
                            .child(listbox.as_ref().expect("listbox initialized").clone());
                        if empty_drop_target {
                            viewport = viewport.child(
                                div()
                                    .absolute()
                                    .left(px(12.0))
                                    .right(px(12.0))
                                    .top(px(SORTABLE_LISTBOX_HEIGHT / 2.0))
                                    .h(px(4.0))
                                    .rounded(px(2.0))
                                    .bg(drop_marker),
                            );
                        }
                        viewport
                    })
            })
            .collect();
        div()
            .id("developer-sortable-collection")
            .size_full()
            .overflow_y_scroll()
            .bg(background)
            .px(px(32.0))
            .py(px(28.0))
            .on_drag_move(cx.listener(Self::update_drop_target))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::finish_drag))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::finish_drag))
            .capture_key_down(cx.listener(Self::cancel_drag))
            .child(
                div()
                    .max_w(px(980.0))
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(foreground)
                                    .child("Developer · Reorderable collection"),
                            )
                            .child(div().text_sm().text_color(muted).child(
                                "SDK ListBox prototype with configurable cross-list moves and same-list reordering.",
                            )),
                    )
                    .child(div().flex().gap(px(16.0)).children(listboxes)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{DropTarget, SortableItem, SortableRow, apply_reorder_move, drop_animation_offsets, select_drag_items};
    use gpui::{Bounds, SharedString, point, px, size};

    fn row(ids: &[&str]) -> SortableRow {
        SortableRow {
            label: "row".into(),
            items: ids.iter().map(|id| SortableItem { id: (*id).into(), label: (*id).into() }).collect(),
        }
    }

    fn ids(rows: &[SortableRow]) -> Vec<Vec<SharedString>> {
        rows.iter().map(|row| row.items.iter().map(|item| item.id.clone()).collect()).collect()
    }

    fn shared_ids(ids: &[&str]) -> Vec<SharedString> {
        ids.iter().map(|id| (*id).into()).collect()
    }

    #[test]
    fn cross_list_move_preserves_selected_order() {
        let mut rows = vec![row(&["a", "b", "c", "d"]), row(&["x", "y"])];
        let selected = vec![SharedString::from("b"), SharedString::from("c")];

        assert!(apply_reorder_move(&mut rows, 0, &selected, DropTarget { row: 1, index: 1 }));
        assert_eq!(ids(&rows), vec![shared_ids(&["a", "d"]), shared_ids(&["x", "b", "c", "y"])]);
    }

    #[test]
    fn same_list_non_contiguous_move_preserves_relative_order() {
        let mut rows = vec![row(&["a", "b", "c", "d", "e"]), row(&[])];
        let selected = vec![SharedString::from("b"), SharedString::from("d")];

        assert!(apply_reorder_move(&mut rows, 0, &selected, DropTarget { row: 0, index: 5 }));
        assert_eq!(ids(&rows)[0], shared_ids(&["a", "c", "e", "b", "d"]));
    }

    #[test]
    fn invalid_move_leaves_rows_unchanged() {
        let mut rows = vec![row(&["a", "b"]), row(&["x"])];
        let before = ids(&rows);

        assert!(!apply_reorder_move(&mut rows, 0, &[SharedString::from("missing")], DropTarget { row: 1, index: 0 },));
        assert_eq!(ids(&rows), before);
    }

    #[test]
    fn starting_drag_clears_selection_in_the_other_list() {
        let mut selected_ids = [shared_ids(&["source"]), shared_ids(&["destination"])];

        select_drag_items(&mut selected_ids, 0, &shared_ids(&["source"]));

        assert_eq!(selected_ids, [shared_ids(&["source"]), Vec::new()]);
    }

    fn bounds_at(y: f32) -> Bounds<gpui::Pixels> {
        Bounds::new(point(px(0.0), px(y)), size(px(100.0), px(10.0)))
    }

    #[test]
    fn drop_animation_offsets_follow_same_list_shift() {
        let rows = vec![row(&["a", "b", "c"]), row(&[])];
        let from_positions = [("a", 10.0), ("b", 78.0), ("c", 146.0)]
            .into_iter()
            .map(|(id, y)| (SharedString::from(id), px(y)))
            .collect();
        let bounds = vec![vec![bounds_at(10.0), bounds_at(146.0), bounds_at(78.0)], vec![]];
        let valid = vec![vec![true, true, true], vec![]];

        let offsets = drop_animation_offsets(&from_positions, &rows, &bounds, &valid);

        assert_eq!(offsets["b"].as_f32(), -68.0);
        assert_eq!(offsets["c"].as_f32(), 68.0);
    }

    #[test]
    fn drop_animation_offsets_support_cross_list_moves() {
        let rows = vec![row(&["a"]), row(&["b"])];
        let from_positions = [(SharedString::from("b"), px(78.0))].into_iter().collect();
        let bounds = vec![vec![bounds_at(10.0)], vec![bounds_at(200.0)]];
        let valid = vec![vec![true], vec![true]];

        let offsets = drop_animation_offsets(&from_positions, &rows, &bounds, &valid);

        assert_eq!(offsets["b"].as_f32(), -122.0);
    }

    #[test]
    fn drop_animation_offsets_skip_unpainted_items() {
        let rows = vec![row(&["a", "b"]), row(&[])];
        let from_positions =
            [(SharedString::from("a"), px(10.0)), (SharedString::from("b"), px(78.0))].into_iter().collect();
        let bounds = vec![vec![bounds_at(100.0), Bounds::default()], vec![]];
        let valid = vec![vec![false, false], vec![]];

        let offsets = drop_animation_offsets(&from_positions, &rows, &bounds, &valid);

        assert!(offsets.is_empty());
    }
}
