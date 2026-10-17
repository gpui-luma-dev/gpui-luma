use super::*;
use super::super::{ListBoxInput, ListBoxState, ListBoxUpdate, SelectionMode};
use gpui::{AppContext, TestAppContext};
use std::{cell::Cell, rc::Rc};

struct Owner {
    list: ListBoxControl<Self, String, String>,
    log: Vec<ListBoxEvent<String>>,
    batches: Vec<ListBoxEvent<String>>,
}

fn list(cx: &mut Context<Owner>) -> ListBoxControl<Owner, String, String> {
    let state = ListBoxState::try_new(["a".into(), "b".into()], |s: &String| s.clone(), SelectionMode::SingleAllowNone)
        .unwrap();
    ListBoxControl::new(
        state,
        |owner, input, _, cx| {
            owner.list.apply(input, cx);
        },
        |key| key.clone().into(),
        |row| row.clone().into(),
        cx,
    )
}

#[test]
fn named_handlers_filter_payloads_and_preserve_batch_order() {
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let list = list(cx)
                .on_selection_policy_changed(cx, |owner, policy, _| {
                    owner.log.push(ListBoxEvent::SelectionPolicyChanged { policy })
                })
                .on_selection_changed(cx, |owner, selected, active, _| {
                    owner
                        .log
                        .push(ListBoxEvent::SelectionChanged { selected: selected.to_vec(), active: active.cloned() })
                })
                .on_active_item_changed(cx, |owner, active, _| {
                    owner.log.push(ListBoxEvent::ActiveItemChanged { active: active.cloned() })
                })
                .on_item_activated(cx, |owner, key, _| owner.log.push(ListBoxEvent::ItemActivated { key: key.clone() }))
                .on_projection_changed(cx, |owner, visible_count, _| {
                    owner.log.push(ListBoxEvent::ProjectionChanged { visible_count })
                })
                .on_focus_changed(cx, |owner, is_focused, _| owner.log.push(ListBoxEvent::FocusChanged { is_focused }))
                .on_event(cx, |owner, event, _| owner.batches.push(event.clone()));
            Owner { list, log: vec![], batches: vec![] }
        })
    });
    let events = vec![
        ListBoxEvent::SelectionPolicyChanged { policy: SelectionPolicy::new(SelectionMode::Extended) },
        ListBoxEvent::SelectionChanged { selected: vec!["a".into(), "b".into()], active: Some("b".into()) },
        ListBoxEvent::ActiveItemChanged { active: None },
        ListBoxEvent::ItemActivated { key: "a".into() },
        ListBoxEvent::ProjectionChanged { visible_count: 1 },
        ListBoxEvent::FocusChanged { is_focused: true },
    ];
    app.update(|cx| {
        owner.update(cx, |owner, cx| {
            owner.list.handle_update(&ListBoxUpdate { changed: true, events: events.clone(), reveal: None }, cx);
            assert!(owner.log.is_empty(), "delivery waits for owner borrow to end");
        })
    });
    app.read(|cx| {
        assert_eq!(owner.read(cx).log, events);
        assert_eq!(owner.read(cx).batches, events);
    });
}

#[test]
fn callbacks_observe_committed_state_append_and_can_update_list() {
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let list = list(cx)
                .on_event(cx, |owner, event, _| owner.batches.push(event.clone()))
                .on_item_activated(cx, |owner, key, cx| {
                    assert_eq!(owner.list.state.active_key(), Some(key));
                    owner.log.push(ListBoxEvent::ItemActivated { key: key.clone() });
                    owner.list.apply(ListBoxInput::Focus(true), cx);
                })
                .on_item_activated(cx, |owner, key, _| {
                    owner.log.push(ListBoxEvent::ItemActivated { key: key.clone() })
                });
            Owner { list, log: vec![], batches: vec![] }
        })
    });
    let returned = app.update(|cx| {
        owner.update(cx, |owner, cx| {
            owner.list.apply(ListBoxInput::Select("b".into()), cx);
            owner.list.apply(ListBoxInput::ConfirmActive, cx)
        })
    });
    app.read(|cx| {
        let owner = owner.read(cx);
        assert_eq!(owner.log, vec![ListBoxEvent::ItemActivated { key: "b".into() }; 2]);
        assert_eq!(returned.events, vec![ListBoxEvent::ItemActivated { key: "b".into() }]);
        assert_eq!(owner.batches.last(), Some(&ListBoxEvent::FocusChanged { is_focused: true }));
    });
}

#[test]
fn callbacks_do_not_retain_owner_and_release_captures() {
    let app = TestAppContext::single();
    let capture = Rc::new(());
    let calls = Rc::new(Cell::new(0));
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let capture = capture.clone();
            let calls = calls.clone();
            let list = list(cx).on_event(cx, move |_, _, _| {
                let _keep = &capture;
                calls.set(calls.get() + 1);
            });
            Owner { list, log: vec![], batches: vec![] }
        })
    });
    let weak = owner.downgrade();
    app.update(|cx| {
        owner.update(cx, |owner, cx| {
            owner.list.apply(ListBoxInput::Focus(true), cx);
        });
        drop(owner);
    });
    assert!(weak.upgrade().is_none());
    assert_eq!(calls.get(), 0);
    assert_eq!(Rc::strong_count(&capture), 1);
}

#[test]
fn programmatic_updates_and_direct_state_mutations_deliver_once() {
    let app = TestAppContext::single();
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let list = list(cx).on_event(cx, |owner, event, _| owner.batches.push(event.clone()));
            Owner { list, log: vec![], batches: vec![] }
        })
    });
    let expected = app.update(|cx| {
        owner.update(cx, |owner, cx| {
            let mut expected =
                owner.list.set_selection_policy(SelectionPolicy::new(SelectionMode::Extended), cx).events;
            expected.extend(owner.list.set_projection(["a".into()], cx).unwrap().events);
            expected.extend(owner.list.reset_projection(cx).events);
            let update = owner.list.state.apply(ListBoxInput::Select("b".into()));
            owner.list.handle_update(&update, cx);
            expected.extend(update.events);
            // An idempotent input must not introduce another callback.
            assert!(owner.list.apply(ListBoxInput::Select("b".into()), cx).events.is_empty());
            expected
        })
    });
    assert!(!expected.is_empty());
    app.read(|cx| assert_eq!(owner.read(cx).batches, expected));
}
