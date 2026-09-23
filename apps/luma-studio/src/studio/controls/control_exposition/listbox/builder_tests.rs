//! Exercise the local builder through a headless GPUI TestWindow, never Studio.
use std::{cell::Cell, rc::Rc};

use gpui::{MouseButton, TestAppContext, VisualTestContext, point};
use luma::controls::listbox::{ListBoxSnapshot, SelectionMode};
use luma::infra::drag_drop::DragEndReason;

use super::*;

struct Preview;
impl Render for Preview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size(px(20.0))
    }
}

struct Harness {
    lists: [ListBoxState<u32, u32>; 2],
    bindings: [ListBoxBinding; 2],
    scrolls: [ListBoxScrollHandle<u32>; 2],
    look: Arc<ShadcnLook>,
    dnd: bool,
    previews: Rc<Cell<usize>>,
    events: Vec<DragDropEvent<usize, u32>>,
    drops: Vec<(usize, Option<u32>)>,
}

impl Harness {
    fn new(dnd: bool, cx: &mut Context<Self>) -> Self {
        luma::focus::bind_default_focus_keys(cx);
        Self {
            lists: [0, 10].map(|offset| {
                let snapshot =
                    ListBoxSnapshot::try_with_enabled((1..=4).map(|n| n + offset), |n| *n, |n| *n != 3).unwrap();
                ListBoxState::from_snapshot(snapshot, SelectionMode::Multiple)
            }),
            bindings: std::array::from_fn(|_| ListBoxBinding::new(cx)),
            scrolls: std::array::from_fn(|_| ListBoxScrollHandle::default().require_focus_for_scroll(true)),
            look: Arc::new(ShadcnLook::built_in()),
            dnd,
            previews: Rc::default(),
            events: Vec::new(),
            drops: Vec::new(),
        }
    }

    fn input(&mut self, side: usize, input: ListBoxInput<u32>, cx: &mut Context<Self>) {
        let update = self.lists[side].apply(input);
        self.scrolls[side].handle_update(&update, cx);
    }
    fn left_input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.input(0, input, cx);
    }
    fn right_input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.input(1, input, cx);
    }
    fn event(&mut self, event: DragDropEvent<usize, u32>, cx: &mut Context<Self>) {
        self.events.push(event);
        cx.notify();
    }
    fn drop(&mut self, proposal: DropProposal<usize, u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.drops.push((*proposal.target(), proposal.before().copied()));
        for event in proposal.committed(true, proposal.keys().to_vec()) {
            self.event(event, cx);
        }
    }
}

impl Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lists = [0, 1].map(|side| {
            let previews = self.previews.clone();
            let mut builder = ListBoxBuilder::new(
                if side == 0 { "left" } else { "right" },
                &self.lists[side],
                &mut self.bindings[side],
                &self.scrolls[side],
                if side == 0 { Self::left_input } else { Self::right_input },
                |key| ("item", *key).into(),
                |surface, _| surface.size_full(),
            )
            .empty(div().size(px(20.0)));
            if self.dnd {
                builder = builder.drag_and_drop(side, Self::drop, Self::event, move |_, _, _, cx| {
                    previews.set(previews.get() + 1);
                    cx.new(|_| Preview)
                });
            }
            // Theme may be supplied after DnD without freezing the old look.
            builder.look(self.look.clone()).build(window, cx).w(px(250.0)).flex_shrink_0()
        });
        div().flex().gap(px(20.0)).children(lists)
    }
}

fn drag(cx: &mut VisualTestContext, destination: (f32, f32)) {
    cx.simulate_mouse_down(point(px(30.0), px(25.0)), MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(60.0), px(25.0)), MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(destination.0), px(destination.1)), MouseButton::Left, Default::default());
}

#[test]
fn selected_group_reaches_keyed_gaps_and_same_list_targets() {
    for (destination, expected) in [
        ((300.0, 20.0), (1, Some(11))),
        ((300.0, 59.0), (1, Some(12))),
        ((300.0, 80.0), (1, Some(13))),
        ((300.0, 160.0), (1, None)),
        ((30.0, 100.0), (0, Some(3))),
    ] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let mut harness = Harness::new(true, cx);
            harness.lists[0].set_selected_keys([2, 1]).unwrap();
            harness
        });
        cx.update(|_, app| assert_eq!(view.read(app).previews.get(), 0));
        drag(cx, destination);
        cx.simulate_mouse_up(point(px(destination.0), px(destination.1)), MouseButton::Left, Default::default());
        cx.update(|_, app| {
            let state = view.read(app);
            assert_eq!(state.drops, vec![expected]);
            assert_eq!(state.previews.get(), 1);
            assert_eq!(state.events[0], DragDropEvent::DragStarted { source: 0, keys: vec![1, 2] });
            assert_eq!(state.events.iter().filter(|event| matches!(event, DragDropEvent::DragEnded { .. })).count(), 1);
            assert!(!app.has_active_drag());
        });
    }
}

#[test]
fn optional_dnd_preserves_selection_keyboard_focus_and_disabled_rows() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, cx| Harness::new(false, cx));
    // TestWindow starts inactive; focus observers require an active test window.
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    cx.simulate_click(point(px(30.0), px(25.0)), Default::default());
    cx.simulate_keystrokes("down down space");
    cx.update(|window, app| {
        let state = view.read(app);
        assert!(state.bindings[0].focus_handle().is_focused(window));
        assert!(state.lists[0].is_focused());
        assert_eq!(state.lists[0].active_key(), Some(&4));
        assert_eq!(state.lists[0].selected_keys().copied().collect::<Vec<_>>(), vec![1, 4]);
    });
    drag(cx, (300.0, 25.0));
    cx.update(|_, app| {
        let state = view.read(app);
        assert!(!app.has_active_drag());
        assert_eq!(state.previews.get(), 0);
        assert!(state.events.is_empty());
    });
    cx.simulate_mouse_up(point(px(300.0), px(25.0)), MouseButton::Left, Default::default());
    cx.simulate_click(point(px(30.0), px(100.0)), Default::default());
    cx.update(|_, app| assert_eq!(view.read(app).lists[0].selected_keys().copied().collect::<Vec<_>>(), vec![1, 4]));
}

#[test]
fn empty_destination_accepts_append_and_escape_cancels_once() {
    for cancel in [false, true] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let mut harness = Harness::new(true, cx);
            harness.lists[1].replace_snapshot(ListBoxSnapshot::try_new([], |n: &u32| *n).unwrap());
            harness
        });
        drag(cx, (300.0, 90.0));
        if cancel {
            cx.simulate_keystrokes("escape");
        }
        cx.simulate_mouse_up(point(px(300.0), px(90.0)), MouseButton::Left, Default::default());
        cx.update(|_, app| {
            let state = view.read(app);
            assert_eq!(state.drops, if cancel { vec![] } else { vec![(1, None)] });
            let reason = if cancel {
                DragEndReason::Cancelled
            } else {
                DragEndReason::Transferred
            };
            assert_eq!(state.events.last(), Some(&DragDropEvent::DragEnded { source: 0, keys: vec![1], reason }));
            assert_eq!(state.events.iter().filter(|event| matches!(event, DragDropEvent::DragEnded { .. })).count(), 1);
        });
    }
}
