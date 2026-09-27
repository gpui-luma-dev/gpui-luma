//! Headless native event dispatch: assert both viewport offsets, not just handlers.
use gpui::{
    App, Context, Entity, FocusHandle, Focusable, Render, ScrollDelta, ScrollHandle, ScrollWheelEvent, TestAppContext,
    VisualTestContext, Window, div, point, prelude::*, px,
};
use crate::interaction::{ScrollBoundaryPolicy, ScrollInteraction, WheelScrollPolicy};

pub(crate) struct Page<V: Render + 'static> {
    pub(crate) child: Entity<V>,
    pub(crate) scroll: ScrollHandle,
    pub(crate) focus: FocusHandle,
}
impl<V: Render + 'static> Render for Page<V> {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("policy-page")
            .w(px(300.0))
            .h(px(240.0))
            .track_focus(&self.focus)
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(div().w_full().h(px(150.0)).child(self.child.clone()))
            .child(div().w_full().h(px(1500.0)))
    }
}
pub(crate) fn wheel(cx: &mut VisualTestContext, x: f32, y: f32) {
    cx.simulate_event(ScrollWheelEvent {
        position: point(px(20.0), px(30.0)),
        delta: ScrollDelta::Pixels(point(px(x), px(y))),
        ..Default::default()
    });
    cx.run_until_parked();
}
pub(crate) fn matrix<V: Render + Focusable + 'static>(
    create: impl Fn(ScrollInteraction, &mut Context<Page<V>>) -> Entity<V>,
    offset: impl Fn(&V, &App) -> f32,
    end: impl Fn(&mut V, &mut Context<V>),
) {
    for wheel_policy in [WheelScrollPolicy::Pointer, WheelScrollPolicy::RequireFocus, WheelScrollPolicy::PassThrough] {
        for boundary in [ScrollBoundaryPolicy::Contain, ScrollBoundaryPolicy::Chain] {
            for focused in [false, true] {
                let policy = ScrollInteraction { wheel: wheel_policy, boundary, ..ScrollInteraction::VIEWPORT };
                let mut app = TestAppContext::single();
                let (page, cx) = app.add_window_view(|window, cx| {
                    window.activate_window();
                    Page { child: create(policy, cx), scroll: ScrollHandle::new(), focus: cx.focus_handle() }
                });
                let child = cx.update(|window, app| {
                    let p = page.read(app);
                    let child = p.child.clone();
                    let focus = if focused {
                        child.read(app).focus_handle(app)
                    } else {
                        p.focus.clone()
                    };
                    focus.focus(window, app);
                    child
                });
                cx.run_until_parked();
                let accepts = wheel_policy.accepts(focused);
                wheel(cx, 0.0, -60.0);
                cx.update(|window, app| {
                    assert!(
                        (offset(child.read(app), app) - if accepts { 60.0 } else { 0.0 }).abs() < 0.1,
                        "child: {policy:?}, focused={focused}"
                    );
                    assert_eq!(
                        page.read(app).scroll.offset().y,
                        px(if accepts { 0.0 } else { -60.0 }),
                        "parent: {policy:?}"
                    );
                    assert_eq!(child.read(app).focus_handle(app).is_focused(window), focused, "wheel stole focus");
                });
                // Reset only the parent so the pointer remains over the child.
                page.update(cx, |p, cx| {
                    p.scroll.set_offset(point(px(0.0), px(0.0)));
                    cx.notify();
                });
                cx.run_until_parked();
                if accepts {
                    wheel(cx, 0.0, 100_000.0); // partial movement: no residual forwarded
                    cx.update(|_, app| {
                        assert_eq!(offset(child.read(app), app), 0.0);
                        assert_eq!(page.read(app).scroll.offset().y, px(0.0));
                    });
                }
                // Programmatic positioning works even for ineligible viewports.
                child.update(cx, |v, cx| end(v, cx));
                cx.run_until_parked();
                let maximum = cx.update(|_, app| offset(child.read(app), app));
                assert!(maximum > 60.0);
                wheel(cx, 0.0, -60.0);
                cx.update(|_, app| {
                    assert!((offset(child.read(app), app) - maximum).abs() < 0.1);
                    let contains = accepts && boundary == ScrollBoundaryPolicy::Contain;
                    assert_eq!(page.read(app).scroll.offset().y, px(if contains { 0.0 } else { -60.0 }));
                });
                page.update(cx, |p, cx| {
                    p.scroll.set_offset(point(px(0.0), px(0.0)));
                    cx.notify();
                });
                cx.run_until_parked();
                wheel(cx, -30.0, 0.0); // unsupported axis never moves the vertical child
                wheel(cx, 0.0, 0.0);
                cx.update(|_, app| {
                    assert!((offset(child.read(app), app) - maximum).abs() < 0.1);
                    // The native document surface remaps X-only input to Y.
                    // Passing this event outward must preserve that host behavior.
                    assert_eq!(page.read(app).scroll.offset().y, px(-30.0));
                });
                page.update(cx, |p, cx| {
                    p.scroll.set_offset(point(px(0.0), px(0.0)));
                    cx.notify();
                });
                cx.run_until_parked();
                wheel(cx, 10.0, 0.25); // diagonal + fractional input uses only Y
                cx.update(|_, app| {
                    let expected = maximum - if accepts { 0.25 } else { 0.0 };
                    assert!((offset(child.read(app), app) - expected).abs() < 0.1);
                });
            }
        }
    }
}
