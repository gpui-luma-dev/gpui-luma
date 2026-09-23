//! Headless integration coverage for windowed rendering on both axes.
use std::{cell::RefCell, rc::Rc};
use gpui::{Bounds, Pixels, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase, VisualTestContext, point};
use luma::controls::listbox::{ListBoxSnapshot, SelectionMode};
use super::*;

struct Harness {
    list: ListBoxControl<Self, u32, u32>,
    look: ShadcnLook,
    horizontal: bool,
    width: f32,
    visible: usize,
    rendered: Rc<RefCell<Vec<u32>>>,
    parent_wheels: usize,
}

impl Harness {
    fn new(horizontal: bool, virtualized: bool, cx: &mut Context<Self>) -> Self {
        let snapshot = ListBoxSnapshot::try_with_enabled(1..=10_000, |n| *n, |n| *n != 7).unwrap();
        Self {
            list: ListBoxControl::new(
                ListBoxState::from_snapshot(snapshot, SelectionMode::Extended),
                Self::input,
                |key| ("item", *key).into(),
                |n| format!("Item {n}").into(),
                cx,
            )
            .require_focus_for_scroll(true)
            .virtualization(if virtualized {
                ListBoxVirtualization::Uniform { overscan: 2 }
            } else {
                ListBoxVirtualization::Eager
            }),
            look: ShadcnLook::built_in(),
            horizontal,
            width: 250.0,
            visible: 5,
            rendered: Rc::default(),
            parent_wheels: 0,
        }
    }

    fn input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.list.apply(input, cx);
    }
}

impl Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.rendered.borrow_mut().clear();
        // Deliberately borrow a local non-Send value: virtualization must not
        // impose a retained/'static template callback.
        let rendered = self.rendered.clone();
        let flow = if self.horizontal {
            ListBoxFlow::Horizontal { item_width: 136.0, item_height: 36.0, gap: 8.0 }
        } else {
            ListBoxFlow::Vertical { visible_items: self.visible, item_height: 36.0, gap: 4.0 }
        };
        let surface = self
            .look
            .render_listbox(
                &mut self.list,
                "virtual-list",
                flow,
                (8.0, 8.0),
                |model, _| {
                    rendered.borrow_mut().push(*model.item);
                    div().size_full().child(model.item.to_string())
                },
                window,
                cx,
            )
            .w(px(self.width))
            .flex_shrink_0();
        div()
            .id("parent")
            .child(surface)
            .on_scroll_wheel(cx.listener(|this, _, _, _| this.parent_wheels += 1))
    }
}

fn settle(cx: &mut VisualTestContext) {
    // TestWindow has no platform frame loop; deliver requested layout frames.
    for _ in 0..4 {
        cx.run_until_parked();
        if cx.update(|window, app| window.simulate_next_frame(app)) == 0 {
            break;
        }
    }
    cx.run_until_parked();
}

fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector).unwrap_or_else(|| panic!("missing {selector}"))
}

fn assert_revealed(cx: &mut VisualTestContext, selector: &'static str, horizontal: bool) {
    settle(cx);
    let viewport = bounds(cx, "virtual-list-viewport");
    let item = bounds(cx, selector);
    if horizontal {
        assert!(
            item.left() >= viewport.left() && item.right() <= viewport.right(),
            "{selector}: {item:?} vs {viewport:?}"
        );
    } else {
        assert!(
            item.top() >= viewport.top() && item.bottom() <= viewport.bottom(),
            "{selector}: {item:?} vs {viewport:?}"
        );
    }
}

fn wheel(cx: &mut VisualTestContext, delta: f32) {
    cx.simulate_event(ScrollWheelEvent {
        position: point(px(30.0), px(25.0)),
        delta: ScrollDelta::Pixels(point(px(0.0), px(delta))),
        modifiers: Default::default(),
        touch_phase: TouchPhase::Moved,
    });
    settle(cx);
}

#[test]
fn listbox_virtualization_bounds_templates_and_reveals_offscreen_selection_on_both_axes() {
    for horizontal in [false, true] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| Harness::new(horizontal, true, cx));
        cx.update(|window, _| window.activate_window());
        settle(cx);
        cx.update(|_, app| assert_eq!(view.read(app).rendered.borrow().len(), if horizontal { 4 } else { 7 }));
        cx.simulate_click(point(px(30.0), px(25.0)), Default::default());
        cx.simulate_keystrokes("shift-end");
        assert_revealed(cx, "virtual-list-item-9999", horizontal);
        cx.update(|_, app| {
            let host = view.read(app);
            assert_eq!(host.list.state.active_key(), Some(&10_000));
            assert_eq!(host.list.state.selected_keys().count(), 9_999); // disabled seventh item
            let rendered = host.rendered.borrow();
            assert!(rendered.len() <= 9 && rendered.contains(&10_000) && !rendered.contains(&1));
        });
        // Resize with the active item at the far end; preserve its reveal.
        cx.update(|_, app| {
            view.update(app, |host, cx| {
                host.width = 200.0;
                host.visible = 3;
                cx.notify();
            })
        });
        assert_revealed(cx, "virtual-list-item-9999", horizontal);
        cx.simulate_keystrokes("home");
        assert_revealed(cx, "virtual-list-item-0", horizontal);
        cx.simulate_keystrokes(if horizontal {
            "right right right right right right"
        } else {
            "down down down down down down"
        });
        assert_revealed(cx, "virtual-list-item-7", horizontal);
        cx.update(|_, app| assert_eq!(view.read(app).list.state.active_key(), Some(&8)));
    }
}

#[test]
fn listbox_virtualization_wheel_focus_endpoints_and_snapshot_shrink() {
    for horizontal in [false, true] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| Harness::new(horizontal, true, cx));
        cx.update(|window, _| window.activate_window());
        settle(cx);
        let first = bounds(cx, "virtual-list-item-0");
        wheel(cx, -10_000.0);
        assert_eq!(bounds(cx, "virtual-list-item-0"), first);
        cx.simulate_click(point(px(30.0), px(25.0)), Default::default());
        wheel(cx, -10_000.0);
        cx.update(|_, app| {
            let host = view.read(app);
            assert_eq!(host.parent_wheels, 1);
            assert!(!host.rendered.borrow().contains(&1));
            assert!(host.rendered.borrow().len() <= 10);
        });
        wheel(cx, -10_000_000.0);
        assert_revealed(cx, "virtual-list-item-9999", horizontal);
        wheel(cx, -100.0);
        assert_revealed(cx, "virtual-list-item-9999", horizontal);
        cx.update(|_, app| assert_eq!(view.read(app).parent_wheels, 1));
        cx.update(|window, app| {
            let elsewhere = app.focus_handle();
            elsewhere.focus(window, app);
        });
        settle(cx);
        wheel(cx, 500.0);
        assert_revealed(cx, "virtual-list-item-9999", horizontal);
        cx.update(|_, app| assert_eq!(view.read(app).parent_wheels, 2));
        for count in [3, 0, 20] {
            cx.update(|_, app| {
                view.update(app, |host, cx| {
                    let update = host.list.state.replace_snapshot(ListBoxSnapshot::try_new(1..=count, |n| *n).unwrap());
                    host.list.handle_update(&update, cx);
                })
            });
            settle(cx);
            cx.update(|_, app| {
                let rendered = view.read(app).rendered.borrow();
                assert!(rendered.len() <= 9);
                if count == 0 {
                    assert!(rendered.is_empty());
                } else {
                    assert!(rendered.contains(&1));
                }
            });
        }
    }
}

#[test]
fn listbox_eager_rendering_remains_the_default() {
    assert_eq!(ListBoxVirtualization::default(), ListBoxVirtualization::Eager);
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, cx| {
        let mut host = Harness::new(false, false, cx);
        host.list.state.replace_snapshot(ListBoxSnapshot::try_new(1..=20, |n| *n).unwrap());
        host
    });
    cx.update(|_, app| assert_eq!(view.read(app).rendered.borrow().len(), 20));
}
