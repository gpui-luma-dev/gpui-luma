//! Owner-bound callbacks on controls returned by factories.
/// Semantic event callbacks. Subscriptions live until the control is released;
/// owners are held weakly. Capture entities weakly when they can own this control.
pub trait SelectionPanelEventExt<T>: Sized
where
    T: crate::controls::selection_panel::SelectionPanelItemLike + 'static,
{
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::selection_panel::SelectionPanelEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;

    /// Handle `SelectionPanelEvent::HoverChanged` through the existing event stream.
    fn on_hover_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<usize>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::selection_panel::SelectionPanelEvent::HoverChanged { visible_index } = event {
                callback(owner, visible_index, cx);
            }
        })
    }

    /// Handle `SelectionPanelEvent::ActivateRow` through the existing event stream.
    fn on_activate_row<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, usize, usize, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::selection_panel::SelectionPanelEvent::ActivateRow {
                source_index,
                visible_index,
                item_id,
            } = event
            {
                callback(owner, *source_index, *visible_index, item_id, cx);
            }
        })
    }

    /// Handle `SelectionPanelEvent::ActiveIndexChanged` through the existing event stream.
    fn on_active_index_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<usize>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::selection_panel::SelectionPanelEvent::ActiveIndexChanged { visible_index } = event {
                callback(owner, visible_index, cx);
            }
        })
    }

    /// Handle `SelectionPanelEvent::FocusChanged` through the existing event stream.
    fn on_focus_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::selection_panel::SelectionPanelEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `SelectionPanelEvent::OpenChanged` through the existing event stream.
    fn on_open_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::selection_panel::SelectionPanelEvent::OpenChanged { open } = event {
                callback(owner, *open, cx);
            }
        })
    }
}

impl<T> SelectionPanelEventExt<T> for gpui::Entity<crate::controls::selection_panel::SelectionPanelControl<T>>
where
    T: crate::controls::selection_panel::SelectionPanelItemLike + 'static,
{
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::selection_panel::SelectionPanelEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
