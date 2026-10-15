//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait OverlayWindowEventExt: Sized {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::overlay_window::OverlayWindowEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Handle `OverlayWindowEvent::Opened` through the existing event stream.
    fn on_opened<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::overlay_window::OverlayWindowEvent::Opened = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `OverlayWindowEvent::Dismissed` through the existing event stream.
    fn on_dismissed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::overlay_window::OverlayWindowEvent::Dismissed = event {
                callback(owner, cx);
            }
        })
    }
}

impl OverlayWindowEventExt for gpui::Entity<crate::controls::overlay_window::OverlayWindowControl> {
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::overlay_window::OverlayWindowEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
