use std::{cell::Cell, rc::Rc};

use gpui::{AppContext, Entity, TestAppContext};
use super::*;

struct Owner {
    button: Entity<Button>,
    log: Vec<String>,
    _observer: Option<Subscription>,
}

fn activate(button: &Entity<Button>, cx: &mut App) {
    button.update(cx, |button, cx| {
        button.command.activate(button.model.enabled, cx);
    });
}

#[test]
fn callbacks_append_in_order_and_coexist_with_subscribers() {
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let button = Button::new("save")
                .on_event(cx, |owner: &mut Owner, event, _| {
                    if event.is_click() {
                        owner.log.push("all".into());
                    }
                })
                .on_click(cx, |owner, _| owner.log.push("first".into()))
                .on_click(cx, |owner, _| owner.log.push("second".into()))
                .spawn(cx);
            let observer = cx.subscribe(&button, |owner: &mut Owner, _, event: &ButtonEvent, _| {
                if event.is_click() {
                    owner.log.push("observer".into());
                }
            });
            Owner { button, log: Vec::new(), _observer: Some(observer) }
        })
    });
    let button = app.read(|cx| owner.read(cx).button.clone());
    app.update(|cx| activate(&button, cx));
    app.read(|cx| {
        let log = &owner.read(cx).log;
        let inline: Vec<_> = log.iter().filter(|entry| entry.as_str() != "observer").map(String::as_str).collect();
        assert_eq!(inline, ["all", "first", "second"]);
        assert_eq!(log.iter().filter(|entry| entry.as_str() == "observer").count(), 1);
    });
}

#[test]
fn callbacks_receive_secondary_events_and_idempotent_setters_do_not_repeat() {
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let button = Button::new("states")
                .on_focus_changed(cx, |owner: &mut Owner, value, _| owner.log.push(format!("focus:{value}")))
                .on_hover_changed(cx, |owner, value, _| owner.log.push(format!("hover:{value}")))
                .on_enabled_changed(cx, |owner, value, _| owner.log.push(format!("enabled:{value}")))
                .spawn(cx);
            Owner { button, log: Vec::new(), _observer: None }
        })
    });
    let button = app.read(|cx| owner.read(cx).button.clone());
    app.update(|cx| {
        button.update(cx, |button, cx| {
            button.command.handle_focus_changed(true, true, cx);
            button.command.handle_hover(true, true, cx);
        })
    });
    app.update(|cx| button.update(cx, |button, cx| button.set_enabled(false, cx)));
    app.update(|cx| button.update(cx, |button, cx| button.set_enabled(false, cx)));
    app.read(|cx| {
        assert_eq!(owner.read(cx).log, ["focus:true", "hover:true", "enabled:false", "hover:false", "focus:false"])
    });
}

#[test]
fn callbacks_can_update_the_emitter_and_queue_followup_events() {
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let button = Button::new("save")
                .on_click(cx, |owner: &mut Owner, cx| {
                    owner.log.push("click".into());
                    owner.button.update(cx, |button, cx| button.set_enabled(false, cx));
                    cx.notify();
                })
                .on_enabled_changed(cx, |owner, enabled, _| owner.log.push(format!("enabled:{enabled}")))
                .spawn(cx);
            Owner { button, log: Vec::new(), _observer: None }
        })
    });
    let button = app.read(|cx| owner.read(cx).button.clone());
    app.update(|cx| activate(&button, cx));
    app.update(|cx| activate(&button, cx));
    app.read(|cx| assert_eq!(owner.read(cx).log, ["click", "enabled:false"]));
}

#[test]
fn owner_is_weak_and_button_drop_releases_callback_captures() {
    let app = TestAppContext::single();
    let calls = Rc::new(Cell::new(0));
    let captured = Rc::new(());
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let calls = calls.clone();
            let captured = captured.clone();
            let button = Button::new("lifetime")
                .on_click(cx, move |_: &mut Owner, _| {
                    let _ = &captured;
                    calls.set(calls.get() + 1);
                })
                .spawn(cx);
            Owner { button, log: Vec::new(), _observer: None }
        })
    });
    let button = app.read(|cx| owner.read(cx).button.clone());
    let weak_owner = owner.downgrade();
    app.update(|_| drop(owner));
    assert!(weak_owner.upgrade().is_none(), "callback must not retain owner");
    app.update(|cx| activate(&button, cx));
    assert_eq!(calls.get(), 0);
    app.update(|_| drop(button));
    assert_eq!(Rc::strong_count(&captured), 1, "released control frees captures");
}

#[test]
fn typed_conversion_preserves_callbacks_and_unspawned_builders_release_them() {
    let app = TestAppContext::single();
    let captured = Rc::new(());
    app.update(|cx| {
        let owner = cx.new(|cx| {
            let captured = captured.clone();
            let builder = Button::new("draft").on_click(cx, move |_: &mut (), _| {
                let _ = &captured;
            });
            drop(builder.typed(42_u32));
        });
        drop(owner);
    });
    assert_eq!(Rc::strong_count(&captured), 1);
    struct TypedOwner {
        button: Entity<Button<u32>>,
        clicks: usize,
    }
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let button = Button::new("typed")
                .on_click(cx, |owner: &mut TypedOwner, _| owner.clicks += 1)
                .typed(42_u32)
                .spawn(cx);
            TypedOwner { button, clicks: 0 }
        })
    });
    app.update(|cx| {
        let button = owner.read(cx).button.clone();
        button.update(cx, |button, cx| {
            button.command.activate(button.model.enabled, cx);
        });
    });
    app.read(|cx| assert_eq!(owner.read(cx).clicks, 1));
}

#[test]
fn pointer_and_keyboard_activation_dispatch_once_each_and_respect_disabled() {
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let button =
                Button::new("input").on_click(cx, |owner: &mut Owner, _| owner.log.push("click".into())).spawn(cx);
            Owner { button, log: Vec::new(), _observer: None }
        })
    });
    let button = app.read(|cx| owner.read(cx).button.clone());
    app.update(|cx| {
        button.update(cx, |button, cx| {
            button.command.handle_click(true, &gpui::ClickEvent::Mouse(Default::default()), cx);
            button.command.handle_activate_control(true, &crate::key_handling::ActivateControl, cx);
            // GPUI's synthetic keyboard click must not activate a second time.
            button.command.handle_click(true, &gpui::ClickEvent::Keyboard(Default::default()), cx);
        })
    });
    app.update(|cx| {
        button.update(cx, |button, cx| {
            button.set_enabled(false, cx);
            button.command.handle_click(false, &gpui::ClickEvent::Mouse(Default::default()), cx);
            button.command.handle_activate_control(false, &crate::key_handling::ActivateControl, cx);
        })
    });
    app.read(|cx| assert_eq!(owner.read(cx).log, ["click", "click"]));
}

#[test]
fn factory_buttons_retain_multiple_entity_callbacks_without_replacing_builder_callbacks() {
    use crate::controls::button::ButtonEventExt;
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let button = Button::new("factory")
                .on_click(cx, |owner: &mut Owner, _| owner.log.push("builder".into()))
                .spawn(cx)
                .on_click(cx, |owner, _| owner.log.push("entity".into()))
                .on_focus_changed(cx, |owner, value, _| owner.log.push(format!("focus:{value}")))
                .on_hover_changed(cx, |owner, value, _| owner.log.push(format!("hover:{value}")))
                .on_enabled_changed(cx, |owner, value, _| owner.log.push(format!("enabled:{value}")));
            Owner { button, log: Vec::new(), _observer: None }
        })
    });
    let button = app.read(|cx| owner.read(cx).button.clone());
    app.update(|cx| activate(&button, cx));
    app.update(|cx| {
        button.update(cx, |button, cx| {
            button.command.handle_focus_changed(true, true, cx);
            button.command.handle_hover(true, true, cx);
            button.set_enabled(false, cx);
        })
    });
    app.read(|cx| {
        let log = &owner.read(cx).log;
        assert_eq!(log.iter().filter(|entry| entry.as_str() == "builder").count(), 1);
        assert_eq!(log.iter().filter(|entry| entry.as_str() == "entity").count(), 1);
        for event in ["focus:true", "hover:true", "enabled:false", "focus:false", "hover:false"] {
            assert!(log.iter().any(|entry| entry == event), "missing {event}");
        }
    });
}

#[test]
fn callback_can_bind_another_handler_for_subsequent_events() {
    use crate::controls::button::ButtonEventExt;
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let button = Button::new("register")
                .on_click(cx, |owner: &mut Owner, cx| {
                    if owner.log.is_empty() {
                        owner.button.clone().on_click(cx, |owner, _| owner.log.push("late".into()));
                    }
                    owner.log.push("first".into());
                })
                .spawn(cx);
            Owner { button, log: Vec::new(), _observer: None }
        })
    });
    let button = app.read(|cx| owner.read(cx).button.clone());
    app.update(|cx| activate(&button, cx));
    app.read(|cx| assert_eq!(owner.read(cx).log, ["first"]));
    app.update(|cx| activate(&button, cx));
    app.read(|cx| assert_eq!(owner.read(cx).log, ["first", "first", "late"]));
}
