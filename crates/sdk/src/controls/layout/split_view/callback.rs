//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait SplitViewEventExt: Sized {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::split_view::SplitViewEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Handle `SplitViewEvent::ResizeStart` through the existing event stream.
    fn on_resize_start<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_view::SplitViewEvent::ResizeStart = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `SplitViewEvent::SidebarWidthChanged` through the existing event stream.
    fn on_sidebar_width_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::Pixels, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_view::SplitViewEvent::SidebarWidthChanged { width } = event {
                callback(owner, *width, cx);
            }
        })
    }

    /// Handle `SplitViewEvent::ResizeEnd` through the existing event stream.
    fn on_resize_end<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::Pixels, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_view::SplitViewEvent::ResizeEnd { width } = event {
                callback(owner, *width, cx);
            }
        })
    }

    /// Handle `SplitViewEvent::CollapsedChanged` through the existing event stream.
    fn on_collapsed_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_view::SplitViewEvent::CollapsedChanged { collapsed } = event {
                callback(owner, *collapsed, cx);
            }
        })
    }

    /// Handle `SplitViewEvent::SeparatorHoverChanged` through the existing event stream.
    fn on_separator_hover_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_view::SplitViewEvent::SeparatorHoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Handle `SplitViewEvent::EnabledChanged` through the existing event stream.
    fn on_enabled_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::split_view::SplitViewEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }
}

impl SplitViewEventExt for gpui::Entity<crate::controls::split_view::SplitView> {
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::split_view::SplitViewEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
