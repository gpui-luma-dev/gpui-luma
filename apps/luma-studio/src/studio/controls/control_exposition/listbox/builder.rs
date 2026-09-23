//! Local prototype: assemble SDK ListBox bindings without owning domain data.
//!
//! Keep the state, binding, and scroll handle for the lifetime of the host. Build
//! the surface each render; rebuilding must not reset focus or scroll position.
//! Item content and drop commits remain application choices. This is deliberately
//! a borrowed `.build(window, cx)` API, not a second entity owning the collection.

#[cfg(all(test, feature = "test-support"))]
#[path = "builder_tests.rs"]
mod tests;

use std::{hash::Hash, sync::Arc};

use gpui::{
    AnyElement, App, Axis, Context, Div, ElementId, Entity, EntityId, Hsla, Render, Stateful, Window, div, prelude::*,
    px,
};
use luma::controls::listbox::{
    ListBoxAxis, ListBoxBinding, ListBoxInput, ListBoxScrollHandle, ListBoxState, ListBoxVisibleItem,
};
use luma::infra::drag_drop::{
    DragDropElementExt, DragDropEvent, DropEdge, DropProposal, DropZone, KeyedDrag, KeyedDropTarget, bind_drag_source,
};
use luma::vstack;
use luma_look_shadcn::ShadcnLook;

use super::ListBoxSampleLayout;

type Surface = Stateful<Div>;
type InputHandler<M, K> = fn(&mut M, ListBoxInput<K>, &mut Window, &mut Context<M>);
type ItemRenderer<'a, T, K> = Box<dyn Fn(Surface, ListBoxVisibleItem<'_, T, K>) -> AnyElement + 'a>;
type DragRow<'a, M, T, K> = Box<dyn Fn(Surface, &T, &K, &mut Context<M>) -> Surface + 'a>;
type DropTarget<'a, M, K> = Box<dyn Fn(Surface, Option<K>, &mut Context<M>) -> Surface + 'a>;

struct DragDropBindings<'a, M: 'static, T, K> {
    row: DragRow<'a, M, T, K>,
    target: DropTarget<'a, M, K>,
    viewport: Box<dyn Fn(Surface, EntityId) -> Surface + 'a>,
    highlight: Box<dyn Fn(Surface, EntityId, Hsla) -> Surface + 'a>,
}

/// Fixed-height vertical flow prototype, currently exercised by the DnD pair.
/// No DnD handlers or gap overlays are installed unless `drag_and_drop` is used.
pub(super) struct ListBoxBuilder<'a, M: 'static, T, K> {
    id: &'static str,
    look: Option<Arc<ShadcnLook>>,
    state: &'a ListBoxState<T, K>,
    binding: &'a mut ListBoxBinding,
    scroll: &'a ListBoxScrollHandle<K>,
    layout: ListBoxSampleLayout,
    on_input: InputHandler<M, K>,
    item_id: fn(&K) -> ElementId,
    item: ItemRenderer<'a, T, K>,
    label: &'static str,
    empty: Option<AnyElement>,
    drag_drop: Option<DragDropBindings<'a, M, T, K>>,
}

impl<'a, M: 'static, T: 'static, K: Clone + Eq + Hash + 'static> ListBoxBuilder<'a, M, T, K> {
    /// Reuse the host's existing SDK state and handles. Stable item IDs must be
    /// unique within this list and remain the same when items move or reorder.
    pub fn new<E: IntoElement>(
        id: &'static str,
        state: &'a ListBoxState<T, K>,
        binding: &'a mut ListBoxBinding,
        scroll: &'a ListBoxScrollHandle<K>,
        on_input: InputHandler<M, K>,
        item_id: fn(&K) -> ElementId,
        item: impl Fn(Surface, ListBoxVisibleItem<'_, T, K>) -> E + 'a,
    ) -> Self {
        Self {
            id,
            state,
            binding,
            scroll,
            on_input,
            item_id,
            item: Box::new(move |surface, item_state| item(surface, item_state).into_any_element()),
            look: None,
            layout: ListBoxSampleLayout {
                item_height: 36.0,
                spacing: 4.0,
                item_padding: 8.0,
                viewport_height: 196.0,
                inset_x: 8.0,
                inset_y: 8.0,
                radius: 8.0,
                border: 1.0,
            },
            label: id,
            empty: None,
            drag_drop: None,
        }
    }

    pub fn look(mut self, look: Arc<ShadcnLook>) -> Self {
        self.look = Some(look);
        self
    }

    pub fn layout(mut self, layout: ListBoxSampleLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn aria_label(mut self, label: &'static str) -> Self {
        self.label = label;
        self
    }

    pub fn empty(mut self, content: impl IntoElement) -> Self {
        self.empty = Some(content.into_any_element());
        self
    }

    /// This prototype connects lists owned by the same host entity, which serves
    /// as their shared scope. The SDK owns session admission and lifecycle guards;
    /// `on_drop` validates/mutates the host data and completes the proposal.
    /// Preview construction happens only once a native drag actually starts.
    pub fn drag_and_drop<S: Clone + 'static, P: Render + 'static>(
        mut self,
        list: S,
        on_drop: fn(&mut M, DropProposal<S, K>, &mut Window, &mut Context<M>),
        on_event: fn(&mut M, DragDropEvent<S, K>, &mut Context<M>),
        preview: impl Fn(&T, &[K], &mut Window, &mut App) -> Entity<P> + 'static,
    ) -> Self
    where
        T: Clone,
    {
        let state = self.state;
        let source = list.clone();
        let preview = std::rc::Rc::new(preview);
        let scroll = self.scroll;
        // Read scope and theme at build time so fluent option order is immaterial.
        self.drag_drop = Some(DragDropBindings {
            row: Box::new(move |surface, item, key, cx| {
                let Some(drag) =
                    state.drag_keys(key).and_then(|keys| KeyedDrag::new(cx.entity_id(), source.clone(), keys))
                else {
                    return surface;
                };
                let item = item.clone();
                let preview = preview.clone();
                bind_drag_source(
                    surface.cursor_grab(),
                    drag,
                    move |drag, _, window, cx| preview(&item, drag.keys(), window, cx),
                    cx,
                    on_event,
                )
            }),
            target: Box::new(move |surface, before, cx| {
                KeyedDropTarget::new(cx.entity_id(), list.clone(), before).bind(surface, cx, on_drop)
            }),
            viewport: Box::new(move |surface, scope| {
                scroll.bind_drag_auto_scroll(surface, ListBoxAxis::Vertical, move |drag: &KeyedDrag<S, K>| {
                    drag.accepts(scope)
                })
            }),
            highlight: Box::new(move |surface, scope, color| {
                surface.drag_over::<KeyedDrag<S, K>>(move |style, drag, _, _| {
                    if drag.accepts(scope) {
                        style.border_color(color)
                    } else {
                        style
                    }
                })
            }),
        });
        self
    }

    pub fn build(self, window: &mut Window, cx: &mut Context<M>) -> Surface {
        let scope = cx.entity_id();
        let look = self.look.as_ref().cloned().unwrap_or_else(|| Arc::new(ShadcnLook::built_in()));
        let highlight = look.token_color("ring").unwrap_or(look.chrome().border);
        let insertion_target = |before: Option<K>, edge: DropEdge, cx: &mut Context<M>| {
            let zone = DropZone {
                axis: Axis::Vertical,
                edge,
                item_extent: px(self.layout.item_height),
                following_gap: px(if before.is_some() { self.layout.spacing } else { 0.0 }),
                marker_width: px(2.0),
            };
            let mut surface = zone
                .apply(div().id(if edge == DropEdge::Before {
                    "insert-before"
                } else {
                    "insert-after"
                }))
                .border_color(gpui::transparent_black());
            if let Some(drag_drop) = &self.drag_drop {
                surface = (drag_drop.highlight)(surface, scope, highlight);
                surface = (drag_drop.target)(surface, before, cx);
            }
            surface
        };
        let mut items = self.state.visible_items().peekable();
        let mut rows = Vec::with_capacity(items.len());
        while let Some(item) = items.next() {
            let key = item.key.clone();
            let surface = look.listbox_row((self.item_id)(&key), item.state, self.binding.focus_visible(window));
            let mut surface = self.binding.bind_row(surface, key.clone(), item.state, cx, self.on_input);
            if let Some(drag_drop) = &self.drag_drop {
                surface = (drag_drop.row)(surface, item.item, &key, cx);
            }
            // Exactly one measured child per visible item, including during drag.
            let mut row = div()
                .id((self.item_id)(&key))
                .relative()
                .w_full()
                .min_w(px(0.0))
                .h(px(self.layout.item_height))
                .flex_shrink_0()
                .child((self.item)(surface, item));
            if self.drag_drop.is_some() && cx.has_active_drag() {
                row = row.child(insertion_target(Some(key), DropEdge::Before, cx)).child(insertion_target(
                    items.peek().map(|item| item.key.clone()),
                    DropEdge::After,
                    cx,
                ));
            }
            rows.push(row);
        }
        let mut viewport = vstack! {}
            .id(format!("{}-viewport", self.id))
            .overflow_y_scroll()
            .w_full()
            .min_w(px(0.0))
            .h(px(self.layout.viewport_height))
            .gap(px(self.layout.spacing))
            .children(rows);
        if let Some(drag_drop) = &self.drag_drop {
            viewport = (drag_drop.viewport)(viewport, scope);
        }
        let mut surface = look
            .listbox_surface(format!("{}-surface", self.id), self.state.is_focused())
            .relative()
            .w_full()
            .min_w(px(0.0))
            .rounded(px(self.layout.radius))
            .px(px(self.layout.inset_x))
            .py(px(self.layout.inset_y))
            .aria_label(self.label);
        if let Some(drag_drop) = &self.drag_drop {
            surface = (drag_drop.highlight)(surface, scope, highlight);
            surface = (drag_drop.target)(surface, None, cx).cancel_drag_on_escape();
        }
        if self.state.snapshot().items().is_empty()
            && let Some(empty) = self.empty
        {
            surface = surface.child(div().absolute().inset_0().flex().items_center().justify_center().child(empty));
        }
        let surface = self.scroll.bind(surface, viewport, self.state);
        self.binding
            .bind_root(surface, ListBoxAxis::Vertical, self.state.selection_mode(), window, cx, self.on_input)
    }
}
