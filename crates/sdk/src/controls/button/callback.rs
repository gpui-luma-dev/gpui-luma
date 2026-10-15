//! Owner-bound Button callbacks dispatched through the ordinary GPUI event stream.
use gpui::{App, Context, Entity, Subscription};
use super::{Button, ButtonEvent};

type Handler = Box<dyn Fn(&ButtonEvent, &mut App)>;

/// Callback storage shared by the SDK and Look-owned Button builders.
///
/// Handlers run in registration order, including repeated registrations. They hold
/// owners weakly: a released owner is skipped. Captures live until the button (or
/// unspawned builder) is dropped. Separate GPUI subscribers remain supported;
/// their ordering relative to these callbacks follows GPUI dispatch semantics.
#[derive(Default)]
pub struct ButtonCallbacks {
    handlers: Vec<Handler>,
}

impl ButtonCallbacks {
    /// Append a callback for every semantic Button event.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &ButtonEvent, &mut Context<M>) + 'static,
    ) -> Self {
        let owner = cx.weak_entity();
        self.handlers.push(Box::new(move |event, cx| {
            let _ = owner.update(cx, |owner, cx| callback(owner, event, cx));
        }));
        self
    }

    /// Append an activation callback (pointer and keyboard share Click).
    pub fn on_click<M: 'static>(self, cx: &Context<M>, callback: impl Fn(&mut M, &mut Context<M>) + 'static) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ButtonEvent::Click = event {
                callback(owner, cx);
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
            if let ButtonEvent::FocusChanged { focused } = event {
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
            if let ButtonEvent::HoverChanged { hovered } = event {
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
            if let ButtonEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub(super) fn bind<D: Clone + 'static>(self, entity: &Entity<Button<D>>, cx: &mut App) {
        if let Some(subscription) = self.subscribe(entity, cx) {
            entity.update(cx, |button, _| button.callback_subscriptions.push(subscription));
        }
    }

    pub(super) fn subscribe<D: Clone + 'static>(
        self,
        entity: &Entity<Button<D>>,
        cx: &mut App,
    ) -> Option<Subscription> {
        if self.handlers.is_empty() {
            return None;
        }
        // App::subscribe does not borrow the emitting Button during dispatch, so
        // an owner callback may update that Button and emit follow-up events.
        Some(cx.subscribe(entity, move |_, event, cx| {
            for handler in &self.handlers {
                handler(event, cx);
            }
        }))
    }
}

/// Owner-bound callbacks for Buttons returned by factories or shared composition.
///
/// Each call retains its subscription on the Button, holds the owner weakly,
/// and returns the same entity. Separate bindings follow GPUI subscriber order.
/// Prefer builder callbacks when constructing the Button directly.
pub trait ButtonEventExt: Sized {
    /// Attach a callback for every semantic Button event.
    fn on_event<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &ButtonEvent, &mut Context<M>) + 'static,
    ) -> Self;

    /// Attach an activation callback to an existing Button.
    fn on_click<M: 'static>(self, cx: &mut Context<M>, callback: impl Fn(&mut M, &mut Context<M>) + 'static) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let ButtonEvent::Click = event {
                callback(owner, cx);
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
            if let ButtonEvent::FocusChanged { focused } = event {
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
            if let ButtonEvent::HoverChanged { hovered } = event {
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
            if let ButtonEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }
}

impl<D: Clone + 'static> ButtonEventExt for Entity<Button<D>> {
    fn on_event<M: 'static>(
        self,
        cx: &mut Context<M>,
        callback: impl Fn(&mut M, &ButtonEvent, &mut Context<M>) + 'static,
    ) -> Self {
        ButtonCallbacks::default().on_event(cx, callback).bind(&self, cx);
        self
    }
}
