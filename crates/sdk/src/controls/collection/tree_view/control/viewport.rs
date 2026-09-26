//! One ListState owns native wheel input, keyboard reveal, and scrollbar geometry.
use gpui::{AnyElement, DispatchPhase, HitboxBehavior, canvas, point};
use crate::controls::scrollbar::ScrollbarEvent;
use crate::controls::ScrollbarVisibility;
use crate::controls::scroll_container::AUTO_HIDE_TIMEOUT;
use super::*;

impl<T: Clone + Send + Sync + 'static> TreeViewControl<T> {
    /// Current scrollbar chrome policy. Scrolling remains enabled when hidden.
    pub fn scrollbar_visibility(&self) -> ScrollbarVisibility {
        self.model.scrollbar_visibility
    }

    /// Change scrollbar chrome without changing the list's scroll position.
    pub fn set_scrollbar_visibility(&mut self, visibility: ScrollbarVisibility, cx: &mut Context<Self>) {
        if self.model.scrollbar_visibility == visibility {
            return;
        }
        self.model.scrollbar_visibility = visibility;
        self.scrollbar_active = false;
        self.scrollbar_hide_task = None;
        self.list_state.scrollbar_drag_ended();
        cx.notify();
    }

    fn wake_scrollbar(&mut self, cx: &mut Context<Self>) {
        if self.scrollbar.is_none() || self.model.scrollbar_visibility != ScrollbarVisibility::AutoHide {
            return;
        }
        if !self.scrollbar_active {
            self.scrollbar_active = true;
            cx.notify();
        }
        self.scrollbar_hide_task = Some(cx.spawn(async |tree, cx| {
            cx.background_executor().timer(AUTO_HIDE_TIMEOUT).await;
            let _ = tree.update(cx, |tree, cx| {
                if !tree.list_state.is_scrollbar_dragging() {
                    tree.scrollbar_active = false;
                    cx.notify();
                }
            });
        }));
    }

    pub(super) fn reveal_item(&self, index: usize) {
        if self.list_state.bounds_for_item(index).is_some() {
            self.list_state.scroll_to_reveal_item(index);
        } else {
            // Native reveal sums measured heights only. Anchor directly when
            // keyboard navigation jumps beyond the measured virtualized region.
            self.list_state.scroll_to(gpui::ListOffset { item_ix: index, offset_in_item: px(0.0) });
        }
    }

    pub(super) fn handle_scrollbar(&mut self, event: &ScrollbarEvent, cx: &mut Context<Self>) {
        match event {
            ScrollbarEvent::Change { value } if self.model.enabled => {
                self.cancel_position();
                self.list_state.set_offset_from_scrollbar(point(px(0.0), px(-*value)));
                cx.notify();
            }
            ScrollbarEvent::DragStart => {
                self.cancel_position();
                self.list_state.scrollbar_drag_started();
                self.wake_scrollbar(cx);
            }
            ScrollbarEvent::DragEnd { .. } => {
                self.list_state.scrollbar_drag_ended();
                self.wake_scrollbar(cx);
                cx.notify();
            }
            _ => {}
        }
    }

    fn sync_viewport(&mut self, cx: &mut Context<Self>) {
        let height = self.list_state.viewport_bounds().size.height.as_f32().max(0.0);
        let maximum = self.list_state.max_offset_for_scrollbar().y.as_f32().max(0.0);
        let offset = (-self.list_state.scroll_px_offset_for_scrollbar().y.as_f32()).clamp(0.0, maximum);
        if let Some(scrollbar) = &self.scrollbar {
            scrollbar.update(cx, |scrollbar, cx| {
                scrollbar.set_length(height.max(1.0), cx);
                scrollbar.set_step(1.0, cx);
                scrollbar.set_page_step((height * 0.85).max(1.0), cx);
                scrollbar.set_viewport(0.0, height + maximum, offset, offset + height, cx);
                scrollbar.set_enabled(self.model.enabled && maximum > 0.5, cx);
            });
        }
        let scrollable = maximum > 0.5;
        if self.scrollbar_scrollable != scrollable {
            self.scrollbar_scrollable = scrollable;
            cx.notify();
        }
        let current = self.list_state.logical_scroll_top();
        if self.viewport_offset.is_some_and(|previous| {
            previous.item_ix != current.item_ix || previous.offset_in_item != current.offset_in_item
        }) {
            self.wake_scrollbar(cx);
        }
        self.viewport_offset = Some(current);
        self.emit_scroll_changed_if_needed(cx);
        self.advance_position(cx);
    }

    pub(super) fn render_viewport(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let state = self.list_state.clone();
        let focus = self.focus_handle.clone();
        let required = self.model.require_focus_for_scroll;
        let enabled = self.model.enabled;
        let owner = cx.entity().downgrade();
        // Register before List::paint: capture snapshots the live position;
        // bubble runs after the native list and before the enclosing page.
        // GPUI 1.21 has no wheel-enable hook on List. Restoring a rejected wheel
        // here retains measured geometry, never paints an intermediate offset,
        // and lets the original event continue to its ancestors. Do not attach
        // a native ListState scroll callback: it would see the transient offset.
        let routing = canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |bounds, hitbox, window, cx| {
                let pointer_owner = owner.clone();
                window.on_mouse_event(move |event: &gpui::MouseDownEvent, phase, _, cx| {
                    if phase == DispatchPhase::Capture && bounds.contains(&event.position) {
                        let _ = pointer_owner.update(cx, |tree, _| tree.cancel_position());
                    }
                });
                let layout_owner = owner.clone();
                cx.defer(move |cx| {
                    let _ = layout_owner.update(cx, |tree, cx| tree.sync_viewport(cx));
                });
                let mut rejected_offset = None;
                window.on_mouse_event(move |_: &ScrollWheelEvent, phase, window, cx| {
                    if !hitbox.should_handle_scroll(window) {
                        return;
                    }
                    let accepts = enabled && (!required || focus.contains_focused(window, cx));
                    if phase == DispatchPhase::Capture {
                        rejected_offset = (!accepts).then(|| state.logical_scroll_top());
                        if accepts {
                            let _ = owner.update(cx, |tree, _| tree.cancel_position());
                        }
                    } else if let Some(offset) = rejected_offset.take() {
                        state.scroll_to(offset);
                    } else if accepts {
                        let _ = owner.update(cx, |tree, cx| tree.wake_scrollbar(cx));
                        // Includes endpoints and horizontal deltas, matching ListBox.
                        cx.stop_propagation();
                    }
                });
            },
        )
        .absolute()
        .size_full();
        let debug_id = format!("{}-viewport", self.model.id);
        let body = div()
            .id(format!("{}-drag-viewport", self.model.id))
            .debug_selector(move || debug_id.clone())
            .relative()
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .overflow_hidden()
            .child(routing)
            .child(list(self.list_state.clone(), cx.processor(Self::render_row)).size_full());
        let body = self.bind_drag_viewport(body, cx);
        let reserve_gutter =
            self.scrollbar_scrollable && self.model.scrollbar_visibility != ScrollbarVisibility::Hidden;
        let show_scrollbar = self.model.scrollbar_visibility == ScrollbarVisibility::AlwaysVisible
            || self.scrollbar_active
            || self.list_state.is_scrollbar_dragging();
        if !(reserve_gutter && show_scrollbar)
            && self.model.enabled
            && self.scrollbar.as_ref().is_some_and(|bar| bar.read(cx).focus_handle(cx).is_focused(window))
        {
            // Removing auto-hidden chrome must not take keyboard/wheel focus
            // away from the tree after a scrollbar interaction.
            self.focus_handle.focus(window, cx);
        }
        let chrome_id = format!("{}-scrollbar-chrome", self.model.id);
        div()
            .size_full()
            .flex()
            .min_h(px(0.0))
            .child(body)
            .when(reserve_gutter, |root| {
                root.when_some(self.scrollbar.clone(), |root, scrollbar| {
                    root.child(div().w(px(12.0)).h_full().flex_shrink_0().when(show_scrollbar, |gutter| {
                        gutter.child(div().debug_selector(move || chrome_id.clone()).size_full().child(scrollbar))
                    }))
                })
            })
            .into_any_element()
    }
}
