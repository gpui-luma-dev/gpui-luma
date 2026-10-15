//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait ResizablePanelsEventExt: Sized {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::resizable_panels::ResizablePanelsEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Handle `ResizablePanelsEvent::ResizeStart` through the existing event stream.
    fn on_resize_start<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::resizable_panels::ResizablePanelsEvent::ResizeStart = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `ResizablePanelsEvent::SizesChanged` through the existing event stream.
    fn on_sizes_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &Vec<f32>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::resizable_panels::ResizablePanelsEvent::SizesChanged { sizes_px } = event {
                callback(owner, sizes_px, cx);
            }
        })
    }

    /// Handle `ResizablePanelsEvent::ResizeEnd` through the existing event stream.
    fn on_resize_end<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &Vec<f32>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::resizable_panels::ResizablePanelsEvent::ResizeEnd { sizes_px } = event {
                callback(owner, sizes_px, cx);
            }
        })
    }

    /// Handle `ResizablePanelsEvent::PanelHiddenChanged` through the existing event stream.
    fn on_panel_hidden_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, usize, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::resizable_panels::ResizablePanelsEvent::PanelHiddenChanged { panel_index, hidden } =
                event
            {
                callback(owner, *panel_index, *hidden, cx);
            }
        })
    }

    /// Handle `ResizablePanelsEvent::PanelRegionHiddenChanged` through the existing event stream.
    fn on_panel_region_hidden_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::resizable_panels::PanelId, usize, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::resizable_panels::ResizablePanelsEvent::PanelRegionHiddenChanged {
                region,
                panel_index,
                hidden,
            } = event
            {
                callback(owner, region, *panel_index, *hidden, cx);
            }
        })
    }

    /// Handle `ResizablePanelsEvent::HandleFocusChanged` through the existing event stream.
    fn on_handle_focus_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, usize, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::resizable_panels::ResizablePanelsEvent::HandleFocusChanged {
                handle_index,
                focused,
            } = event
            {
                callback(owner, *handle_index, *focused, cx);
            }
        })
    }

    /// Handle `ResizablePanelsEvent::HandleHoverChanged` through the existing event stream.
    fn on_handle_hover_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, usize, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::resizable_panels::ResizablePanelsEvent::HandleHoverChanged {
                handle_index,
                hovered,
            } = event
            {
                callback(owner, *handle_index, *hovered, cx);
            }
        })
    }

    /// Handle `ResizablePanelsEvent::EnabledChanged` through the existing event stream.
    fn on_enabled_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::resizable_panels::ResizablePanelsEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }
}

impl ResizablePanelsEventExt for gpui::Entity<crate::controls::resizable_panels::ResizablePanels> {
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::resizable_panels::ResizablePanelsEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
