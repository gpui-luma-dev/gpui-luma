//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait TableEventExt<T: 'static>: Sized {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::table::TableEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Observe every drag/drop lifecycle phase. Alias of `on_row_drag`.
    fn on_drag_drop<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::table::TableRowDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_row_drag(cx, callback)
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

    /// Handle `TableEvent::RowDrag` through the existing event stream.
    fn on_row_drag<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::table::TableRowDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::RowDrag(event) = event {
                callback(owner, event, cx);
            }
        })
    }

    /// Handle `TableEvent::SelectionChanged` through the existing event stream.
    fn on_selection_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &Vec<usize>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::SelectionChanged { selected_indices } = event {
                callback(owner, selected_indices, cx);
            }
        })
    }

    /// Handle `TableEvent::SelectedKeysChanged` through the existing event stream.
    fn on_selected_keys_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::SelectedKeysChanged { selected_keys } = event {
                callback(owner, selected_keys, cx);
            }
        })
    }

    /// Handle `TableEvent::ActiveIndexChanged` through the existing event stream.
    fn on_active_index_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<usize>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::ActiveIndexChanged { active_index } = event {
                callback(owner, active_index, cx);
            }
        })
    }

    /// Handle `TableEvent::PageChanged` through the existing event stream.
    fn on_page_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::PageChanged { page } = event {
                callback(owner, *page, cx);
            }
        })
    }

    /// Handle `TableEvent::PageSizeChanged` through the existing event stream.
    fn on_page_size_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::PageSizeChanged { page_size } = event {
                callback(owner, *page_size, cx);
            }
        })
    }

    /// Handle `TableEvent::ScrollChanged` through the existing event stream.
    fn on_scroll_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::ScrollChanged { top_index } = event {
                callback(owner, *top_index, cx);
            }
        })
    }

    /// Handle `TableEvent::FocusChanged` through the existing event stream.
    fn on_focus_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `TableEvent::RowHoverChanged` through the existing event stream.
    fn on_row_hover_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, usize, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::RowHoverChanged { index, hovered } = event {
                callback(owner, *index, *hovered, cx);
            }
        })
    }

    /// Handle `TableEvent::EnabledChanged` through the existing event stream.
    fn on_enabled_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::table::TableEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }
}

impl<T: 'static> TableEventExt<T> for gpui::Entity<crate::controls::table::PagingTableControl<T>> {
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::table::TableEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}

impl<T: 'static> TableEventExt<T> for gpui::Entity<crate::controls::table::TableControl<T>> {
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::table::TableEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
