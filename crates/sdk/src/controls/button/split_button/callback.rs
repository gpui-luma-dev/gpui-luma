//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait SplitButtonEventExt: Sized {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::split_button::SplitButtonEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Handle `SplitButtonEvent::ActionClick` through the existing event stream.
    fn on_action_click<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::ActionClick = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::Select` through the existing event stream.
    fn on_select<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::Select { item_id, label } = event {
                callback(owner, item_id, label, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::OpenChanged` through the existing event stream.
    fn on_open_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::Dismiss` through the existing event stream.
    fn on_dismiss<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::Dismiss = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::FocusChanged` through the existing event stream.
    fn on_focus_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::HoverChanged` through the existing event stream.
    fn on_hover_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::HoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Handle `SplitButtonEvent::EnabledChanged` through the existing event stream.
    fn on_enabled_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_button::SplitButtonEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }
}

impl SplitButtonEventExt for gpui::Entity<crate::controls::split_button::SplitButton> {
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::split_button::SplitButtonEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
