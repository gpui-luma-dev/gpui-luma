use std::cell::{Cell, RefCell};
use std::hash::Hash;
use std::rc::Rc;

use gpui::{Context, Div, DragMoveEvent, Pixels, ScrollHandle, Size, Stateful, div, prelude::*, px};

use super::{
    ListBoxAxis, ListBoxState, ListBoxUpdate,
    drag_scroll::{DragAutoScroll, DEFAULT_MAX_SPEED},
};

/// Optional scrolling adapter for a host-composed list. The host supplies the
/// styled surface and scrolling stack, with one direct child per visible item
/// in `visible_items()` order. GPUI measures the children, so either axis and
/// variable item sizes work without host-written offset calculations.
///
/// Keep one handle per list. This adapter contains wheel events at the surface
/// and keeps the focused active item visible when the viewport resizes. It does
/// not choose the axis, dimensions, spacing, item content, or appearance.
pub struct ListBoxScrollHandle<K> {
    scroll: ScrollHandle,
    pending_reveal: RefCell<Option<K>>,
    viewport_size: Rc<Cell<Size<Pixels>>>,
    drag_scroll: Rc<DragAutoScroll>,
    require_focus_for_scroll: bool,
}

impl<K> Default for ListBoxScrollHandle<K> {
    fn default() -> Self {
        Self {
            scroll: ScrollHandle::new(),
            pending_reveal: RefCell::new(None),
            viewport_size: Rc::default(),
            drag_scroll: Rc::default(),
            require_focus_for_scroll: false,
        }
    }
}

impl<K: Clone + Eq + Hash> ListBoxScrollHandle<K> {
    /// Require the existing list binding's focus before accepting wheel input.
    /// Defaults to false (hover scrolling). When true, an unfocused viewport
    /// passes wheel input to its ancestors; a focused list contains it even at
    /// either endpoint. The host must deliver the binding's focus updates.
    /// Drag auto-scroll remains available without focus. Pending item reveals
    /// wait until focus returns, preserving the viewport on focus exit.
    pub fn require_focus_for_scroll(mut self, required: bool) -> Self {
        self.require_focus_for_scroll = required;
        self
    }

    /// Set maximum drag auto-scroll speed in logical pixels per second, per list.
    /// Defaults to 540. The basic auto-scrolling function increases speed linearly
    /// with pointer depth into the edge zone, without temporal easing or inertia.
    /// Zero disables movement; negative or non-finite values restore the default.
    /// Changes are read on the next drag-scroll frame, including during a drag.
    ///
    /// ```
    /// use luma::controls::listbox::ListBoxScrollHandle;
    /// let scroll = ListBoxScrollHandle::<u32>::default();
    /// scroll.set_drag_auto_scroll_speed(720.0);
    /// ```
    pub fn set_drag_auto_scroll_speed(&self, pixels_per_second: f32) {
        let speed = if pixels_per_second.is_finite() && pixels_per_second >= 0.0 {
            pixels_per_second
        } else {
            DEFAULT_MAX_SPEED
        };
        self.drag_scroll.max_speed.set(speed);
    }

    /// Opt into continuous edge scrolling for an accepted drag payload. Attach
    /// to the same viewport passed to `bind`. The host retains drop policy;
    /// scrolling stops outside the viewport, at its limits, or when the drag ends.
    /// Configure speed with [`Self::set_drag_auto_scroll_speed`].
    /// Currently uses a basic linear auto-scrolling function; custom speed/easing
    /// functions are a future extension.
    pub fn bind_drag_auto_scroll<D: 'static>(
        &self,
        viewport: Stateful<Div>,
        axis: ListBoxAxis,
        accepts: impl Fn(&D) -> bool + 'static,
    ) -> Stateful<Div> {
        let controller = self.drag_scroll.clone();
        let scroll = self.scroll.clone();
        viewport.on_drag_move(move |event: &DragMoveEvent<D>, window, cx| {
            controller.accepted.set(accepts(event.drag(cx)));
            controller.schedule(scroll.clone(), axis, window);
        })
    }

    /// Deliver rendering and reveal effects after committing an update to the
    /// model. The host still owns event delivery and application reactions.
    pub fn handle_update<M: 'static>(&self, update: &ListBoxUpdate<K>, cx: &mut Context<M>) {
        self.queue_reveal(update);
        if update.changed || update.reveal.is_some() {
            cx.notify();
        }
    }

    /// Attach the scrolling stack inside its surface. Resolve reveal keys at
    /// render time against the current snapshot, not against stale indices.
    pub fn bind<T>(
        &self,
        surface: Stateful<Div>,
        viewport: Stateful<Div>,
        state: &ListBoxState<T, K>,
    ) -> Stateful<Div> {
        let (viewport, accepts_wheel) = self.bind_viewport(viewport, state);
        let active_index = state.active_key().filter(|_| state.is_focused()).and_then(|key| state.visible_index(key));
        let scroll = self.scroll.clone();
        let viewport_size = self.viewport_size.clone();
        let measured_viewport = div()
            .w_full()
            .min_w(px(0.0))
            .on_children_prepainted(move |bounds, window, _cx| {
                let Some(bounds) = bounds.first() else { return };
                let previous = viewport_size.replace(bounds.size);
                if let Some(index) = resize_reveal(previous, bounds.size, active_index) {
                    // The child has recorded its new viewport bounds. Let GPUI
                    // reveal against those bounds on the following frame.
                    scroll.scroll_to_item(index);
                    let view = window.current_view();
                    window.on_next_frame(move |_, cx| cx.notify(view));
                }
            })
            .child(viewport);
        surface
            // Bubble after the child scrolls, including at either endpoint.
            .when(accepts_wheel, |surface| surface.on_scroll_wheel(|_, _, cx| cx.stop_propagation()))
            .child(measured_viewport)
    }

    fn bind_viewport<T>(&self, viewport: Stateful<Div>, state: &ListBoxState<T, K>) -> (Stateful<Div>, bool) {
        let accepts_wheel = !self.require_focus_for_scroll || state.is_focused();
        if accepts_wheel && let Some(index) = self.take_reveal_index(state) {
            self.scroll.scroll_to_item(index);
        }
        // Hidden overflow keeps clipping, measured bounds, and the tracked
        // offset, but GPUI applies no wheel delta on either axis. Retaining
        // the handle also lets drag auto-scroll move an unfocused target.
        (
            viewport.when(!accepts_wheel, |viewport| viewport.overflow_hidden()).track_scroll(&self.scroll),
            accepts_wheel,
        )
    }

    fn queue_reveal(&self, update: &ListBoxUpdate<K>) {
        if let Some(key) = &update.reveal {
            *self.pending_reveal.borrow_mut() = Some(key.clone());
        }
    }

    fn take_reveal_index<T>(&self, state: &ListBoxState<T, K>) -> Option<usize> {
        self.pending_reveal.borrow_mut().take().and_then(|key| state.visible_index(&key))
    }
}

fn resize_reveal(previous: Size<Pixels>, current: Size<Pixels>, active_index: Option<usize>) -> Option<usize> {
    (previous != current).then_some(active_index).flatten()
}

#[cfg(test)]
mod tests {
    use gpui::size;
    use super::*;
    use super::super::{ListBoxInput, ListBoxNavigation, ListBoxSnapshot, SelectionMode};

    #[test]
    fn focus_policy_controls_both_axes_and_preserves_offset_and_pending_reveal() {
        for horizontal in [false, true] {
            for require_focus in [false, true] {
                let scroll = ListBoxScrollHandle::<u32>::default().require_focus_for_scroll(require_focus);
                let mut state = ListBoxState::try_new([1, 2, 3], |item| *item, SelectionMode::Multiple).unwrap();
                let offset = gpui::point(px(-40.0), px(-80.0));
                scroll.scroll.set_offset(offset);
                scroll.queue_reveal(&state.apply(ListBoxInput::Navigate(ListBoxNavigation::Last)));

                // Enter, leave, and re-enter focus without changing the host's
                // scroll axis or discarding the tracked position.
                for focused in [false, true, false, true] {
                    state.apply(ListBoxInput::Focus(focused));
                    let viewport = div().id("viewport");
                    let viewport = if horizontal {
                        viewport.overflow_x_scroll()
                    } else {
                        viewport.overflow_y_scroll()
                    };
                    let (mut viewport, accepts_wheel) = scroll.bind_viewport(viewport, &state);
                    assert_eq!(accepts_wheel, !require_focus || focused);
                    let overflow = &viewport.interactivity().base_style.overflow;
                    if accepts_wheel {
                        assert_eq!(if horizontal { overflow.x } else { overflow.y }, Some(gpui::Overflow::Scroll));
                        assert!(scroll.pending_reveal.borrow().is_none());
                    } else {
                        assert_eq!(overflow.x, Some(gpui::Overflow::Hidden));
                        assert_eq!(overflow.y, Some(gpui::Overflow::Hidden));
                    }
                    assert_eq!(scroll.scroll.offset(), offset);
                }
            }
        }
    }

    #[test]
    fn drag_scroll_speed_is_per_list_and_invalid_values_restore_default() {
        let left = ListBoxScrollHandle::<u32>::default();
        let right = ListBoxScrollHandle::<u32>::default();
        left.set_drag_auto_scroll_speed(720.0);
        assert_eq!(left.drag_scroll.max_speed.get(), 720.0);
        assert_eq!(right.drag_scroll.max_speed.get(), DEFAULT_MAX_SPEED);
        left.set_drag_auto_scroll_speed(0.0);
        assert_eq!(left.drag_scroll.max_speed.get(), 0.0);
        for invalid in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            left.set_drag_auto_scroll_speed(invalid);
            assert_eq!(left.drag_scroll.max_speed.get(), DEFAULT_MAX_SPEED);
        }
    }

    #[test]
    fn pending_reveal_follows_keys_across_reorder_and_ignores_removed_items() {
        let mut state = ListBoxState::try_new([1, 2, 3], |item| *item, SelectionMode::Multiple).unwrap();
        let scroll = ListBoxScrollHandle::default();
        scroll.queue_reveal(&state.apply(ListBoxInput::Navigate(ListBoxNavigation::Last)));
        state.replace_snapshot(ListBoxSnapshot::try_new([3, 1, 2], |item| *item).unwrap());
        assert_eq!(scroll.take_reveal_index(&state), Some(0));
        assert_eq!(scroll.take_reveal_index(&state), None);
        scroll.queue_reveal(&state.apply(ListBoxInput::Navigate(ListBoxNavigation::Last)));
        state.replace_snapshot(ListBoxSnapshot::try_new([3, 1], |item| *item).unwrap());
        assert_eq!(scroll.take_reveal_index(&state), None);
    }

    #[test]
    fn last_navigation_wins_and_selection_only_updates_do_not_cancel_reveal() {
        let mut state = ListBoxState::try_new([1, 2, 3], |item| *item, SelectionMode::Multiple).unwrap();
        let scroll = ListBoxScrollHandle::default();
        scroll.queue_reveal(&state.apply(ListBoxInput::Navigate(ListBoxNavigation::Last)));
        scroll.queue_reveal(&state.apply(ListBoxInput::Navigate(ListBoxNavigation::First)));
        scroll.queue_reveal(&state.apply(ListBoxInput::SelectActive));
        assert_eq!(scroll.take_reveal_index(&state), Some(0));
    }

    #[test]
    fn resize_reveals_on_either_axis_without_resetting_ordinary_scrolling() {
        let initial = size(px(300.0), px(196.0));
        assert_eq!(resize_reveal(initial, size(px(200.0), px(196.0)), Some(8)), Some(8));
        assert_eq!(resize_reveal(initial, size(px(300.0), px(100.0)), Some(8)), Some(8));
        assert_eq!(resize_reveal(initial, initial, Some(8)), None);
        assert_eq!(resize_reveal(initial, size(px(200.0), px(100.0)), None), None);
    }
}
