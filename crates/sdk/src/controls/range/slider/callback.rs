//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait SliderEventExt: Sized {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::SliderEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Handle `SliderEvent::Change` through the existing event stream.
    fn on_value_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::Change { thumb_id, value } = event {
                callback(owner, thumb_id, *value, cx);
            }
        })
    }

    /// Handle `SliderEvent::Release` through the existing event stream.
    fn on_release<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::Release { thumb_id, value } = event {
                callback(owner, thumb_id, *value, cx);
            }
        })
    }

    /// Handle `SliderEvent::DragStart` through the existing event stream.
    fn on_drag_start<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::DragStart { thumb_id } = event {
                callback(owner, thumb_id, cx);
            }
        })
    }

    /// Handle `SliderEvent::DragEnd` through the existing event stream.
    fn on_drag_end<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::DragEnd { thumb_id, value } = event {
                callback(owner, thumb_id, *value, cx);
            }
        })
    }

    /// Handle `SliderEvent::ThumbAdded` through the existing event stream.
    fn on_thumb_added<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::ThumbAdded { thumb_id, value } = event {
                callback(owner, thumb_id, *value, cx);
            }
        })
    }

    /// Handle `SliderEvent::ThumbRemoved` through the existing event stream.
    fn on_thumb_removed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::ThumbRemoved { thumb_id } = event {
                callback(owner, thumb_id, cx);
            }
        })
    }

    /// Handle `SliderEvent::ThumbSelected` through the existing event stream.
    fn on_thumb_selected<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::ThumbSelected { thumb_id } = event {
                callback(owner, thumb_id, cx);
            }
        })
    }

    /// Handle `SliderEvent::FocusChanged` through the existing event stream.
    fn on_focus_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `SliderEvent::HoverChanged` through the existing event stream.
    fn on_hover_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::HoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Handle `SliderEvent::EnabledChanged` through the existing event stream.
    fn on_enabled_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }
}

impl SliderEventExt for gpui::Entity<crate::controls::slider::SliderControl> {
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::SliderEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
