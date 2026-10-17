//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait TreeViewEventExt<T>: Sized
where
    T: Clone + Send + Sync + 'static,
{
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::tree_view::TreeViewEvent<T>, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Observe the committed `ItemsRemoved` phase before successful completion.
    fn on_items_removed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewDragEvent::ItemsRemoved { source, target, node_id, node_ids } =
                event
            {
                callback(owner, *source, *target, node_id, node_ids, cx);
            }
        })
    }

    /// Observe the committed `ItemsAdded` phase before successful completion.
    fn on_items_added<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &crate::controls::tree_view::TreeDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewDragEvent::ItemsAdded {
                source,
                target,
                node_id,
                node_ids,
                location,
            } = event
            {
                callback(owner, *source, *target, node_id, node_ids, location, cx);
            }
        })
    }

    /// Observe the committed `ItemsReordered` phase before successful completion.
    fn on_items_reordered<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &crate::controls::tree_view::TreeDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewDragEvent::ItemsReordered {
                target,
                node_id,
                node_ids,
                location,
            } = event
            {
                callback(owner, *target, node_id, node_ids, location, cx);
            }
        })
    }

    /// Observe `Started` through the existing drag/drop lifecycle.
    fn on_drag_started<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, &gpui::SharedString, &Vec<gpui::SharedString>, &mut gpui::Context<M>)
        + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewDragEvent::Started { source, node_id, node_ids } = event {
                callback(owner, *source, node_id, node_ids, cx);
            }
        })
    }

    /// Observe `Dropped` through the existing drag/drop lifecycle.
    fn on_dropped<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &crate::controls::tree_view::TreeDropLocation,
            bool,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewDragEvent::Dropped {
                source,
                target,
                node_id,
                node_ids,
                location,
                changed,
            } = event
            {
                callback(owner, *source, *target, node_id, node_ids, location, *changed, cx);
            }
        })
    }

    /// Observe `Rejected` through the existing drag/drop lifecycle.
    fn on_drop_rejected<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &gpui::SharedString,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewDragEvent::Rejected {
                source,
                target,
                node_id,
                node_ids,
                error,
            } = event
            {
                callback(owner, *source, *target, node_id, node_ids, error, cx);
            }
        })
    }

    /// Observe `Ended` through the existing drag/drop lifecycle.
    fn on_drag_ended<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            crate::controls::tree_view::TreeDragEndReason,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewDragEvent::Ended { source, node_id, node_ids, reason } = event {
                callback(owner, *source, node_id, node_ids, *reason, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::DragDrop` through the existing event stream.
    fn on_drag_drop<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::tree_view::TreeViewDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewEvent::DragDrop(event) = event {
                callback(owner, event, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::NodeActivated` through the existing event stream.
    fn on_node_activated<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &T, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewEvent::NodeActivated { node_id, data } = event {
                callback(owner, node_id, data, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::SelectionPolicyChanged` through the existing event stream.
    fn on_selection_policy_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::tree_view::TreeViewSelectionPolicy, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewEvent::SelectionPolicyChanged { policy } = event {
                callback(owner, policy, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::NodeExpanded` through the existing event stream.
    fn on_node_expanded<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &T, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewEvent::NodeExpanded { node_id, data } = event {
                callback(owner, node_id, data, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::NodeCollapsed` through the existing event stream.
    fn on_node_collapsed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &T, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewEvent::NodeCollapsed { node_id, data } = event {
                callback(owner, node_id, data, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::SelectionChanged` through the existing event stream.
    fn on_selection_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &std::collections::HashSet<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewEvent::SelectionChanged { selected_ids } = event {
                callback(owner, selected_ids, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::ActiveNodeChanged` through the existing event stream.
    fn on_active_node_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewEvent::ActiveNodeChanged { node_id } = event {
                callback(owner, node_id, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::ScrollChanged` through the existing event stream.
    fn on_scroll_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewEvent::ScrollChanged { top_index } = event {
                callback(owner, *top_index, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::FocusChanged` through the existing event stream.
    fn on_focus_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::RowHoverChanged` through the existing event stream.
    fn on_row_hover_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, usize, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewEvent::RowHoverChanged { node_id, index, hovered } = event {
                callback(owner, node_id, *index, *hovered, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::EnabledChanged` through the existing event stream.
    fn on_enabled_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }
}

impl<T> TreeViewEventExt<T> for gpui::Entity<crate::controls::tree_view::TreeViewControl<T>>
where
    T: Clone + Send + Sync + 'static,
{
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::tree_view::TreeViewEvent<T>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
