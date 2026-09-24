//! Shadcn composition of SDK ListBox state, bindings, and optional DnD.
//!
//! Keep the state, binding, and scroll handle for the lifetime of the host. Build
//! the surface each render; rebuilding must not reset focus or scroll position.
//! Item content and drop commits remain application choices. This is deliberately
//! a borrowed `.build(window, cx)` API, not a second entity owning the collection.

#[cfg(all(test, feature = "test-support"))]
#[path = "listbox_builder_tests.rs"]
mod tests;

#[cfg(all(test, feature = "test-support"))]
#[path = "listbox_virtualization_tests.rs"]
mod virtualization_tests;

#[cfg(all(test, feature = "test-support"))]
#[path = "listbox_measured_tests.rs"]
mod measured_tests;

use std::hash::Hash;

use gpui::{
    AnyElement, App, Axis, Context, Div, ElementId, Entity, EntityId, Hsla, Render, Stateful, Window, div, prelude::*,
    px,
};
use luma::controls::listbox::{
    ListBoxAxis, ListBoxBinding, ListBoxControl, ListBoxFlow, ListBoxInput, ListBoxItemRenderModel, ListBoxLayout,
    ListBoxScrollHandle, ListBoxState, ListBoxVisibleItem, ListBoxVirtualization,
};
use luma::infra::drag_drop::{
    DragDropElementExt, DragDropEvent, DropEdge, DropProposal, DropZone, KeyedDrag, KeyedDropTarget, bind_drag_source,
};
use luma::vstack;
use crate::look::{ShadcnLook, resolve_look_from};

const CORNER_RADIUS: f32 = 8.0;

type Surface = Stateful<Div>;
type InputHandler<M, K> = fn(&mut M, ListBoxInput<K>, &mut Window, &mut Context<M>);
type ItemRenderer<'a, T, K> = Box<dyn Fn(Surface, ListBoxVisibleItem<'_, T, K>, &mut App) -> AnyElement + 'a>;
type DragRow<'a, M, T, K> = Box<dyn Fn(Surface, &T, &K, &mut Context<M>) -> Surface + 'a>;
type DropTarget<'a, M, K> = Box<dyn Fn(Surface, Option<K>, &mut Context<M>) -> Surface + 'a>;

struct DragDropBindings<'a, M: 'static, T, K> {
    row: DragRow<'a, M, T, K>,
    target: DropTarget<'a, M, K>,
    viewport: Box<dyn Fn(Surface, EntityId, ListBoxAxis) -> Surface + 'a>,
    highlight: Box<dyn Fn(Surface, EntityId, Hsla) -> Surface + 'a>,
}

/// Look-owned fixed-size or content-sized list composition. Reuse SDK state and handles between renders.
/// No DnD handlers or gap overlays are installed unless `drag_and_drop` is used.
pub struct ListBoxBuilder<'a, M: 'static, T, K> {
    id: &'static str,
    look: Option<ShadcnLook>,
    state: &'a ListBoxState<T, K>,
    binding: &'a mut ListBoxBinding,
    scroll: &'a ListBoxScrollHandle<K>,
    layout: ListBoxLayout,
    axis: ListBoxAxis,
    item_width: f32,
    content_sized: bool,
    virtualization: ListBoxVirtualization,
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
        item: impl Fn(Surface, ListBoxVisibleItem<'_, T, K>, &mut App) -> E + 'a,
    ) -> Self {
        Self {
            id,
            state,
            binding,
            scroll,
            on_input,
            item_id,
            item: Box::new(move |surface, item_state, cx| item(surface, item_state, cx).into_any_element()),
            axis: ListBoxAxis::Vertical,
            item_width: 136.0,
            content_sized: false,
            virtualization: ListBoxVirtualization::default(),
            look: None,
            layout: ListBoxLayout {
                item_height: 36.0,
                spacing: 4.0,
                viewport_height: 196.0,
                inset_x: 8.0,
                inset_y: 8.0,
            },
            label: id,
            empty: None,
            drag_drop: None,
        }
    }

    /// Bind an explicit look, including live theme changes.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    /// Set SDK geometry; appearance remains owned by this look.
    pub fn layout(mut self, layout: ListBoxLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Fixed-width cards in an available-width horizontal viewport.
    pub fn horizontal(mut self, item_width: f32) -> Self {
        self.axis = ListBoxAxis::Horizontal;
        self.item_width = item_width;
        self
    }

    /// Let vertical row templates determine their height at the available width.
    /// Use an explicit viewport height in `layout`. Templates must size naturally
    /// rather than fill an unspecified height. Horizontal flow remains fixed-size.
    pub fn content_sized(mut self) -> Self {
        self.content_sized = true;
        self
    }

    /// Opt into uniform windowing on either axis or measured vertical windowing.
    /// Measured requires `content_sized`; incompatible policies fall back to eager.
    /// Templates run only for viewport/buffer items and may still borrow local
    /// data. Durable item state belongs in host-owned models or entities.
    pub fn virtualization(mut self, policy: ListBoxVirtualization) -> Self {
        self.virtualization = policy;
        self
    }

    /// Accessible name of the list surface. Defaults to its ID.
    pub fn aria_label(mut self, label: &'static str) -> Self {
        self.label = label;
        self
    }

    /// Content centered over an empty viewport.
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
            viewport: Box::new(move |surface, scope, axis| {
                scroll.bind_drag_auto_scroll(surface, axis, move |drag: &KeyedDrag<S, K>| drag.accepts(scope))
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

    /// Assemble one frame using the existing SDK bindings and scroll handle.
    pub fn build(self, window: &mut Window, cx: &mut Context<M>) -> Surface {
        let content_sized = self.content_sized && self.axis == ListBoxAxis::Vertical;
        let scope = cx.entity_id();
        let look = resolve_look_from(self.look.as_ref(), cx);
        let highlight = look.token_color("ring").unwrap_or(look.chrome().border);
        let insertion_target = |before: Option<K>, edge: DropEdge, cx: &mut Context<M>| {
            let zone = DropZone {
                axis: match self.axis {
                    ListBoxAxis::Vertical => Axis::Vertical,
                    ListBoxAxis::Horizontal => Axis::Horizontal,
                },
                edge,
                item_extent: px(match self.axis {
                    ListBoxAxis::Vertical => self.layout.item_height,
                    ListBoxAxis::Horizontal => self.item_width,
                }),
                following_gap: px(if before.is_some() { self.layout.spacing } else { 0.0 }),
                marker_width: px(2.0),
            };
            let target = div().id(if edge == DropEdge::Before {
                "insert-before"
            } else {
                "insert-after"
            });
            let mut surface = if content_sized {
                zone.apply_measured(target)
            } else {
                zone.apply(target)
            }
            .border_color(gpui::transparent_black());
            if let Some(drag_drop) = &self.drag_drop {
                surface = (drag_drop.highlight)(surface, scope, highlight);
                surface = (drag_drop.target)(surface, before, cx);
            }
            surface
        };
        let virtual_window = if content_sized {
            let (estimate, buffer) = match self.virtualization {
                ListBoxVirtualization::Measured { estimated_height, overscan } => (estimated_height, Some(overscan)),
                _ => (48.0, None),
            };
            Some(self.scroll.content_window(self.state, estimate, self.layout.spacing, buffer))
        } else {
            match self.virtualization {
                ListBoxVirtualization::Eager | ListBoxVirtualization::Measured { .. } => None,
                ListBoxVirtualization::Uniform { overscan } => self.scroll.virtual_window(
                    self.state,
                    self.axis,
                    match self.axis {
                        ListBoxAxis::Vertical => self.layout.item_height,
                        ListBoxAxis::Horizontal => self.item_width,
                    },
                    self.layout.spacing,
                    overscan,
                ),
            }
        };
        let range = virtual_window
            .as_ref()
            .map_or(0..self.state.snapshot().items().len(), |window| window.range.clone());
        // `nth` on the snapshot's mapped slice iterator skips directly to the
        // range; off-screen items do not construct models, templates, or bindings.
        let items = self.state.visible_items().skip(range.start).take(range.len());
        let spacer = |extent: f32| {
            div()
                .flex_shrink_0()
                .when(self.axis == ListBoxAxis::Vertical, |space| space.h(px(extent)).w_full())
                .when(self.axis == ListBoxAxis::Horizontal, |space| space.w(px(extent)).h(px(self.layout.item_height)))
        };
        let mut rows = Vec::with_capacity(range.len() + 2);
        if let Some(extent) = virtual_window.as_ref().and_then(|window| window.leading_space) {
            rows.push(spacer(extent).into_any_element());
        }
        for item in items {
            let key = item.key.clone();
            let visible_index = item.visible_index;
            let surface = look.listbox_row((self.item_id)(&key), item.state, self.binding.focus_visible(window));
            let mut surface = self.binding.bind_row(surface, key.clone(), item.state, cx, self.on_input);
            if let Some(drag_drop) = &self.drag_drop {
                surface = (drag_drop.row)(surface, item.item, &key, cx);
            }
            // Stable keyed wrappers survive window movement; spacers have no bindings.
            let mut row = div()
                .id((self.item_id)(&key))
                .debug_selector(|| format!("{}-item-{visible_index}", self.id))
                .relative()
                .when(self.axis == ListBoxAxis::Vertical, |row| row.w_full())
                .when(self.axis == ListBoxAxis::Horizontal, |row| row.w(px(self.item_width)))
                .min_w(px(0.0))
                .when(!content_sized, |row| row.h(px(self.layout.item_height)))
                .when(content_sized, |row| row.min_h(px(1.0)))
                .flex_shrink_0()
                .child((self.item)(surface, item, cx));
            if self.drag_drop.is_some() && cx.has_active_drag() {
                row = row.child(insertion_target(Some(key), DropEdge::Before, cx)).child(insertion_target(
                    self.state.visible_items().nth(visible_index + 1).map(|item| item.key),
                    DropEdge::After,
                    cx,
                ));
            }
            rows.push(row.into_any_element());
        }
        if let Some(extent) = virtual_window.as_ref().and_then(|window| window.trailing_space) {
            rows.push(spacer(extent).into_any_element());
        }
        let mut viewport = vstack! {}
            .id(format!("{}-viewport", self.id))
            .debug_selector(|| format!("{}-viewport", self.id))
            .when(self.axis == ListBoxAxis::Vertical, |viewport| viewport.overflow_y_scroll())
            .when(self.axis == ListBoxAxis::Horizontal, |viewport| viewport.flex_row().overflow_x_scroll())
            .w_full()
            .min_w(px(0.0))
            .h(px(self.layout.viewport_height))
            .gap(px(self.layout.spacing))
            .children(rows);
        if let Some(drag_drop) = &self.drag_drop {
            viewport = (drag_drop.viewport)(viewport, scope, self.axis);
        }
        let mut surface = look
            .listbox_surface(format!("{}-surface", self.id), self.state.is_focused())
            .debug_selector(|| format!("{}-surface", self.id))
            .relative()
            .w_full()
            .min_w(px(0.0))
            .rounded(px(CORNER_RADIUS))
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
        let surface = match virtual_window {
            Some(rendered) => self.scroll.bind_virtualized(surface, viewport, self.state, rendered),
            None => self.scroll.bind(surface, viewport, self.state),
        };
        self.binding.bind_root(surface, self.axis, self.state.selection_mode(), window, cx, self.on_input)
    }
}

impl ShadcnLook {
    /// Render a persistent SDK control with this look and an arbitrary content
    /// template. The callback may borrow render-local data; it is not retained.
    /// Keep the control across renders so focus and scroll position remain stable.
    #[allow(clippy::too_many_arguments)]
    pub fn render_listbox<M: 'static, T: 'static, K: Clone + Eq + Hash + 'static, E: IntoElement>(
        &self,
        control: &mut ListBoxControl<M, T, K>,
        id: &'static str,
        flow: ListBoxFlow,
        padding: (f32, f32),
        template: impl Fn(&ListBoxItemRenderModel<'_, T>, &mut App) -> E,
        window: &mut Window,
        cx: &mut Context<M>,
    ) -> Stateful<Div> {
        let parts = control.render_parts();
        let layout = flow.layout(padding);
        let content_sized = matches!(flow, ListBoxFlow::VerticalContent { .. });
        let item_label = parts.item_label;
        let mode = parts.state.selection_mode();
        let builder = ListBoxBuilder::new(
            id,
            parts.state,
            parts.binding,
            parts.scroll,
            parts.on_input,
            parts.item_id,
            |surface, item, cx| {
                let model = ListBoxItemRenderModel {
                    item: item.item,
                    selected: item.state.selected,
                    active: item.state.active,
                    enabled: item.state.enabled,
                };
                surface
                    .w_full()
                    .when(!content_sized, |surface| surface.h_full())
                    .rounded(px(CORNER_RADIUS))
                    .overflow_hidden()
                    .aria_label(item_label(item.item))
                    .when(!item.state.enabled, |surface| surface.aria_description("Disabled"))
                    .child(template(&model, cx))
            },
        )
        .look(self)
        .layout(layout)
        .virtualization(parts.virtualization);
        let builder = match flow {
            ListBoxFlow::Vertical { .. } => builder,
            ListBoxFlow::VerticalContent { .. } => builder.content_sized(),
            ListBoxFlow::Horizontal { item_width, .. } => builder.horizontal(item_width),
        };
        builder.build(window, cx).aria_description(format!("Selection mode: {mode:?}"))
    }
}
