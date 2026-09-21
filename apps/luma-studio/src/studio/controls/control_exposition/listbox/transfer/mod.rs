//! A selection transfer example. GPUI owns the drag gesture; this
//! host validates the payload and commits transfers or reordering on drop.

mod model;

use std::{cell::Cell, rc::Rc, sync::Arc};

use gpui::{
    App, Context, Div, Entity, EntityId, FontWeight, IntoElement, Render, RenderOnce, SharedString, Stateful, Window,
    div, prelude::*, px,
};
use luma::controls::listbox::{
    ListBoxAxis, ListBoxBinding, ListBoxInput, ListBoxItemState, ListBoxScrollHandle, ListBoxVisibleItem,
};
use luma::focus::EscapeFocus;
use luma::{hstack, vstack};
use luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use model::{Side, TransferItem, TransferModel};
use super::{ITEM_CONTENT_GAP, ListBoxSampleLayout, VERTICAL_LIST_WIDTH};
use super::presentation::{ExamplePresentation, SelectionMark};
use super::super::event_stream::ControlEventStream;

const IDS: [&str; 2] = ["listbox-transfer-left", "listbox-transfer-right"];
const ROW_HEIGHT: f32 = 36.0;
const ROW_GAP: f32 = 4.0;
const VISIBLE_ROWS: f32 = 5.0;
const LAYOUT: ListBoxSampleLayout = ListBoxSampleLayout {
    item_height: ROW_HEIGHT,
    spacing: ROW_GAP,
    item_padding: 8.0,
    viewport_height: ROW_HEIGHT * VISIBLE_ROWS + ROW_GAP * (VISIBLE_ROWS - 1.0),
    inset_x: 8.0,
    inset_y: 8.0,
    radius: 8.0,
    border: 1.0,
};

#[derive(Clone)]
struct ItemDrag {
    owner: EntityId,
    source: Side,
    keys: Vec<u32>,
    ended: Rc<Cell<bool>>,
}

#[derive(Clone, Copy, Debug)]
enum DragEndReason {
    Transferred,
    Reordered,
    Unchanged,
    Rejected,
    Cancelled,
}

impl ItemDrag {
    fn accepts(&self, owner: EntityId) -> bool {
        self.owner == owner
    }
}

pub(super) struct TransferExample {
    model: TransferModel,
    bindings: [ListBoxBinding; 2],
    scrolls: [ListBoxScrollHandle<u32>; 2],
    presentations: [ExamplePresentation; 2],
}

impl TransferExample {
    pub fn new(look: Arc<ShadcnLook>, events: Entity<ControlEventStream>, cx: &mut Context<Self>) -> Self {
        Self {
            model: TransferModel::new(),
            bindings: Side::ALL.map(|_| ListBoxBinding::new(cx)),
            scrolls: Side::ALL.map(|_| ListBoxScrollHandle::default().require_focus_for_scroll(true)),
            presentations: [
                ExamplePresentation::new(look.clone(), "Left", events.clone()),
                ExamplePresentation::new(look, "Right", events),
            ],
        }
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        for presentation in &mut self.presentations {
            presentation.sync_look(look.clone(), cx);
        }
    }

    fn left_input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_input(Side::Left, input, cx);
    }

    fn right_input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_input(Side::Right, input, cx);
    }

    fn handle_input(&mut self, side: Side, input: ListBoxInput<u32>, cx: &mut Context<Self>) {
        let index = side.index();
        let update = self.model.lists[index].apply(input);
        self.scrolls[index].handle_update(&update, cx);
        self.presentations[index].record(&update.events, cx);
    }

    fn end_drag(&self, drag: &ItemDrag, reason: DragEndReason, cx: &mut Context<Self>) {
        // An accepted/rejected drop and native preview release share this guard.
        if !drag.ended.replace(true) {
            self.presentations[drag.source.index()]
                .record_message(&format!("DragEnded {{ keys: {:?}, reason: {reason:?} }}", drag.keys), cx);
        }
    }

    fn accept_drop(
        &mut self,
        target: Side,
        before: Option<u32>,
        drag: &ItemDrag,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !drag.accepts(cx.entity_id()) || drag.ended.get() {
            return;
        }
        let updates = match self.model.move_items(drag.source, target, &drag.keys, before) {
            Ok(updates) => updates,
            Err(error) => {
                self.presentations[target.index()].record_message(
                    &format!(
                        "DropRejected {{ source: {:?}, keys: {:?}, before: {before:?}, error: {error:?} }}",
                        drag.source, drag.keys
                    ),
                    cx,
                );
                self.end_drag(drag, DragEndReason::Rejected, cx);
                return;
            }
        };
        for (index, update) in updates.iter().enumerate() {
            self.scrolls[index].handle_update(update, cx);
            self.presentations[index].record(&update.events, cx);
        }
        let keys: Vec<u32> = self.model.lists[target.index()]
            .snapshot()
            .items()
            .iter()
            .filter(|item| drag.keys.contains(&item.id))
            .map(|item| item.id)
            .collect();
        let changed = updates.iter().any(|update| update.changed);
        let reason = if !changed {
            DragEndReason::Unchanged
        } else if drag.source == target {
            self.presentations[target.index()]
                .record_message(&format!("ItemsReordered {{ keys: {keys:?}, before: {before:?} }}"), cx);
            DragEndReason::Reordered
        } else {
            self.presentations[drag.source.index()]
                .record_message(&format!("ItemsRemoved {{ keys: {keys:?}, target: {target:?} }}"), cx);
            self.presentations[target.index()].record_message(
                &format!("ItemsAdded {{ keys: {keys:?}, source: {:?}, before: {before:?} }}", drag.source),
                cx,
            );
            DragEndReason::Transferred
        };
        self.presentations[target.index()].record_message(
            &format!(
                "Dropped {{ source: {:?}, target: {target:?}, keys: {keys:?}, before: {before:?}, changed: {changed} }}",
                drag.source,
            ),
            cx,
        );
        self.end_drag(drag, reason, cx);
        self.bindings[target.index()].focus_handle().focus(window, cx);
    }

    fn render_item(
        &self,
        side: Side,
        item: ListBoxVisibleItem<'_, TransferItem, u32>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> TransferRow {
        let index = side.index();
        let look = self.presentations[index].look.clone();
        let label = item.item.label.clone();
        let surface = look
            .listbox_row((IDS[index], item.key as u64), item.state, self.bindings[index].focus_visible(window))
            .aria_label(label.clone());
        let on_input = match side {
            Side::Left => Self::left_input,
            Side::Right => Self::right_input,
        };
        let owner = cx.entity().downgrade();
        let surface = self.bindings[index].bind_row(surface, item.key, item.state, cx, on_input).cursor_grab().on_drag(
            ItemDrag {
                owner: cx.entity_id(),
                source: side,
                keys: self.model.drag_keys(side, item.key),
                ended: Rc::default(),
            },
            move |drag, _, _, cx| {
                // Drag start leaves the source snapshot and selection untouched.
                let _ = owner.update(cx, |this, cx| {
                    this.presentations[side.index()]
                        .record_message(&format!("DragStarted {{ keys: {:?} }}", drag.keys), cx);
                });
                let label = if drag.keys.len() == 1 {
                    label.clone()
                } else {
                    format!("{} items", drag.keys.len()).into()
                };
                let preview = cx.new(|_| DragPreview { look: look.clone(), label });
                let owner = owner.clone();
                let drag = drag.clone();
                // GPUI releases the preview after Escape, outside release, or a
                // drop. This covers cancellation without polling or a second drag loop.
                cx.observe_release(&preview, move |_, cx| {
                    let _ = owner.update(cx, |this, cx| this.end_drag(&drag, DragEndReason::Cancelled, cx));
                })
                .detach();
                preview
            },
        );
        TransferRow { surface, label: item.item.label.clone(), selected: item.state.selected }
    }

    fn insertion_target(&self, side: Side, before: Option<u32>, above: bool, cx: &mut Context<Self>) -> Stateful<Div> {
        let owner = cx.entity_id();
        let look = &self.presentations[side.index()].look;
        let highlight = look.token_color("ring").unwrap_or(look.chrome().border);
        // Each half-row targets an adjacent gap. The lower half also covers
        // the spacing before the next row; these overlays only exist while dragging.
        let extension = if !above && before.is_some() { ROW_GAP } else { 0.0 };
        div()
            .id(if above { "insert-before" } else { "insert-after" })
            .absolute()
            .left_0()
            .right_0()
            .h(px(ROW_HEIGHT / 2.0 + extension))
            .when(above, |zone| zone.top_0().border_t_2())
            // Keep two pixels of the marker inside the row even when the gap
            // below it is clipped at the viewport's lower edge.
            .when(!above, |zone| zone.bottom(px(-extension)).border_b(px(extension + 2.0)))
            .border_color(gpui::transparent_black())
            .can_drop(move |value, _, _| value.downcast_ref::<ItemDrag>().is_some_and(|drag| drag.accepts(owner)))
            .drag_over::<ItemDrag>(move |style, drag, _, _| {
                if drag.accepts(owner) {
                    style.border_color(highlight)
                } else {
                    style
                }
            })
            .on_drop(
                cx.listener(move |this, drag: &ItemDrag, window, cx| this.accept_drop(side, before, drag, window, cx)),
            )
    }

    fn render_list(&mut self, side: Side, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let index = side.index();
        let owner = cx.entity_id();
        let rows = self.model.lists[index]
            .visible_items()
            .map(|item| {
                let key = item.key;
                let next = self.model.lists[index].snapshot().items().get(item.visible_index + 1).map(|item| item.id);
                // One measured child per item keeps SDK scrolling/reveal aligned.
                div()
                    .id((IDS[index], key as u64))
                    .relative()
                    .w_full()
                    .min_w(px(0.0))
                    .h(px(ROW_HEIGHT))
                    .flex_shrink_0()
                    .child(self.render_item(side, item, window, cx))
                    .when(cx.has_active_drag(), |row| {
                        row.child(self.insertion_target(side, Some(key), true, cx))
                            .child(self.insertion_target(side, next, false, cx))
                    })
            })
            .collect::<Vec<_>>();
        let stack = vstack! {}
            .id(format!("{}-viewport", IDS[index]))
            .overflow_y_scroll()
            .w_full()
            .min_w(px(0.0))
            .h(px(LAYOUT.viewport_height))
            .gap(px(LAYOUT.spacing))
            .aria_label(if side == Side::Left {
                "Left transfer list"
            } else {
                "Right transfer list"
            })
            .children(rows);
        let on_input = match side {
            Side::Left => Self::left_input,
            Side::Right => Self::right_input,
        };
        let stack = self.scrolls[index]
            .bind_drag_auto_scroll(stack, ListBoxAxis::Vertical, move |drag: &ItemDrag| drag.accepts(owner));
        let presentation = &self.presentations[index];
        let highlight = presentation.look.token_color("ring").unwrap_or(presentation.look.chrome().border);
        let surface = presentation
            .surface(IDS[index], LAYOUT, self.model.lists[index].is_focused())
            .relative()
            .can_drop(move |value, _, _| value.downcast_ref::<ItemDrag>().is_some_and(|drag| drag.accepts(owner)))
            .drag_over::<ItemDrag>(move |style, drag, _, _| {
                if drag.accepts(owner) {
                    style.border_color(highlight)
                } else {
                    style
                }
            })
            .on_drop(
                cx.listener(move |this, drag: &ItemDrag, window, cx| this.accept_drop(side, None, drag, window, cx)),
            )
            .when(self.model.lists[index].snapshot().items().is_empty(), |surface| {
                surface.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(presentation.look.chrome().muted_text)
                        .child("Drop here"),
                )
            });
        let surface = self.scrolls[index].bind(surface, stack, &self.model.lists[index]);
        let surface = self.bindings[index].bind_root(
            surface,
            ListBoxAxis::Vertical,
            self.model.lists[index].selection_mode(),
            window,
            cx,
            on_input,
        );
        vstack! { gap=4.0;
            presentation.section(self.model.lists[index].selected_keys().count(), surface),
            div().typography_style(presentation.look.typography_scale(ShadcnTextSize::Xs))
                .text_color(presentation.look.chrome().muted_text)
                .child(format!("{} items", self.model.lists[index].snapshot().items().len())),
        }
        .w(px(VERTICAL_LIST_WIDTH))
        .flex_shrink_0()
    }
}

impl Render for TransferExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let left = self.render_list(Side::Left, window, cx);
        let right = self.render_list(Side::Right, window, cx);
        let look = &self.presentations[0].look;
        vstack! { gap=8.0;
            div().typography_style(look.typography_scale(ShadcnTextSize::Sm)).font_weight(FontWeight::SEMIBOLD)
                .text_color(look.chrome().title_text).child("Drag and drop"),
            div().typography_style(look.typography_scale(ShadcnTextSize::Xs)).text_color(look.chrome().muted_text)
                .child("Click or Space toggles selection. Drag within or between lists. A selected row moves the selection; an unselected row moves alone. Hold near the top or bottom edge to scroll. Drop at the line. Escape cancels."),
            hstack! { gap=12.0; left, right }.w_full().min_w(px(0.0)),
        }
        .id("listbox-transfer-example")
        .w_full()
        .min_w(px(0.0))
        .on_action(|_: &EscapeFocus, window, cx| {
            if cx.stop_active_drag(window) {
                cx.stop_propagation();
            } else {
                cx.propagate();
            }
        })
    }
}

#[derive(IntoElement)]
struct TransferRow {
    surface: Stateful<Div>,
    label: SharedString,
    selected: bool,
}

impl RenderOnce for TransferRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.surface
            .flex()
            .items_center()
            .gap(px(ITEM_CONTENT_GAP))
            .flex_shrink_0()
            .w_full()
            .min_w(px(0.0))
            .h(px(LAYOUT.item_height))
            .px(px(LAYOUT.item_padding))
            .rounded(px(LAYOUT.radius))
            .overflow_hidden()
            .child(SelectionMark { selected: self.selected })
            .child(div().min_w(px(0.0)).truncate().child(self.label))
    }
}

struct DragPreview {
    look: Arc<ShadcnLook>,
    label: SharedString,
}

impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let surface = self.look.listbox_row(
            "transfer-drag-preview",
            ListBoxItemState { selected: true, active: false, enabled: true },
            false,
        );
        div()
            .w(px(136.0))
            .opacity(0.9)
            .child(TransferRow { surface, label: self.label.clone(), selected: true })
    }
}
