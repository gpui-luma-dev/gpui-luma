use std::sync::Arc;
use std::time::Instant;

use gpui::{
    Bounds, Context, CursorStyle, DragMoveEvent, IntoElement, KeyDownEvent, MouseButton, MouseUpEvent, Pixels, Point,
    Render, SharedString, Window, div, prelude::*, point, px,
};
use luma::controls::control_group::{ControlGroupItemRenderModel, ControlGroupItemTemplate};
use luma::controls::listbox::ListBox as SdkListBox;
use luma::infra::ElementExt;
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
    list_bounds: Vec<Bounds<Pixels>>,
    dragging: Option<SortableDrag>,
    drop_target: Option<DropTarget>,
    drag_trace: Option<DragTrace>,
    pending_trace: Option<(DragTrace, &'static str)>,
    last_event: SharedString,
    listboxes: [Option<SdkListBox>; 2],
}

impl SortableCollectionPrototype {
    pub fn new(look: Arc<ShadcnLook>) -> Self {
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
            list_bounds: Vec::new(),
            dragging: None,
            drop_target: None,
            drag_trace: None,
            pending_trace: None,
            last_event: "Ready — drag an item between lists".into(),
            listboxes: [None, None],
        }
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }

    fn begin_drag(&mut self, drag: SortableDrag, cx: &mut Context<Self>) {
        self.dragging = Some(drag.clone());
        self.drop_target = None;
        self.drag_trace = Some(DragTrace::new(drag.item_id.clone()));
        self.last_event = format!("Dragging {}", drag.item_id).into();
        cx.notify();
    }

    fn update_drop_target(&mut self, event: &DragMoveEvent<SortableDrag>, window: &mut Window, cx: &mut Context<Self>) {
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
            if bounds.is_empty() {
                return self
                    .list_bounds
                    .get(row)
                    .filter(|bounds| bounds.contains(&position))
                    .map(|_| DropTarget { row, index: 0 });
            }
            let first = bounds.first()?;
            let last = bounds.last()?;
            if position.x < first.origin.x - px(8.0)
                || position.x > first.right() + px(8.0)
                || position.y < first.origin.y - px(8.0)
                || position.y > last.bottom() + px(8.0)
            {
                return None;
            }
            let index = bounds.iter().position(|bound| position.y < bound.center().y).unwrap_or(bounds.len());
            Some(DropTarget { row, index })
        });
        if target.is_some_and(|target| target.row == session.row) {
            target = None;
        }
        cx.set_active_drag_cursor_style(
            if target.is_some() {
                CursorStyle::DragCopy
            } else {
                CursorStyle::OperationNotAllowed
            },
            window,
        );
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
        self.sync_listboxes(cx);
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

    fn sync_listboxes(&mut self, cx: &mut Context<Self>) {
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

    fn create_listbox(&mut self, row_index: usize, cx: &mut Context<Self>) -> SdkListBox {
        let prototype = cx.entity();
        let look = self.look.clone();
        let item_look = look.clone();
        let item_template: ControlGroupItemTemplate<luma::controls::listbox::ListBoxItem> =
            Arc::new(move |item: &ControlGroupItemRenderModel<'_, _>, _, _| {
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
                let item_id = item.item.id().clone();
                let item_index = item.index;
                let sibling_count = item.sibling_count;
                let bounds_host = bounds_host_seed.clone();
                let start_host = start_host_seed.clone();
                let finish_host = finish_host_seed.clone();
                let finish_host_out = finish_host_out_seed.clone();
                let drag = SortableDrag {
                    row: row_index,
                    index: item_index,
                    item_id: item_id.clone(),
                    label: item.item.label_text().clone(),
                    accent: accent_for(item_id.as_ref()),
                    look: look.clone(),
                    cursor_offset: point(px(0.0), px(0.0)),
                };
                let drop_target = prototype.read(app).drop_target;
                let target_before = drop_target == Some(DropTarget { row: row_index, index: item_index });
                let target_at_end = drop_target == Some(DropTarget { row: row_index, index: item_index + 1 })
                    && item_index + 1 == sibling_count;
                let card = look.token_color("card").unwrap_or(look.chrome().panel_background);
                let border = look.token_color("border").unwrap_or(look.chrome().border);
                let content = item_template
                    .map(|template| template(item, window, app))
                    .unwrap_or_else(|| div().into_any_element());
                let mut element = div()
                    .id(format!("sortable-listbox-item-{}", item_id))
                    .relative()
                    .w_full()
                    .min_h(px(52.0))
                    .px(px(14.0))
                    .py(px(10.0))
                    .rounded(px(look.radius(ShadcnRadius::Lg)))
                    .bg(card)
                    .border_1()
                    .border_color(border)
                    .cursor_grab()
                    .on_mouse_up(MouseButton::Left, move |event, window, cx| {
                        finish_host.update(cx, |this, cx| this.finish_drag(event, window, cx));
                    })
                    .on_mouse_up_out(MouseButton::Left, move |event, window, cx| {
                        finish_host_out.update(cx, |this, cx| this.finish_drag(event, window, cx));
                    })
                    .on_drag(drag, move |payload, cursor_offset, window, cx| {
                        start_host.update(cx, |this, cx| this.begin_drag(payload.clone(), cx));
                        cx.set_active_drag_cursor_style(CursorStyle::OperationNotAllowed, window);
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
                    })
                    .child(content);
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
                element
            },
        );
        let items = self.rows[row_index].items.iter().map(|item| {
            luma::controls::listbox::ListBoxItem::new(item.id.clone(), item.id.clone()).label(item.label.clone())
        });
        ListBox::new(format!("developer-sortable-listbox-{row_index}"))
            .look(&self.look)
            .template(self.look.listbox_template())
            .with_template_modifier(|root, _| root.min_h(px(60.0)))
            .items(items)
            .item_template(item_template)
            .item_element_template(item_element_template)
            .spawn(cx)
    }
}

impl Render for SortableCollectionPrototype {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.listboxes[0].is_none() {
            for row_index in 0..self.rows.len() {
                let listbox = self.create_listbox(row_index, cx);
                self.listboxes[row_index] = Some(listbox);
            }
        }
        self.item_bounds = self.rows.iter().map(|row| vec![Bounds::default(); row.items.len()]).collect();
        self.list_bounds = vec![Bounds::default(); self.rows.len()];
        if let Some((trace, outcome)) = self.pending_trace.take() {
            trace.print(outcome);
        }
        let background = self.look.token_color("background").unwrap_or(self.look.chrome().app_background);
        let card = self.look.token_color("card").unwrap_or(self.look.chrome().panel_background);
        let foreground = self.look.token_color("foreground").unwrap_or(self.look.chrome().body_text);
        let muted = self.look.token_color("muted-foreground").unwrap_or(self.look.chrome().muted_text);
        let border = self.look.token_color("border").unwrap_or(self.look.chrome().border);
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
                    .child(
                        div()
                            .min_h(px(60.0))
                            .on_prepaint(move |bounds, _, cx| {
                                bounds_host.update(cx, |this, _| {
                                    if let Some(slot) = this.list_bounds.get_mut(row_index) {
                                        *slot = bounds;
                                    }
                                });
                            })
                            .child(listbox.as_ref().expect("listbox initialized").clone()),
                    )
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
            .on_key_down(cx.listener(Self::cancel_drag))
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
                                    .child("Developer · Sortable collection"),
                            )
                            .child(div().text_sm().text_color(muted).child(
                                "SDK ListBox prototype for cross-list moves; same-list reorder is disabled for now.",
                            )),
                    )
                    .child(
                        div()
                            .p(px(12.0))
                            .rounded(px(8.0))
                            .bg(card)
                            .border_1()
                            .border_color(border)
                            .child(div().text_sm().text_color(foreground).child(self.last_event.clone())),
                    )
                    .child(div().flex().gap(px(16.0)).children(listboxes)),
            )
    }
}
