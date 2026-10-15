//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait ControlGroupEventExt<T>: Sized
where
    T: crate::controls::control_group::ControlGroupItemLike + 'static,
{
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::control_group::ControlGroupEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Handle `ControlGroupEvent::Activate` through the existing event stream.
    fn on_activate<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::control_group::ControlGroupEvent::Activate { activated_id } = event {
                callback(owner, activated_id, cx);
            }
        })
    }

    /// Handle `ControlGroupEvent::Change` through the existing event stream.
    fn on_value_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, bool, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::control_group::ControlGroupEvent::Change { changed_id, selected, selected_ids } =
                event
            {
                callback(owner, changed_id, *selected, selected_ids, cx);
            }
        })
    }

    /// Handle `ControlGroupEvent::ItemBoundsChanged` through the existing event stream.
    fn on_item_bounds_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::Bounds<gpui::Pixels>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::control_group::ControlGroupEvent::ItemBoundsChanged { item_id, bounds } = event {
                callback(owner, item_id, bounds, cx);
            }
        })
    }

    /// Handle `ControlGroupEvent::FocusChanged` through the existing event stream.
    fn on_focus_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::control_group::ControlGroupEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `ControlGroupEvent::ItemFocused` through the existing event stream.
    fn on_item_focused<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::control_group::ControlGroupEvent::ItemFocused { item_id } = event {
                callback(owner, item_id, cx);
            }
        })
    }
}

impl<T> ControlGroupEventExt<T> for gpui::Entity<crate::controls::control_group::ControlGroupControl<T>>
where
    T: crate::controls::control_group::ControlGroupItemLike + 'static,
{
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::control_group::ControlGroupEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
