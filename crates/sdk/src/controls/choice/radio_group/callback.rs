//! Owner-bound callbacks on factory-created radio groups.
/// Semantic callbacks retained until the group is released; owners are weak.
pub trait RadioGroupEventExt<T>: Sized
where
    T: crate::controls::control_group::ControlGroupItemLike + 'static,
{
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::radio_group::RadioGroupEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self;
    /// Handle `RadioGroupEvent::Change` through the existing event stream.
    fn on_value_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::radio_group::RadioGroupEvent::Change { value } = event {
                callback(owner, value, cx);
            }
        })
    }

    /// Handle `RadioGroupEvent::Activate` through the existing event stream.
    fn on_activate<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::radio_group::RadioGroupEvent::Activate { value } = event {
                callback(owner, value, cx);
            }
        })
    }

    /// Handle `RadioGroupEvent::FocusChanged` through the existing event stream.
    fn on_focus_changed<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::radio_group::RadioGroupEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `RadioGroupEvent::ItemFocused` through the existing event stream.
    fn on_item_focused<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::radio_group::RadioGroupEvent::ItemFocused { item_id } = event {
                callback(owner, item_id, cx);
            }
        })
    }
}
impl<T> RadioGroupEventExt<T> for gpui::Entity<crate::controls::radio_group::RadioGroupControl<T>>
where
    T: crate::controls::control_group::ControlGroupItemLike + 'static,
{
    fn on_event<M: 'static>(
        self,
        cx: &mut gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::radio_group::RadioGroupEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        crate::infra::callbacks::EventCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
