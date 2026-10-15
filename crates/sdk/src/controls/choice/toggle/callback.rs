//! Owner-bound Toggle callbacks dispatched through the ordinary GPUI event stream.
use gpui::{App, Context, Entity, Subscription};
use super::{ToggleControl, ToggleEvent};

type Handler = Box<dyn Fn(&ToggleEvent, &mut App)>;

/// Callback storage shared by the SDK and Look-owned Toggle builders.
///
/// Handlers run in registration order, including repeated registrations. They hold
/// owners weakly: a released owner is skipped. Captures live until the toggle (or
/// unspawned builder) is dropped. Separate GPUI subscribers remain supported;
/// their ordering relative to these callbacks follows GPUI dispatch semantics.
#[derive(Default)]
pub struct ToggleCallbacks {
    handlers: Vec<Handler>,
}

impl ToggleCallbacks {
    /// Append a callback for every semantic Toggle event.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &ToggleEvent, &mut Context<M>) + 'static,
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
            if let ToggleEvent::Change { selected } = event {
                callback(owner, *selected, cx);
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
            if let ToggleEvent::FocusChanged { focused } = event {
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
            if let ToggleEvent::HoverChanged { hovered } = event {
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
            if let ToggleEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub(super) fn bind(self, entity: &Entity<ToggleControl>, cx: &mut App) {
        if let Some(subscription) = self.subscribe(entity, cx) {
            entity.update(cx, |toggle, _| toggle._subscriptions.push(subscription));
        }
    }

    pub(super) fn subscribe(self, entity: &Entity<ToggleControl>, cx: &mut App) -> Option<Subscription> {
        if self.handlers.is_empty() {
            return None;
        }
        // App::subscribe does not borrow the emitting Toggle during dispatch, so
        // an owner callback may update that Toggle and emit follow-up events.
        Some(cx.subscribe(entity, move |_, event, cx| {
            for handler in &self.handlers {
                handler(event, cx);
            }
        }))
    }
}

/// Owner-bound callbacks for Toggles returned by factories or shared composition.
///
/// Each call retains its subscription on the Toggle, holds the owner weakly,
/// and returns the same entity. Separate bindings follow GPUI subscriber order.
/// Prefer builder callbacks when constructing the Toggle directly.
pub trait ToggleEventExt: Sized {
    /// Attach a callback for every semantic Toggle event.
    fn on_event<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &ToggleEvent, &mut Context<M>) + 'static,
    ) -> Self;

    /// Attach a value-change callback to an existing Toggle.
    fn on_value_changed<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ToggleEvent::Change { selected } = event {
                callback(owner, *selected, cx);
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
            if let ToggleEvent::FocusChanged { focused } = event {
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
            if let ToggleEvent::HoverChanged { hovered } = event {
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
            if let ToggleEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }
}

impl ToggleEventExt for Entity<ToggleControl> {
    fn on_event<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &ToggleEvent, &mut Context<M>) + 'static,
    ) -> Self {
        ToggleCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
