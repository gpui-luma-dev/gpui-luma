//! Native event dispatch tests, using GPUI's TestWindow rather than a platform app.
use super::*;
use gpui::{App, Axis, Context, MouseButton, Render, TestAppContext, Window, div, point, prelude::*, px};

struct Preview;
impl Render for Preview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size(px(20.0))
    }
}

#[derive(Default)]
struct Harness {
    focus: Option<gpui::FocusHandle>,
    list_focus: Option<gpui::FocusHandle>,
    events: Vec<DragDropEvent<u8, u32>>,
    row_clicks: usize,
    child_clicks: usize,
    child_drags: usize,
    child_draggable: bool,
    proposals: Vec<Option<u32>>,
}

impl Harness {
    fn event(&mut self, event: DragDropEvent<u8, u32>, cx: &mut Context<Self>) {
        self.events.push(event);
        cx.notify();
    }
    fn drop(&mut self, proposal: DropProposal<u8, u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.proposals.push(proposal.before().copied());
        for event in proposal.committed(true, proposal.keys().to_vec()) {
            self.event(event, cx);
        }
    }
}

impl Render for Harness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        let child = div()
            .id("child")
            .w(px(80.0))
            .h(px(40.0))
            .on_click(cx.listener(|this, _, _, cx| {
                this.child_clicks += 1;
                cx.notify();
            }))
            .when(self.child_draggable, |child| {
                child.on_drag(42_u32, move |_, _, _, cx| {
                    let _ = owner.update(cx, |this, cx| {
                        this.child_drags += 1;
                        cx.notify();
                    });
                    cx.new(|_| Preview)
                })
            });
        let row = div()
            .id("row")
            .w(px(200.0))
            .h(px(40.0))
            .flex_shrink_0()
            .on_click(cx.listener(|this, _, _, cx| {
                this.row_clicks += 1;
                cx.notify();
            }))
            .child(div().id("child-boundary").w(px(80.0)).h(px(40.0)).drag_boundary().child(child));
        let row = bind_drag_source(
            row,
            KeyedDrag::new(cx.entity_id(), 0, vec![1]).unwrap(),
            |_, _, _, cx| cx.new(|_| Preview),
            cx,
            Self::event,
        );
        let mut target = div().id("target").relative().w(px(200.0)).h(px(40.0)).flex_shrink_0();
        for (edge, before) in [(DropEdge::Before, Some(9)), (DropEdge::After, None)] {
            let zone = DropZone {
                axis: Axis::Vertical,
                edge,
                item_extent: px(40.0),
                following_gap: px(0.0),
                marker_width: px(2.0),
            };
            let zone = zone.apply(div().id(if before.is_some() { "before" } else { "after" }));
            target = target.child(KeyedDropTarget::new(cx.entity_id(), 1, before).bind(zone, cx, Self::drop));
        }
        use crate::focus::LumaFocusScopeExt;
        div().when_some(self.focus.as_ref(), |root, focus| root.luma_focus_scope(focus)).child(
            div()
                .id("root")
                .when_some(self.list_focus.as_ref(), |root, focus| root.track_focus(focus))
                .flex()
                .flex_col()
                .size_full()
                .child(row)
                .child(target)
                .cancel_drag_on_escape(),
        )
    }
}

#[test]
fn nested_child_click_and_pointer_drag_do_not_arm_parent_row_drag() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, _| Harness::default());
    cx.simulate_click(point(px(20.0), px(20.0)), Default::default());
    cx.simulate_mouse_down(point(px(20.0), px(20.0)), MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(120.0), px(20.0)), MouseButton::Left, Default::default());
    cx.update(|_, app: &mut App| {
        assert!(!app.has_active_drag());
        let state = view.read(app);
        assert_eq!(state.child_clicks, 1);
        assert_eq!(state.row_clicks, 0);
        assert!(state.events.is_empty());
    });
    cx.simulate_mouse_up(point(px(120.0), px(20.0)), MouseButton::Left, Default::default());
}

#[test]
fn nested_child_retains_its_own_native_drag() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, _| Harness { child_draggable: true, ..Default::default() });
    cx.simulate_mouse_down(point(px(20.0), px(20.0)), MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(120.0), px(20.0)), MouseButton::Left, Default::default());
    cx.update(|window, app| {
        assert!(app.has_active_drag());
        assert_eq!(view.read(app).child_drags, 1);
        assert!(view.read(app).events.is_empty());
        app.stop_active_drag(window);
    });
}

#[test]
fn native_row_drag_resolves_both_gap_targets_and_emits_one_end() {
    for (y, before) in [(50.0, Some(9)), (70.0, None)] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, _| Harness::default());
        cx.simulate_mouse_down(point(px(120.0), px(20.0)), MouseButton::Left, Default::default());
        cx.simulate_mouse_move(point(px(140.0), px(20.0)), MouseButton::Left, Default::default());
        cx.simulate_mouse_move(point(px(140.0), px(y)), MouseButton::Left, Default::default());
        cx.simulate_mouse_up(point(px(140.0), px(y)), MouseButton::Left, Default::default());
        cx.update(|_, app| {
            let state = view.read(app);
            assert_eq!(state.proposals, vec![before]);
            assert!(matches!(state.events.first(), Some(DragDropEvent::DragStarted { .. })));
            assert_eq!(state.events.len(), 5);
            assert!(matches!(
                state.events.last(),
                Some(DragDropEvent::DragEnded { reason: DragEndReason::Transferred, .. })
            ));
        });
    }
}

#[test]
fn native_outside_release_cancels_without_drop() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, _| Harness::default());
    cx.simulate_mouse_down(point(px(120.0), px(20.0)), MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(140.0), px(20.0)), MouseButton::Left, Default::default());
    cx.simulate_mouse_up(point(px(250.0), px(150.0)), MouseButton::Left, Default::default());
    cx.update(|_, app| {
        let state = view.read(app);
        assert!(state.proposals.is_empty());
        assert_eq!(state.events.len(), 2);
        assert!(matches!(
            state.events.last(),
            Some(DragDropEvent::DragEnded { reason: DragEndReason::Cancelled, .. })
        ));
    });
}

#[test]
fn escape_cancels_drag_and_then_returns_to_normal_focus_scope_handling() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|window, cx| {
        crate::focus::bind_default_focus_keys(cx);
        let focus = cx.focus_handle();
        let list_focus = cx.focus_handle();
        list_focus.focus(window, cx);
        Harness { focus: Some(focus), list_focus: Some(list_focus), ..Default::default() }
    });
    cx.simulate_mouse_down(point(px(120.0), px(20.0)), MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(140.0), px(20.0)), MouseButton::Left, Default::default());
    cx.simulate_keystrokes("escape");
    cx.simulate_keystrokes("escape");
    cx.update(|window, app| {
        let state = view.read(app);
        assert!(!app.has_active_drag());
        assert_eq!(state.events.len(), 2);
        assert!(matches!(
            state.events.last(),
            Some(DragDropEvent::DragEnded { reason: DragEndReason::Cancelled, .. })
        ));
        assert!(state.focus.as_ref().unwrap().is_focused(window));
    });
}
