//! Owner-bound Checkbox callbacks dispatched through the ordinary GPUI event stream.
use gpui::{App, Context, Entity, Subscription};
use super::{CheckboxControl, CheckboxEvent};

type Handler = Box<dyn Fn(&CheckboxEvent, &mut App)>;

/// Callback storage shared by the SDK and Look-owned Checkbox builders.
///
/// Handlers run in registration order, including repeated registrations. They hold
/// owners weakly: a released owner is skipped. Captures live until the checkbox (or
/// unspawned builder) is dropped. Separate GPUI subscribers remain supported;
/// their ordering relative to these callbacks follows GPUI dispatch semantics.
#[derive(Default)]
pub struct CheckboxCallbacks {
    handlers: Vec<Handler>,
}

impl CheckboxCallbacks {
    /// Append a callback for every semantic Checkbox event.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &CheckboxEvent, &mut Context<M>) + 'static,
    ) -> Self {
        let owner = cx.weak_entity();
        self.handlers.push(Box::new(move |event, cx| {
            let _ = owner.update(cx, |owner, cx| callback(owner, event, cx));
        }));
        self
    }

    /// Append a user-change callback; `set_data` remains silent.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let CheckboxEvent::Change { checked } = event {
                callback(owner, *checked, cx);
            }
        })
    }

    /// Append a callback for the semantic FocusChanged event.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let CheckboxEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Append a callback for the semantic HoverChanged event.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let CheckboxEvent::HoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Append a callback for the semantic EnabledChanged event.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let CheckboxEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub(super) fn bind(self, entity: &Entity<CheckboxControl>, cx: &mut App) {
        if let Some(subscription) = self.subscribe(entity, cx) {
            entity.update(cx, |checkbox, _| checkbox._subscriptions.push(subscription));
        }
    }

    pub(super) fn subscribe(self, entity: &Entity<CheckboxControl>, cx: &mut App) -> Option<Subscription> {
        if self.handlers.is_empty() {
            return None;
        }
        // App::subscribe does not borrow the emitting Checkbox during dispatch, so
        // an owner callback may update that Checkbox and emit follow-up events.
        Some(cx.subscribe(entity, move |_, event, cx| {
            for handler in &self.handlers {
                handler(event, cx);
            }
        }))
    }
}

/// Owner-bound callbacks for Checkbox controls returned by factories or shared composition.
///
/// Each call retains its subscription on the Checkbox, holds the owner weakly,
/// and returns the same entity. Separate bindings follow GPUI subscriber order.
/// Prefer builder callbacks when constructing the Checkbox directly.
pub trait CheckboxEventExt: Sized {
    /// Attach a callback for every semantic Checkbox event.
    fn on_event<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &CheckboxEvent, &mut Context<M>) + 'static,
    ) -> Self;

    /// Attach a value-change callback to an existing Checkbox.
    fn on_value_changed<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let CheckboxEvent::Change { checked } = event {
                callback(owner, *checked, cx);
            }
        })
    }

    /// Attach a callback for the semantic FocusChanged event.
    fn on_focus_changed<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let CheckboxEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Attach a callback for the semantic HoverChanged event.
    fn on_hover_changed<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let CheckboxEvent::HoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Attach a callback for the semantic EnabledChanged event.
    fn on_enabled_changed<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let CheckboxEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }
}

impl CheckboxEventExt for Entity<CheckboxControl> {
    fn on_event<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &CheckboxEvent, &mut Context<M>) + 'static,
    ) -> Self {
        CheckboxCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
