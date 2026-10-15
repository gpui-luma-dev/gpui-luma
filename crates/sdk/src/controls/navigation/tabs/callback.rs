//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait TabsEventExt: Sized {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::tabs::TabsEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Handle `TabsEvent::Change` through the existing event stream.
    fn on_value_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::Change { tab_id, label } = event {
                callback(owner, tab_id, label, cx);
            }
        })
    }

    /// Handle `TabsEvent::Activate` through the existing event stream.
    fn on_activate<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::Activate { tab_id, label } = event {
                callback(owner, tab_id, label, cx);
            }
        })
    }

    /// Handle `TabsEvent::Reactivate` through the existing event stream.
    fn on_reactivate<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::Reactivate { tab_id, label } = event {
                callback(owner, tab_id, label, cx);
            }
        })
    }

    /// Handle `TabsEvent::DropdownRequested` through the existing event stream.
    fn on_dropdown_requested<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            &gpui::SharedString,
            &gpui::SharedString,
            &Option<gpui::Bounds<gpui::Pixels>>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::DropdownRequested { tab_id, label, bounds } = event {
                callback(owner, tab_id, label, bounds, cx);
            }
        })
    }

    /// Handle `TabsEvent::ItemBoundsChanged` through the existing event stream.
    fn on_item_bounds_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::Bounds<gpui::Pixels>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::ItemBoundsChanged { tab_id, bounds } = event {
                callback(owner, tab_id, bounds, cx);
            }
        })
    }

    /// Handle `TabsEvent::FocusChanged` through the existing event stream.
    fn on_focus_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `TabsEvent::ItemFocused` through the existing event stream.
    fn on_item_focused<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tabs::TabsEvent::ItemFocused { tab_id, label } = event {
                callback(owner, tab_id, label, cx);
            }
        })
    }
}

impl TabsEventExt for gpui::Entity<crate::controls::tabs::Tabs> {
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::tabs::TabsEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
