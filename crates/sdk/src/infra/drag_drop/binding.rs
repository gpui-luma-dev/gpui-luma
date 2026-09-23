use gpui::{
    App, Axis, Context, Div, Entity, EntityId, MouseButton, Pixels, Point, Render, Stateful, Window, prelude::*, px,
};
use super::{DragDropEvent, DropProposal, KeyedDrag};

type EventHandler<M, S, K> = fn(&mut M, DragDropEvent<S, K>, &mut Context<M>);
type DropHandler<M, S, K> = fn(&mut M, DropProposal<S, K>, &mut Window, &mut Context<M>);

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
        on_drop: DropHandler<M, S, K>,
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
