//! Persistent state and bindings for look-owned ListBox composition.

use std::hash::Hash;
use gpui::{App, Context, ElementId, SharedString, Window};

use super::{
    ListBoxBinding, ListBoxError, ListBoxInput, ListBoxScrollHandle, ListBoxState, ListBoxUpdate,
    ListBoxVirtualization, SelectionPolicy,
};

/// Host callback for semantic input. Apply it to the control and deliver its events.
pub type ListBoxInputHandler<M, K> = fn(&mut M, ListBoxInput<K>, &mut Window, &mut Context<M>);

/// Persistent SDK objects shared by successive renders. This is host-owned,
/// not a separately spawned entity; the look supplies the visual composition.
pub struct ListBoxControl<M: 'static, T, K> {
    /// Collection and selection state. Deliver updates from direct mutations
    /// through [`Self::handle_update`] before forwarding their events.
    pub state: ListBoxState<T, K>,
    binding: ListBoxBinding,
    scroll: ListBoxScrollHandle<K>,
    on_input: ListBoxInputHandler<M, K>,
    item_id: fn(&K) -> ElementId,
    item_label: fn(&T) -> SharedString,
    virtualization: ListBoxVirtualization,
}

/// Borrowed integration point for look-owned builders. The binding is mutable
/// only to register its persistent focus subscriptions during rendering.
pub struct ListBoxRenderParts<'a, M: 'static, T, K> {
    /// Current collection and selection state.
    pub state: &'a ListBoxState<T, K>,
    /// Stable keyboard and focus binding.
    pub binding: &'a mut ListBoxBinding,
    /// Stable wheel, reveal, and auto-scroll adapter.
    pub scroll: &'a ListBoxScrollHandle<K>,
    /// Host callback used by the input binding.
    pub on_input: ListBoxInputHandler<M, K>,
    /// Stable, unique element ID within this list for each key.
    pub item_id: fn(&K) -> ElementId,
    /// Accessible name of each item, independent of its visual template.
    pub item_label: fn(&T) -> SharedString,
    /// Eager or opt-in virtualized rendering.
    pub virtualization: ListBoxVirtualization,
}

impl<M: 'static, T: 'static, K: Clone + Eq + Hash + 'static> ListBoxControl<M, T, K> {
    /// Create once per list. IDs must remain stable across reorder/replacement;
    /// `on_input` must apply SDK input and forward events to the application.
    pub fn new(
        state: ListBoxState<T, K>,
        on_input: ListBoxInputHandler<M, K>,
        item_id: fn(&K) -> ElementId,
        item_label: fn(&T) -> SharedString,
        cx: &mut App,
    ) -> Self {
        Self {
            state,
            on_input,
            item_id,
            item_label,
            virtualization: ListBoxVirtualization::default(),
            binding: ListBoxBinding::new(cx),
            scroll: ListBoxScrollHandle::default(),
        }
    }

    /// Opt into focus-required wheel routing. Defaults to hover scrolling.
    pub fn require_focus_for_scroll(mut self, required: bool) -> Self {
        self.scroll = self.scroll.require_focus_for_scroll(required);
        self
    }

    /// Opt into uniform or measured virtualization; eager rendering remains the default.
    /// Item templates must keep durable state in their host/entities, since
    /// off-screen element instances are not retained.
    pub fn virtualization(mut self, policy: ListBoxVirtualization) -> Self {
        self.virtualization = policy;
        self
    }

    /// Bring the item with this stable key into view on the next layout.
    /// An already fully visible item does not move. Oversized items show their
    /// leading edge. Does not change selection, active item, or focus; works
    /// without focus. Unknown/removed keys are ignored. The latest request wins.
    pub fn scroll_to(&self, item_key: K, cx: &mut Context<M>) {
        self.scroll.scroll_to(item_key, cx);
    }

    /// Center this item's midpoint in the visible viewport on the next layout.
    /// Uses the list's scrolling axis and clamps at collection boundaries, where
    /// exact centering may be impossible. Does not change selection or focus.
    /// This is a one-time request, not a scrolling or selection policy.
    pub fn scroll_to_center(&self, item_key: K, cx: &mut Context<M>) {
        self.scroll.scroll_to_center(item_key, cx);
    }

    /// Smoothly bring this item into view using shared scroll motion (200ms).
    /// Does not change selection/focus. New input or another request cancels it.
    pub fn scroll_to_smooth(&self, item_key: K, cx: &mut Context<M>) {
        self.scroll.scroll_to_smooth(item_key, cx);
    }

    /// Smoothly center this item in the viewport, clamped at collection boundaries.
    /// Measured/virtualized destinations track the item's actual layout.
    pub fn scroll_to_center_smooth(&self, item_key: K, cx: &mut Context<M>) {
        self.scroll.scroll_to_center_smooth(item_key, cx);
    }

    /// Invalidate one or all content-sized measurements after external content
    /// or theme changes. Snapshot replacement and width changes are automatic.
    pub fn invalidate_measurements(&self, key: Option<&K>, cx: &mut Context<M>) {
        self.scroll.invalidate_measurements(key);
        cx.notify();
    }

    /// Configure the existing basic linear edge-scrolling function, in logical
    /// pixels per second. See [`ListBoxScrollHandle::set_drag_auto_scroll_speed`].
    pub fn set_drag_auto_scroll_speed(&self, pixels_per_second: f32) {
        self.scroll.set_drag_auto_scroll_speed(pixels_per_second);
    }

    /// Apply semantic input and queue rendering/reveal effects. Events remain
    /// host-owned and are returned for delivery after the state is committed.
    pub fn apply(&mut self, input: ListBoxInput<K>, cx: &mut Context<M>) -> ListBoxUpdate<K> {
        let update = self.state.apply(input);
        self.handle_update(&update, cx);
        update
    }

    /// Reconcile a runtime policy change and queue its rendering/reveal effects.
    pub fn set_selection_policy(&mut self, policy: SelectionPolicy, cx: &mut Context<M>) -> ListBoxUpdate<K> {
        let update = self.state.set_selection_policy(policy);
        self.handle_update(&update, cx);
        update
    }

    /// Display an ordered subset of source keys and deliver rendering effects.
    /// Hidden selections are preserved. Return events for host-owned delivery.
    pub fn set_projection(
        &mut self,
        keys: impl IntoIterator<Item = K>,
        cx: &mut Context<M>,
    ) -> Result<ListBoxUpdate<K>, ListBoxError> {
        let update = self.state.set_projection(keys)?;
        self.handle_update(&update, cx);
        Ok(update)
    }

    /// Restore source order and visibility, preserving keyed selection.
    pub fn reset_projection(&mut self, cx: &mut Context<M>) -> ListBoxUpdate<K> {
        let update = self.state.reset_projection();
        self.handle_update(&update, cx);
        update
    }

    /// Deliver effects after direct state mutation, including snapshot changes.
    pub fn handle_update(&self, update: &ListBoxUpdate<K>, cx: &mut Context<M>) {
        self.scroll.handle_update(update, cx);
    }

    /// Borrow existing state/handles for composition without recreating them.
    pub fn render_parts(&mut self) -> ListBoxRenderParts<'_, M, T, K> {
        ListBoxRenderParts {
            state: &self.state,
            binding: &mut self.binding,
            scroll: &self.scroll,
            on_input: self.on_input,
            item_id: self.item_id,
            item_label: self.item_label,
            virtualization: self.virtualization,
        }
    }
}
