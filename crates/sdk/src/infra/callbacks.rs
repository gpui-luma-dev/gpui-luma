//! Owner-bound semantic callbacks shared by control builders and entity adapters.
use gpui::{App, AppContext, Context, Entity, EventEmitter};
use std::rc::Rc;

type Handler<E> = Rc<dyn Fn(&E, &mut App)>;

/// Callback storage used by SDK and Look builders.
///
/// Registrations append in order and hold their owner weakly. Captures live until
/// the builder or emitting control is released. Existing subscribers coexist;
/// ordering between separate bindings follows GPUI event dispatch semantics.
#[doc(hidden)]
#[derive(Clone)]
pub struct EventCallbacks<E> {
    handlers: Vec<Handler<E>>,
}

impl<E> Default for EventCallbacks<E> {
    fn default() -> Self {
        Self { handlers: Vec::new() }
    }
}

impl<E: 'static> EventCallbacks<E> {
    pub fn on_event<M: 'static>(
        mut self,
        cx: &Context<M>,
        callback: impl Fn(&mut M, &E, &mut Context<M>) + 'static,
    ) -> Self {
        let owner = cx.weak_entity();
        self.handlers.push(Rc::new(move |event, cx| {
            let _ = owner.update(cx, |owner, cx| callback(owner, event, cx));
        }));
        self
    }

    /// Deliver a committed host-owned transaction after the current borrow ends.
    pub(crate) fn dispatch_batch(&self, events: &[E], cx: &mut App)
    where
        E: Clone,
    {
        if self.handlers.is_empty() || events.is_empty() {
            return;
        }
        let handlers = self.handlers.clone();
        let events = events.to_vec();
        cx.defer(move |cx| {
            for event in &events {
                for handler in &handlers {
                    handler(event, cx);
                }
            }
        });
    }

    pub fn bind<T: EventEmitter<E> + 'static>(self, entity: &Entity<T>, cx: &mut impl AppContext) {
        if self.handlers.is_empty() {
            return;
        }
        entity.update(cx, |_, cx| {
            // Subscribe through App so callbacks can update the emitting control.
            let app: &mut App = cx;
            let subscription = app.subscribe(entity, move |_, event, cx| {
                for handler in &self.handlers {
                    handler(event, cx);
                }
            });
            // A release listener retains the subscription without retaining the
            // entity itself. Releasing the control drops handlers and captures.
            cx.on_release(move |_, _| drop(subscription)).detach();
        });
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Subscription, TestAppContext};
    use std::cell::Cell;

    struct Signal;
    impl EventEmitter<bool> for Signal {}
    struct Owner {
        signal: Entity<Signal>,
        log: Vec<&'static str>,
        _observer: Option<Subscription>,
    }

    #[test]
    fn callbacks_are_ordered_can_reenter_and_coexist_with_subscribers() {
        let app = TestAppContext::single();
        let owner = app.update(|cx| {
            cx.new(|cx| {
                let signal = cx.new(|_| Signal);
                EventCallbacks::default()
                    .on_event(cx, |owner: &mut Owner, event: &bool, cx| {
                        if *event {
                            owner.log.push("first");
                            owner.signal.update(cx, |_, cx| cx.emit(false));
                        } else {
                            owner.log.push("followup");
                        }
                    })
                    .on_event(cx, |owner, event, _| {
                        if *event {
                            owner.log.push("second");
                        }
                    })
                    .bind(&signal, cx);
                let observer = cx.subscribe(&signal, |owner: &mut Owner, _, event: &bool, _| {
                    if *event {
                        owner.log.push("observer");
                    }
                });
                Owner { signal, log: vec![], _observer: Some(observer) }
            })
        });
        let signal = app.read(|cx| owner.read(cx).signal.clone());
        app.update(|cx| signal.update(cx, |_, cx| cx.emit(true)));
        app.read(|cx| {
            let log = &owner.read(cx).log;
            let inline: Vec<_> = log.iter().copied().filter(|s| *s != "observer").collect();
            assert_eq!(inline, ["first", "second", "followup"]);
            assert_eq!(log.iter().filter(|s| **s == "observer").count(), 1);
        });
    }

    #[test]
    fn release_listener_does_not_retain_owner_or_control_and_releases_captures() {
        let app = TestAppContext::single();
        let calls = Rc::new(Cell::new(0));
        let capture = Rc::new(());
        let owner = app.update(|cx| {
            cx.new(|cx| {
                let signal = cx.new(|_| Signal);
                let calls = calls.clone();
                let capture = capture.clone();
                EventCallbacks::default()
                    .on_event(cx, move |_: &mut Owner, _: &bool, _| {
                        let _keep = &capture;
                        calls.set(calls.get() + 1);
                    })
                    .bind(&signal, cx);
                Owner { signal, log: vec![], _observer: None }
            })
        });
        let signal = app.read(|cx| owner.read(cx).signal.clone());
        let weak_owner = owner.downgrade();
        let weak_signal = signal.downgrade();
        drop(owner);
        app.update(|_| {});
        assert!(weak_owner.upgrade().is_none());
        app.update(|cx| signal.update(cx, |_, cx| cx.emit(true)));
        assert_eq!(calls.get(), 0);
        assert_eq!(Rc::strong_count(&capture), 2);
        drop(signal);
        app.update(|_| {});
        assert!(weak_signal.upgrade().is_none());
        assert_eq!(Rc::strong_count(&capture), 1);
    }

    #[test]
    fn dropping_unspawned_callbacks_releases_captures() {
        let app = TestAppContext::single();
        let capture = Rc::new(());
        app.update(|cx| {
            let owner = cx.new(|_| ());
            owner.update(cx, |_, cx| {
                let retained = capture.clone();
                let callbacks = EventCallbacks::default().on_event(cx, move |_, _: &bool, _| {
                    let _keep = &retained;
                });
                assert_eq!(Rc::strong_count(&capture), 2);
                drop(callbacks);
            });
        });
        assert_eq!(Rc::strong_count(&capture), 1);
    }
}
