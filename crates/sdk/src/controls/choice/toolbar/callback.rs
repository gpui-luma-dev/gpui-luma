//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait ToolbarEventExt: Sized {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::toolbar::ToolbarEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Handle `ToolbarEvent::Click` through the existing event stream.
    fn on_click<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::toolbar::ToolbarEvent::Click { id } = event {
                callback(owner, id, cx);
            }
        })
    }

    /// Handle `ToolbarEvent::Change` through the existing event stream.
    fn on_value_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &crate::controls::toolbar::ToolbarValue, &mut gpui::Context<M>)
        + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::toolbar::ToolbarEvent::Change { id, value } = event {
                callback(owner, id, value, cx);
            }
        })
    }
}

impl ToolbarEventExt for gpui::Entity<crate::controls::toolbar::ToolbarControl> {
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::toolbar::ToolbarEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
