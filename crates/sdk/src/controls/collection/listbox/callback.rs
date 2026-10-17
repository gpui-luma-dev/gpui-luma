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
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &ListBoxEvent<K>, &mut Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default()
            .on_event(cx, callback)
            .bind(&self.event_source, cx);
        self
    }

    /// Observe the shared keyed lifecycle through `ListBoxEvent::DragDrop`.
    pub fn on_drag_drop(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &crate::infra::drag_drop::DragDropEvent<gpui::EntityId, K>, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ListBoxEvent::DragDrop(event) = event {
                callback(owner, event, cx);
            }
        })
    }

    /// Observe `DragStarted` through the existing drag/drop lifecycle.
    pub fn on_drag_started(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, &Vec<K>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::DragStarted { source, keys } = event {
                callback(owner, *source, keys, cx);
            }
        })
    }

    /// Observe `ItemsRemoved` through the existing drag/drop lifecycle.
    pub fn on_items_removed(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, gpui::EntityId, &Vec<K>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::ItemsRemoved { source, target, keys } = event {
                callback(owner, *source, *target, keys, cx);
            }
        })
    }

    /// Observe `ItemsAdded` through the existing drag/drop lifecycle.
    pub fn on_items_added(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, gpui::EntityId, &Vec<K>, &Option<K>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::ItemsAdded { source, target, keys, before } = event {
                callback(owner, *source, *target, keys, before, cx);
            }
        })
    }

    /// Observe `ItemsReordered` through the existing drag/drop lifecycle.
    pub fn on_items_reordered(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, &Vec<K>, &Option<K>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::ItemsReordered { target, keys, before } = event {
                callback(owner, *target, keys, before, cx);
            }
        })
    }

    /// Observe `Dropped` through the existing drag/drop lifecycle.
    pub fn on_dropped(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, gpui::EntityId, &Vec<K>, &Option<K>, bool, &mut gpui::Context<M>)
        + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::Dropped { source, target, keys, before, changed } = event {
                callback(owner, *source, *target, keys, before, *changed, cx);
            }
        })
    }

    /// Observe `DropRejected` through the existing drag/drop lifecycle.
    pub fn on_drop_rejected(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<K>,
            &Option<K>,
            &gpui::SharedString,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::DropRejected { source, target, keys, before, error } = event
            {
                callback(owner, *source, *target, keys, before, error, cx);
            }
        })
    }

    /// Observe `DragEnded` through the existing drag/drop lifecycle.
    pub fn on_drag_ended(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, &Vec<K>, crate::infra::drag_drop::DragEndReason, &mut gpui::Context<M>)
        + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::DragEnded { source, keys, reason } = event {
                callback(owner, *source, keys, *reason, cx);
            }
        })
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
