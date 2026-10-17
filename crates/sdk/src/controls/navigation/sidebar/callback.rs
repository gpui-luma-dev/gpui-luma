//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait SidebarEventExt: Sized {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::sidebar::SidebarEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Observe the complete drag lifecycle through the semantic event stream.
    fn on_drag_drop<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::sidebar::SidebarDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::DragDrop(event) = event {
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
            if let crate::controls::sidebar::SidebarDragEvent::DragStarted { source, keys } = event {
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
            if let crate::controls::sidebar::SidebarDragEvent::ItemsRemoved { source, target, keys } = event {
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
            &crate::controls::sidebar::SidebarDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarDragEvent::ItemsAdded { source, target, keys, location } = event {
                callback(owner, *source, *target, keys, location, cx);
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
            &crate::controls::sidebar::SidebarDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarDragEvent::ItemsReordered { target, keys, location } = event {
                callback(owner, *target, keys, location, cx);
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
            &crate::controls::sidebar::SidebarDropLocation,
            bool,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarDragEvent::Dropped { source, target, keys, location, changed } =
                event
            {
                callback(owner, *source, *target, keys, location, *changed, cx);
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
            &crate::controls::sidebar::SidebarDropLocation,
            &gpui::SharedString,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarDragEvent::DropRejected { source, target, keys, location, error } =
                event
            {
                callback(owner, *source, *target, keys, location, error, cx);
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
            if let crate::controls::sidebar::SidebarDragEvent::DragEnded { source, keys, reason } = event {
                callback(owner, *source, keys, *reason, cx);
            }
        })
    }

    /// Handle `SidebarEvent::PresentationChanged` through the existing event stream.
    fn on_presentation_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::sidebar::SidebarPresentation, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::PresentationChanged { presentation } = event {
                callback(owner, presentation, cx);
            }
        })
    }

    /// Handle `SidebarEvent::Select` through the existing event stream.
    fn on_select<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::Select { id } = event {
                callback(owner, id, cx);
            }
        })
    }

    /// Handle `SidebarEvent::ItemFocused` through the existing event stream.
    fn on_item_focused<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::ItemFocused { id } = event {
                callback(owner, id, cx);
            }
        })
    }

    /// Handle `SidebarEvent::HoverChanged` through the existing event stream.
    fn on_hover_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::HoverChanged { id } = event {
                callback(owner, id, cx);
            }
        })
    }

    /// Handle `SidebarEvent::SubMenuToggle` through the existing event stream.
    fn on_sub_menu_toggle<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::SubMenuToggle { id, open } = event {
                callback(owner, id, *open, cx);
            }
        })
    }

    /// Handle `SidebarEvent::EnabledChanged` through the existing event stream.
    fn on_enabled_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::sidebar::SidebarEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }
}

impl SidebarEventExt for gpui::Entity<crate::controls::sidebar::SidebarControl> {
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::sidebar::SidebarEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
