//! Owner-bound Switch callbacks dispatched through the ordinary GPUI event stream.
use gpui::{App, Context, Entity, Subscription};
use super::{SwitchControl, SwitchEvent};

type Handler = Box<dyn Fn(&SwitchEvent, &mut App)>;

/// Callback storage shared by the SDK and Look-owned Switch builders.
///
/// Handlers run in registration order, including repeated registrations. They hold
/// owners weakly: a released owner is skipped. Captures live until the switch (or
/// unspawned builder) is dropped. Separate GPUI subscribers remain supported;
/// their ordering relative to these callbacks follows GPUI dispatch semantics.
#[derive(Default)]
pub struct SwitchCallbacks {
    handlers: Vec<Handler>,
}

impl SwitchCallbacks {
    /// Append a callback for every semantic Switch event.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &SwitchEvent, &mut Context<M>) + 'static,
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
            if let SwitchEvent::Change { on } = event {
                callback(owner, *on, cx);
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
            if let SwitchEvent::FocusChanged { focused } = event {
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
            if let SwitchEvent::HoverChanged { hovered } = event {
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
            if let SwitchEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub(super) fn bind(self, entity: &Entity<SwitchControl>, cx: &mut App) {
        if let Some(subscription) = self.subscribe(entity, cx) {
            entity.update(cx, |switch, _| switch._subscriptions.push(subscription));
        }
    }

    pub(super) fn subscribe(self, entity: &Entity<SwitchControl>, cx: &mut App) -> Option<Subscription> {
        if self.handlers.is_empty() {
            return None;
        }
        // App::subscribe does not borrow the emitting Switch during dispatch, so
        // an owner callback may update that Switch and emit follow-up events.
        Some(cx.subscribe(entity, move |_, event, cx| {
            for handler in &self.handlers {
                handler(event, cx);
            }
        }))
    }
}

/// Owner-bound callbacks for Switch controls returned by factories or shared composition.
///
/// Each call retains its subscription on the Switch, holds the owner weakly,
/// and returns the same entity. Separate bindings follow GPUI subscriber order.
/// Prefer builder callbacks when constructing the Switch directly.
pub trait SwitchEventExt: Sized {
    /// Attach a callback for every semantic Switch event.
    fn on_event<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &SwitchEvent, &mut Context<M>) + 'static,
    ) -> Self;

    /// Attach a value-change callback to an existing Switch.
    fn on_value_changed<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, bool, &mut Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let SwitchEvent::Change { on } = event {
                callback(owner, *on, cx);
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
            if let SwitchEvent::FocusChanged { focused } = event {
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
            if let SwitchEvent::HoverChanged { hovered } = event {
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
            if let SwitchEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }
}

impl SwitchEventExt for Entity<SwitchControl> {
    fn on_event<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &SwitchEvent, &mut Context<M>) + 'static,
    ) -> Self {
        SwitchCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
