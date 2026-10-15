//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait PopoverButtonEventExt<D: 'static>: Sized {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::popover_button::PopoverButtonEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Handle `PopoverButtonEvent::OpenChanged` through the existing event stream.
    fn on_open_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            let crate::controls::popover_button::PopoverButtonEvent::OpenChanged { open } = event;
            callback(owner, *open, cx);
        })
    }
}

impl<D: 'static> PopoverButtonEventExt<D> for gpui::Entity<crate::controls::popover_button::PopoverButton<D>> {
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::popover_button::PopoverButtonEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
