//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait AccordionEventExt: Sized {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::accordion::AccordionEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Observe the complete drag lifecycle through the semantic event stream.
    fn on_drag_drop<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::accordion::AccordionDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::accordion::AccordionEvent::DragDrop(event) = event {
                callback(owner, event, cx);
            }
        })
    }
    /// Observe `DragStarted` through the existing drag/drop lifecycle.
    fn on_drag_started<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::DragStarted { source, keys } = event {
                callback(owner, *source, keys, cx);
            }
        })
    }

    /// Observe `ItemsRemoved` through the existing drag/drop lifecycle.
    fn on_items_removed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, gpui::EntityId, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::ItemsRemoved { source, target, keys } = event {
                callback(owner, *source, *target, keys, cx);
            }
        })
    }

    /// Observe `ItemsAdded` through the existing drag/drop lifecycle.
    fn on_items_added<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::ItemsAdded { source, target, keys, before } = event {
                callback(owner, *source, *target, keys, before, cx);
            }
        })
    }

    /// Observe `ItemsReordered` through the existing drag/drop lifecycle.
    fn on_items_reordered<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::ItemsReordered { target, keys, before } = event {
                callback(owner, *target, keys, before, cx);
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
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            bool,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::Dropped { source, target, keys, before, changed } = event {
                callback(owner, *source, *target, keys, before, *changed, cx);
            }
        })
    }

    /// Observe `DropRejected` through the existing drag/drop lifecycle.
    fn on_drop_rejected<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
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
    fn on_drag_ended<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            crate::infra::drag_drop::DragEndReason,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::infra::drag_drop::DragDropEvent::DragEnded { source, keys, reason } = event {
                callback(owner, *source, keys, *reason, cx);
            }
        })
    }

    /// Handle `AccordionEvent::ExpandedChanged` through the existing event stream.
    fn on_expanded_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::accordion::AccordionEvent::ExpandedChanged { item_id, expanded } = event {
                callback(owner, item_id, *expanded, cx);
            }
        })
    }

    /// Handle `AccordionEvent::FocusChanged` through the existing event stream.
    fn on_focus_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::accordion::AccordionEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `AccordionEvent::ItemFocused` through the existing event stream.
    fn on_item_focused<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::accordion::AccordionEvent::ItemFocused { item_id } = event {
                callback(owner, item_id, cx);
            }
        })
    }

    /// Handle `AccordionEvent::ItemHoverChanged` through the existing event stream.
    fn on_item_hover_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::accordion::AccordionEvent::ItemHoverChanged { item_id, hovered } = event {
                callback(owner, item_id, *hovered, cx);
            }
        })
    }

    /// Handle `AccordionEvent::EnabledChanged` through the existing event stream.
    fn on_enabled_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::accordion::AccordionEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }
}

impl AccordionEventExt for gpui::Entity<crate::controls::accordion::AccordionControl> {
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::accordion::AccordionEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
