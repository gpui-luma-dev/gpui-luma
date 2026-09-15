use std::sync::Arc;
use std::time::Instant;

use gpui::{
    Bounds, Context, DragMoveEvent, IntoElement, KeyDownEvent, MouseButton, MouseUpEvent, Pixels, Point, Render,
    SharedString, Window, div, prelude::*, point, px,
};
use luma::infra::ElementExt;
use luma_look_shadcn::ShadcnLook;

#[derive(Clone)]
struct SortableItem {
    id: SharedString,
    label: SharedString,
    accent: gpui::Hsla,
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
struct SortableDrag {
    row: usize,
    index: usize,
    item_id: SharedString,
    label: SharedString,
    accent: gpui::Hsla,
    look: Arc<ShadcnLook>,
    cursor_offset: Point<Pixels>,
}

struct DragTrace {
    item_id: SharedString,
    started: Instant,
    first_move: Option<Instant>,
    last_move: Option<Instant>,
    mouse_up: Option<Instant>,
    finish: Option<Instant>,
    commit: Option<Instant>,
    target_changes: usize,
}

impl DragTrace {
    fn new(item_id: SharedString) -> Self {
        Self {
            item_id,
            started: Instant::now(),
            first_move: None,
            last_move: None,
            mouse_up: None,
            finish: None,
            commit: None,
            target_changes: 0,
        }
    }

    fn elapsed_ms(&self, timestamp: Option<Instant>) -> Option<u128> {
        timestamp.map(|timestamp| timestamp.duration_since(self.started).as_millis())
    }

    fn print(&self, outcome: &str) {
        println!(
            "sortable_drag item={} outcome={} targets={} first_move={}ms last_move={}ms mouse_up={}ms finish={}ms commit={}ms total={}ms",
            self.item_id,
            outcome,
            self.target_changes,
            self.elapsed_ms(self.first_move).map_or_else(|| "-".into(), |ms| ms.to_string()),
            self.elapsed_ms(self.last_move).map_or_else(|| "-".into(), |ms| ms.to_string()),
            self.elapsed_ms(self.mouse_up).map_or_else(|| "-".into(), |ms| ms.to_string()),
            self.elapsed_ms(self.finish).map_or_else(|| "-".into(), |ms| ms.to_string()),
            self.elapsed_ms(self.commit).map_or_else(|| "-".into(), |ms| ms.to_string()),
            self.started.elapsed().as_millis(),
        );
    }
}

impl Render for SortableDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let card = self.look.token_color("card").unwrap_or(self.look.chrome().panel_background);
        let foreground = self.look.token_color("foreground").unwrap_or(self.look.chrome().body_text);
        let focus = self.look.token_color("ring").unwrap_or(self.look.chrome().border);
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
                        .child(div().text_sm().text_color(foreground).child(self.label.clone())),
                ),
        )
    }
}

pub struct SortableCollectionPrototype {
    look: Arc<ShadcnLook>,
    rows: Vec<SortableRow>,
    item_bounds: Vec<Vec<Bounds<Pixels>>>,
    dragging: Option<SortableDrag>,
    drop_target: Option<DropTarget>,
    drag_trace: Option<DragTrace>,
    pending_trace: Option<(DragTrace, &'static str)>,
    last_event: SharedString,
}

impl SortableCollectionPrototype {
    pub fn new(look: Arc<ShadcnLook>) -> Self {
        let item = |id: &str, label: &str, hue: f32| SortableItem {
            id: id.into(),
            label: label.into(),
            accent: gpui::hsla(hue, 0.55, 0.52, 1.0),
        };
        Self {
            look,
            rows: vec![
                SortableRow {
                    label: "Backlog".into(),
                    items: vec![item("research", "Research", 0.58), item("wireframes", "Wireframes", 0.08)],
                },
                SortableRow {
                    label: "In progress".into(),
                    items: vec![
                        item("prototype", "Prototype", 0.75),
                        item("qa", "QA pass", 0.32),
                        item("docs", "Docs", 0.16),
                    ],
                },
            ],
            item_bounds: Vec::new(),
            dragging: None,
            drop_target: None,
            drag_trace: None,
            pending_trace: None,
            last_event: "Ready — drag an item within or between rows".into(),
        }
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }

    fn begin_drag(&mut self, drag: SortableDrag, cx: &mut Context<Self>) {
        self.dragging = Some(drag.clone());
        self.drop_target = Some(DropTarget { row: drag.row, index: drag.index });
        self.drag_trace = Some(DragTrace::new(drag.item_id.clone()));
        self.last_event = format!("Dragging {}", drag.item_id).into();
        cx.notify();
    }

    fn update_drop_target(
        &mut self,
        event: &DragMoveEvent<SortableDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let drag = event.drag(cx);
        let Some(session) = self.dragging.as_ref() else { return };
        if drag.item_id != session.item_id {
            return;
        }
        let position = event.event.position;
        if let Some(trace) = self.drag_trace.as_mut() {
            let now = Instant::now();
            trace.first_move.get_or_insert(now);
            trace.last_move = Some(now);
        }
        let mut target = self.item_bounds.iter().enumerate().find_map(|(row, bounds)| {
            let first = bounds.first()?;
            let last = bounds.last()?;
            if position.y < first.origin.y - px(8.0) || position.y > last.bottom() + px(8.0) {
                return None;
            }
            let index = bounds.iter().position(|bound| position.y < bound.center().y).unwrap_or(bounds.len());
            Some(DropTarget { row, index })
        });
        if target.is_some_and(|target| {
            target.row == session.row && (target.index == session.index || target.index == session.index + 1)
        }) {
            target = None;
        }
        let target_changed = self.drop_target != target;
        if target_changed {
            if let Some(trace) = self.drag_trace.as_mut() {
                trace.target_changes += 1;
            }
            self.drop_target = target;
            self.last_event = target.map_or_else(
                || "No valid insertion target".into(),
                |target| format!("Target: row {}, position {}", target.row + 1, target.index + 1).into(),
            );
        }
        if target_changed {
            cx.notify();
        }
    }

    fn finish_drag(&mut self, _: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(session) = self.dragging.take() else { return };
        let mut trace = self.drag_trace.take();
        if let Some(trace) = trace.as_mut() {
            trace.mouse_up = Some(Instant::now());
            trace.finish = Some(Instant::now());
        }
        let Some(mut target) = self.drop_target.take() else {
            if let Some(trace) = trace {
                self.pending_trace = Some((trace, "cancelled"));
            }
            self.last_event = "Cancelled — no insertion target".into();
            cx.notify();
            return;
        };
        if session.row >= self.rows.len()
            || session.index >= self.rows[session.row].items.len()
            || target.row >= self.rows.len()
        {
            if let Some(trace) = trace {
                self.pending_trace = Some((trace, "cancelled"));
            }
            self.last_event = "Cancelled — invalid target".into();
            cx.notify();
            return;
        }
        let item = self.rows[session.row].items.remove(session.index);
        if target.row == session.row && target.index > session.index {
            target.index -= 1;
        }
        target.index = target.index.min(self.rows[target.row].items.len());
        self.rows[target.row].items.insert(target.index, item);
        if let Some(trace) = trace {
            let mut trace = trace;
            trace.commit = Some(Instant::now());
            self.pending_trace = Some((trace, "committed"));
        }
        self.last_event =
            format!("Committed move: {} → row {}, position {}", session.item_id, target.row + 1, target.index + 1)
                .into();
        cx.notify();
    }

    fn cancel_drag(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key.as_str() != "escape" || self.dragging.take().is_none() {
            return;
        }
        if let Some(trace) = self.drag_trace.take() {
            self.pending_trace = Some((trace, "cancelled"));
        }
        window.prevent_default();
        cx.stop_propagation();
        self.drop_target = None;
        self.last_event = "Cancelled — Escape pressed".into();
        cx.notify();
    }
}

impl Render for SortableCollectionPrototype {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some((trace, outcome)) = self.pending_trace.take() {
            trace.print(outcome);
        }
        let background = self.look.token_color("background").unwrap_or(self.look.chrome().app_background);
        let card = self.look.token_color("card").unwrap_or(self.look.chrome().panel_background);
        let foreground = self.look.token_color("foreground").unwrap_or(self.look.chrome().body_text);
        let muted = self.look.token_color("muted-foreground").unwrap_or(self.look.chrome().muted_text);
        let border = self.look.token_color("border").unwrap_or(self.look.chrome().border);
        let focus = self.look.token_color("ring").unwrap_or(self.look.chrome().border);
        self.item_bounds = self.rows.iter().map(|row| vec![Bounds::default(); row.items.len()]).collect();
        let prototype = cx.entity();
        let mut rows = Vec::new();

        for (row_index, row) in self.rows.iter().enumerate() {
            let mut items = Vec::new();
            for (item_index, item) in row.items.iter().enumerate() {
                let drag = SortableDrag {
                    row: row_index,
                    index: item_index,
                    item_id: item.id.clone(),
                    label: item.label.clone(),
                    accent: item.accent,
                    look: self.look.clone(),
                    cursor_offset: point(px(0.0), px(0.0)),
                };
                let bounds_host = prototype.clone();
                let start_host = prototype.clone();
                let finish_host = prototype.clone();
                let target_before = self.drop_target == Some(DropTarget { row: row_index, index: item_index });
                let target_at_end = self.drop_target == Some(DropTarget { row: row_index, index: row.items.len() })
                    && item_index + 1 == row.items.len();
                let mut item_view = div()
                    .id(format!("sortable-item-{}", item.id))
                    .relative()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .w_full()
                    .min_h(px(52.0))
                    .px(px(14.0))
                    .py(px(10.0))
                    .rounded(px(8.0))
                    .bg(card)
                    .border_1()
                    .border_color(border)
                    .cursor_grab()
                    .on_mouse_up(MouseButton::Left, move |event, window, cx| {
                        finish_host.update(cx, |this, cx| this.finish_drag(event, window, cx));
                    })
                    .on_mouse_up_out(MouseButton::Left, cx.listener(Self::finish_drag))
                    .on_drag(drag, move |payload, cursor_offset, _, cx| {
                        start_host.update(cx, |this, cx| this.begin_drag(payload.clone(), cx));
                        let mut preview = payload.clone();
                        preview.cursor_offset = cursor_offset;
                        cx.new(|_| preview)
                    })
                    .on_prepaint(move |bounds, _, cx| {
                        bounds_host.update(cx, |this, _| {
                            if let Some(slot) =
                                this.item_bounds.get_mut(row_index).and_then(|bounds| bounds.get_mut(item_index))
                            {
                                *slot = bounds;
                            }
                        });
                    });
                if target_before || target_at_end {
                    let mut marker = div()
                        .id(format!("sortable-drop-marker-{}", item.id))
                        .absolute()
                        .left(px(0.0))
                        .right(px(0.0))
                        .h(px(4.0))
                        .rounded(px(2.0))
                        .bg(focus);
                    marker = if target_before {
                        marker.top(px(-3.0))
                    } else {
                        marker.bottom(px(-3.0))
                    };
                    item_view = item_view.child(marker);
                }
                items.push(
                    item_view.child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(item.accent)).child(
                        div()
                            .flex_1()
                            .child(div().text_sm().text_color(foreground).child(item.label.clone()))
                            .child(div().text_xs().text_color(muted).child(format!("{} · drag to move", item.id))),
                    ),
                );
            }
            rows.push(
                div()
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
                    .children(items),
            );
        }

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
            .on_key_down(cx.listener(Self::cancel_drag))
            .child(div().max_w(px(760.0)).flex().flex_col().gap(px(24.0)).child(
                div().flex().flex_col().gap(px(8.0)).child(div().text_xl().font_weight(gpui::FontWeight::SEMIBOLD).text_color(foreground).child("Developer · Sortable collection")).child(
                    div().text_sm().text_color(muted).child("App-local prototype for same-row reorder, cross-row moves, insertion gaps, previews, and cancellation."),
                ),
            ).child(div().p(px(12.0)).rounded(px(8.0)).bg(card).border_1().border_color(border).child(div().text_sm().text_color(focus).child(self.last_event.clone()))).children(rows))
    }
}
