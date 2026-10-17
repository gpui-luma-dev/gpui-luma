use gpui::{
    App, Axis, Context, Div, Entity, EntityId, MouseButton, Pixels, Point, Render, Stateful, Window, prelude::*, px,
    relative,
};
use super::{DragDropEvent, DropProposal, KeyedDrag};

type EventHandler<M, S, K> = fn(&mut M, DragDropEvent<S, K>, &mut Context<M>);

/// Attach a native drag to a host-styled surface. The host creates the preview;
/// the SDK reports start and observes preview release for cancellation. GPUI
/// still owns threshold detection and stopping the active drag (including Escape).
/// Do not attach another on_drag handler to this surface.
pub fn bind_drag_source<M: 'static, S: Clone + 'static, K: Clone + 'static, P: Render + 'static>(
    surface: Stateful<Div>,
    drag: KeyedDrag<S, K>,
    preview: impl Fn(&KeyedDrag<S, K>, Point<Pixels>, &mut Window, &mut App) -> Entity<P> + 'static,
    cx: &mut Context<M>,
    on_event: EventHandler<M, S, K>,
) -> Stateful<Div> {
    let owner = cx.entity().downgrade();
    surface.on_drag(drag, move |drag, offset, window, cx| {
        if let Some(event) = drag.start() {
            let _ = owner.update(cx, |owner, cx| on_event(owner, event, cx));
        }
        let preview = preview(drag, offset, window, cx);
        let owner = owner.clone();
        let drag = drag.clone();
        cx.observe_release(&preview, move |_, cx| {
            if let Some(event) = drag.cancel() {
                let _ = owner.update(cx, |owner, cx| on_event(owner, event, cx));
            }
        })
        .detach();
        preview
    })
}

/// Shared scope and keyed gap admission. The host supplies geometry, highlight
/// styling, and final validation/mutation; foreign or ended sessions are ignored.
#[derive(Clone)]
pub struct KeyedDropTarget<S, K> {
    scope: EntityId,
    target: S,
    before: Option<K>,
}

impl<S: Clone + 'static, K: Clone + 'static> KeyedDropTarget<S, K> {
    pub fn new(scope: EntityId, target: S, before: Option<K>) -> Self {
        Self { scope, target, before }
    }

    pub fn accepts(&self, drag: &KeyedDrag<S, K>) -> bool {
        drag.accepts(self.scope)
    }

    pub fn bind<M: 'static>(
        &self,
        surface: Stateful<Div>,
        cx: &mut Context<M>,
        on_drop: impl Fn(&mut M, DropProposal<S, K>, &mut Window, &mut Context<M>) + 'static,
    ) -> Stateful<Div> {
        let scope = self.scope;
        let target = self.target.clone();
        let before = self.before.clone();
        surface
            .can_drop(move |value, _, _| {
                value.downcast_ref::<KeyedDrag<S, K>>().is_some_and(|drag| drag.accepts(scope))
            })
            .on_drop(cx.listener(move |owner, drag: &KeyedDrag<S, K>, window, cx| {
                if let Some(proposal) = drag.propose(scope, target.clone(), before.clone()) {
                    on_drop(owner, proposal, window, cx);
                }
            }))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DropEdge {
    Before,
    After,
}

/// Half-item hit area for a host's insertion marker. Attach as an absolute child
/// of a relative item wrapper while dragging, keeping one measured child per item.
/// The After area includes the following gap. Border width keeps the marker
/// visible inside the item when the gap is clipped by the viewport.
#[derive(Clone, Copy, Debug)]
pub struct DropZone {
    pub axis: Axis,
    pub edge: DropEdge,
    pub item_extent: Pixels,
    pub following_gap: Pixels,
    pub marker_width: Pixels,
}

impl DropZone {
    pub fn apply(self, surface: Stateful<Div>) -> Stateful<Div> {
        let (extent, extension, marker) = self.dimensions();
        let surface = surface.absolute();
        match (self.axis, self.edge) {
            (Axis::Vertical, DropEdge::Before) => surface.left_0().right_0().top_0().h(extent).border_t(marker),
            (Axis::Vertical, DropEdge::After) => {
                surface.left_0().right_0().bottom(-extension).h(extent).border_b(marker)
            }
            (Axis::Horizontal, DropEdge::Before) => surface.top_0().bottom_0().left_0().w(extent).border_l(marker),
            (Axis::Horizontal, DropEdge::After) => {
                surface.top_0().bottom_0().right(-extension).w(extent).border_r(marker)
            }
        }
    }

    /// Use the parent's actual laid-out size for the two row halves. Suitable
    /// for content-sized items; `item_extent` is ignored. Gap/marker behavior
    /// matches `apply`, including clipped trailing gaps.
    pub fn apply_measured(self, surface: Stateful<Div>) -> Stateful<Div> {
        let (_, extension, marker) = self.dimensions();
        let surface = surface.absolute();
        match (self.axis, self.edge) {
            (Axis::Vertical, DropEdge::Before) => {
                surface.left_0().right_0().top_0().bottom(relative(0.5)).border_t(marker)
            }
            (Axis::Vertical, DropEdge::After) => {
                surface.left_0().right_0().top(relative(0.5)).bottom(-extension).border_b(marker)
            }
            (Axis::Horizontal, DropEdge::Before) => {
                surface.top_0().bottom_0().left_0().right(relative(0.5)).border_l(marker)
            }
            (Axis::Horizontal, DropEdge::After) => {
                surface.top_0().bottom_0().left(relative(0.5)).right(-extension).border_r(marker)
            }
        }
    }

    pub(super) fn dimensions(self) -> (Pixels, Pixels, Pixels) {
        let nonnegative = |value: Pixels| {
            if value.as_f32().is_finite() {
                value.max(px(0.0))
            } else {
                px(0.0)
            }
        };
        let extension = if self.edge == DropEdge::After {
            nonnegative(self.following_gap)
        } else {
            px(0.0)
        };
        (
            nonnegative(self.item_extent) / 2.0 + extension,
            extension,
            nonnegative(self.marker_width) + extension,
        )
    }
}

/// Wrap an embedded interactive control in a stateful element with this boundary.
/// Bubble handlers let the child receive its own input first, then stop mouse-down
/// from arming an ancestor row's drag and stop clicks from selecting that row.
/// Child-owned native drags and keyboard input remain available.
pub trait DragDropElementExt: StatefulInteractiveElement + Sized {
    /// Cancel a native drag through the SDK Escape action. When no drag is
    /// active, let the enclosing focus scope handle Escape normally.
    fn cancel_drag_on_escape(self) -> Self {
        self.on_action(|_: &crate::focus::EscapeFocus, window, cx| {
            if cx.stop_active_drag(window) {
                cx.stop_propagation();
            } else {
                cx.propagate();
            }
        })
    }

    fn drag_boundary(self) -> Self {
        self.on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(|_, _, cx| cx.stop_propagation())
    }
}
impl<E: StatefulInteractiveElement> DragDropElementExt for E {}

/// Native edge scrolling for a scoped keyed strip or viewport. Session admission
/// is rechecked each frame; release, cancellation and leaving the viewport stop it.
pub(crate) fn bind_edge_scroll<S: Clone + 'static, K: Clone + 'static>(
    surface: Stateful<Div>,
    handle: gpui::ScrollHandle,
    axis: Axis,
    scope: EntityId,
    pending: std::rc::Rc<std::cell::Cell<bool>>,
) -> Stateful<Div> {
    surface.on_drag_move(move |event: &gpui::DragMoveEvent<KeyedDrag<S, K>>, window, cx| {
        if !pending.replace(true) {
            scroll_frame(handle.clone(), axis, scope, event.drag(cx).clone(), pending.clone(), window, cx);
        }
    })
}
fn scroll_frame<S: Clone + 'static, K: Clone + 'static>(
    handle: gpui::ScrollHandle,
    axis: Axis,
    scope: EntityId,
    drag: KeyedDrag<S, K>,
    pending: std::rc::Rc<std::cell::Cell<bool>>,
    window: &mut Window,
    _: &mut App,
) {
    window.on_next_frame(move |window, cx| {
        pending.set(false);
        let bounds = handle.bounds();
        let pointer = window.mouse_position();
        if !drag.accepts(scope) || !cx.has_active_drag() || !window.is_window_hovered() || !bounds.contains(&pointer) {
            return;
        }
        let (position, extent, offset, max) = match axis {
            Axis::Horizontal => (
                (pointer.x - bounds.left()).as_f32(),
                bounds.size.width.as_f32(),
                handle.offset().x,
                handle.max_offset().x,
            ),
            Axis::Vertical => (
                (pointer.y - bounds.top()).as_f32(),
                bounds.size.height.as_f32(),
                handle.offset().y,
                handle.max_offset().y,
            ),
        };
        let delta = super::edge_scroll_delta(position, extent, std::time::Duration::from_secs_f32(1. / 60.), 540.);
        let next = (-offset.as_f32() + delta).clamp(0., max.as_f32().max(0.));
        if (next + offset.as_f32()).abs() < 0.01 {
            return;
        }
        let mut point = handle.offset();
        match axis {
            Axis::Horizontal => point.x = px(-next),
            Axis::Vertical => point.y = px(-next),
        }
        handle.set_offset(point);
        window.refresh();
        pending.set(true);
        scroll_frame(handle, axis, scope, drag, pending, window, cx);
    });
    window.refresh();
}
