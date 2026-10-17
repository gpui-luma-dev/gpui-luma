//! Named callbacks for persistent host-owned ListBox controls.
use std::hash::Hash;
use gpui::Context;
use super::{ListBoxControl, ListBoxEvent, SelectionPolicy};

impl<M: 'static, T: 'static, K: Clone + Eq + Hash + 'static> ListBoxControl<M, T, K> {
    /// Register an owner-bound callback for every event in a committed update.
    /// Handlers append in registration order and hold the owner weakly. They run
    /// after the current borrow ends and may update the list. Returned event
    /// batches remain available. Capture owning entities weakly.
    pub fn on_event(
        mut self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &ListBoxEvent<K>, &mut Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `ListBoxEvent::SelectionPolicyChanged` after the transaction commits.
    pub fn on_selection_policy_changed(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, SelectionPolicy, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ListBoxEvent::SelectionPolicyChanged { policy } = event {
                callback(owner, *policy, cx);
            }
        })
    }

    /// Handle `ListBoxEvent::SelectionChanged` after the transaction commits.
    pub fn on_selection_changed(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &[K], Option<&K>, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ListBoxEvent::SelectionChanged { selected, active } = event {
                callback(owner, selected, active.as_ref(), cx);
            }
        })
    }

    /// Handle `ListBoxEvent::ActiveItemChanged` after the transaction commits.
    pub fn on_active_item_changed(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, Option<&K>, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ListBoxEvent::ActiveItemChanged { active } = event {
                callback(owner, active.as_ref(), cx);
            }
        })
    }

    /// Handle `ListBoxEvent::ItemActivated` after the transaction commits.
    pub fn on_item_activated(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &K, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ListBoxEvent::ItemActivated { key } = event {
                callback(owner, key, cx);
            }
        })
    }

    /// Handle `ListBoxEvent::ProjectionChanged` after the transaction commits.
    pub fn on_projection_changed(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, usize, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ListBoxEvent::ProjectionChanged { visible_count } = event {
                callback(owner, *visible_count, cx);
            }
        })
    }

    /// Handle `ListBoxEvent::FocusChanged` after the transaction commits.
    pub fn on_focus_changed(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ListBoxEvent::FocusChanged { is_focused } = event {
                callback(owner, *is_focused, cx);
            }
        })
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests;
