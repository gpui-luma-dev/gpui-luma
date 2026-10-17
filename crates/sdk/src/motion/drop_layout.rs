//! Keyed post-drop layout transitions. Only mounted, clipped-in items are tracked.
use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc, time::Duration};
use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels,
    Point, Window, point,
};
use gpui::prelude::*;
use super::{VisualTransition, DEFAULT_TRANSITION_DURATION};
use crate::theme::stylesheet::{MotionSource, ResolvedMotion};

/// Shared retained geometry for one collection. Data commits and lifecycle events
/// remain synchronous; this animates existing keyed elements, never cloned views.
pub struct DropLayoutMotion<K>(Rc<RefCell<State<K>>>);
impl<K> Clone for DropLayoutMotion<K> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
struct State<K> {
    current: HashMap<K, Bounds<Pixels>>,
    from: HashMap<K, Bounds<Pixels>>,
    targets: HashMap<K, Bounds<Pixels>>,
    viewport: Option<Bounds<Pixels>>,
    scroll: Point<Pixels>,
    pending: bool,
    transition: VisualTransition,
    inherited: ResolvedMotion,
    duration: Option<Duration>,
    enabled: bool,
}
impl<K> Default for DropLayoutMotion<K> {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(State {
            current: HashMap::new(),
            from: HashMap::new(),
            targets: HashMap::new(),
            viewport: None,
            scroll: Point::default(),
            pending: false,
            transition: VisualTransition::new(1., Duration::ZERO),
            inherited: ResolvedMotion { duration: DEFAULT_TRANSITION_DURATION, source: MotionSource::SdkFallback },
            duration: None,
            enabled: true,
        })))
    }
}
impl<K: Clone + 'static> DropLayoutMotion<K> {
    /// Override this instance's duration; `None` restores its theme value.
    /// Explicit zero disables drop motion independently of other animations.
    pub fn set_duration(&self, duration: Option<Duration>) {
        let mut s = self.0.borrow_mut();
        s.duration = duration;
        Self::settle(&mut s);
    }
    /// Apply a Look snapshot and the control's animation policy.
    pub fn configure(&self, inherited: ResolvedMotion, enabled: bool) {
        let mut s = self.0.borrow_mut();
        if s.inherited != inherited || s.enabled != enabled {
            Self::settle(&mut s);
        }
        s.inherited = inherited;
        s.enabled = enabled;
    }
    pub fn resolved(&self) -> ResolvedMotion {
        let s = self.0.borrow();
        s.inherited.clone().with_override(s.duration, s.enabled)
    }
    /// Call after a changed atomic commit, before the next render. A no-op,
    /// rejection or cancellation must not start a layout transition.
    pub fn start(&self) {
        let duration = self.resolved().duration;
        let mut s = self.0.borrow_mut();
        Self::settle(&mut s);
        if duration.is_zero() {
            return;
        }
        s.from = s.current.clone();
        s.pending = !s.from.is_empty();
        s.transition = VisualTransition::new(0., duration);
    }
    /// Settle on new input/data/presentation changes. Keep last measured geometry
    /// available for a synchronous host-owned drop commit in the same turn.
    pub fn cancel(&self) {
        Self::settle(&mut self.0.borrow_mut());
    }
    fn settle(s: &mut State<K>) {
        s.from.clear();
        s.targets.clear();
        s.pending = false;
        s.transition.snap_to(1.);
    }
}
impl<K: Clone + Eq + Hash + 'static> DropLayoutMotion<K> {
    /// Wrap the collection viewport once. Offsets are sampled during prepaint,
    /// after layout/scroll clamping; scrolling and resize settle active motion.
    pub fn scope(&self, content: impl IntoElement, scroll: impl Fn() -> Point<Pixels> + 'static) -> AnyElement {
        Scope { child: content.into_any_element(), motion: self.clone(), scroll: Box::new(scroll) }.into_any_element()
    }
    /// Wrap one real keyed row/button. Layout remains at its committed position;
    /// paint and hit testing move together from the previously visible position.
    pub fn item(&self, key: K, content: impl IntoElement) -> AnyElement {
        Item { child: content.into_any_element(), key, motion: self.clone() }.into_any_element()
    }
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn elapse_for_test(&self, elapsed: Duration) {
        self.0.borrow_mut().transition.elapse_for_test(elapsed);
    }
    /// Number of visible measured identities retained (at most 512).
    pub fn tracked_items(&self) -> usize {
        self.0.borrow().current.len()
    }

    pub fn is_animating(&self) -> bool {
        let s = self.0.borrow();
        s.pending || s.transition.is_animating()
    }
    // Preserve stateful template roots (focus, style modifiers and native IDs).
    pub(crate) fn begin_surface(
        &self,
        surface: gpui::Stateful<gpui::Div>,
        scroll: impl Fn() -> Point<Pixels> + 'static,
    ) -> gpui::Stateful<gpui::Div> {
        let motion = self.clone();
        surface.child(
            gpui::canvas(move |bounds, _, cx| motion.begin(bounds, scroll(), cx.reduce_motion()), |_, _, _, _| {})
                .absolute()
                .inset_0()
                .size_full(),
        )
    }
    pub(crate) fn end_surface(&self, surface: gpui::Stateful<gpui::Div>) -> gpui::Stateful<gpui::Div> {
        let motion = self.clone();
        surface.child(
            gpui::canvas(move |_, window, _| motion.finish(window), |_, _, _, _| {})
                .absolute()
                .inset_0()
                .size_full(),
        )
    }
    fn finish(&self, window: &mut Window) {
        {
            let mut s = self.0.borrow_mut();
            if s.pending
                || !s
                    .targets
                    .iter()
                    .any(|(key, target)| s.from.get(key).is_some_and(|from| from.origin != target.origin))
            {
                Self::settle(&mut s);
            }
        }
        if self.is_animating() {
            window.on_next_frame(|window, _| window.refresh());
        }
    }
    fn begin(&self, bounds: Bounds<Pixels>, scroll: Point<Pixels>, reduced: bool) {
        let mut s = self.0.borrow_mut();
        if reduced || s.viewport.is_some_and(|old| old != bounds) || s.scroll != scroll {
            Self::settle(&mut s);
        }
        s.viewport = Some(bounds);
        s.scroll = scroll;
        s.current.clear();
        if !s.pending && !s.transition.sync() {
            s.from.clear();
            s.targets.clear();
        }
    }
    fn offset(&self, key: &K, bounds: Bounds<Pixels>, mask: Bounds<Pixels>) -> Point<Pixels> {
        let mut s = self.0.borrow_mut();
        if !bounds.intersects(&mask) || !s.viewport.is_some_and(|v| v.intersects(&bounds)) {
            return Point::default();
        }
        if s.current.len() >= 512 {
            return Point::default();
        }
        s.current.insert(key.clone(), bounds);
        let Some(from) = s.from.get(key).copied() else {
            return Point::default();
        };
        if s.targets.get(key).is_some_and(|target| *target != bounds) {
            Self::settle(&mut s);
            return Point::default();
        }
        s.targets.insert(key.clone(), bounds);
        if s.pending {
            s.pending = false;
            s.transition.set_target(1.);
        }
        let remaining = 1. - s.transition.progress();
        point((from.origin.x - bounds.origin.x) * remaining, (from.origin.y - bounds.origin.y) * remaining)
    }
}
struct Item<K> {
    child: AnyElement,
    key: K,
    motion: DropLayoutMotion<K>,
}
struct Scope<K> {
    child: AnyElement,
    motion: DropLayoutMotion<K>,
    scroll: Box<dyn Fn() -> Point<Pixels>>,
}
macro_rules! transparent_layout {
    () => {
        type RequestLayoutState = ();
        type PrepaintState = ();
        fn id(&self) -> Option<ElementId> {
            None
        }
        fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
            None
        }
        fn request_layout(
            &mut self,
            _: Option<&GlobalElementId>,
            _: Option<&InspectorElementId>,
            window: &mut Window,
            cx: &mut App,
        ) -> (LayoutId, ()) {
            (self.child.request_layout(window, cx), ())
        }
        fn paint(
            &mut self,
            _: Option<&GlobalElementId>,
            _: Option<&InspectorElementId>,
            _: Bounds<Pixels>,
            _: &mut (),
            _: &mut (),
            window: &mut Window,
            cx: &mut App,
        ) {
            self.child.paint(window, cx);
        }
    };
}
impl<K: Clone + Eq + Hash + 'static> Element for Item<K> {
    transparent_layout!();
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let offset = self.motion.offset(&self.key, bounds, window.content_mask().bounds);
        window.with_element_offset(offset, |window| {
            self.child.prepaint(window, cx);
        });
    }
}
impl<K: Clone + Eq + Hash + 'static> Element for Scope<K> {
    transparent_layout!();
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.motion.begin(bounds, (self.scroll)(), cx.reduce_motion());
        self.child.prepaint(window, cx);
        self.motion.finish(window);
    }
}
impl<K: Clone + Eq + Hash + 'static> IntoElement for Item<K> {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl<K: Clone + Eq + Hash + 'static> IntoElement for Scope<K> {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Context, Entity, Render, TestAppContext, VisualTestContext, div, px};
    struct Page {
        order: Vec<u32>,
        motion: DropLayoutMotion<u32>,
        scroll: Point<Pixels>,
        width: f32,
    }
    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let rows = div().w(gpui::relative(1.)).flex().flex_col().children(self.order.iter().map(|key| {
                self.motion.item(
                    *key,
                    div()
                        .id(("row", *key))
                        .w_full()
                        .h(px(30.))
                        .bg(if *key == 0 {
                            gpui::rgb(0xff0000)
                        } else {
                            gpui::rgb(0x0000ff)
                        })
                        .debug_selector({
                            let key = *key;
                            move || format!("row-{key}")
                        }),
                )
            }));
            let scroll = self.scroll;
            div().w(px(self.width)).h(px(90.)).overflow_hidden().child(self.motion.scope(rows, move || scroll))
        }
    }
    fn setup(app: &mut TestAppContext) -> (Entity<Page>, &mut VisualTestContext) {
        let (page, cx) = app.add_window_view(|_, _| Page {
            order: vec![0, 1, 2],
            motion: DropLayoutMotion::default(),
            scroll: Point::default(),
            width: 120.,
        });
        cx.run_until_parked();
        (page, cx)
    }
    fn frame(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.simulate_next_frame(cx);
        });
        cx.run_until_parked();
    }
    fn red_y(cx: &mut VisualTestContext) -> f32 {
        cx.update(|window, _| {
            window
                .painted_quads()
                .iter()
                .find(|q| q.background.as_solid() == Some(gpui::Hsla::from(gpui::rgb(0xff0000))))
                .unwrap()
                .bounds
                .origin
                .y
                .as_f32()
                / window.scale_factor()
        })
    }
    #[test]
    fn painted_positions_interpolate_and_finish_without_cloning_rows() {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app);
        assert_eq!(red_y(cx), 0.);
        page.update(cx, |page, cx| {
            page.motion.start();
            page.order = vec![1, 2, 0];
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(red_y(cx), 0.);
        page.update(cx, |p, _| p.motion.elapse_for_test(Duration::from_millis(100)));
        frame(cx);
        let y = red_y(cx);
        assert!(y > 0. && y < 60., "intermediate {y}");
        cx.update(|window, _| {
            assert_eq!(
                window
                    .painted_quads()
                    .iter()
                    .filter(|q| q.background.as_solid() == Some(gpui::Hsla::from(gpui::rgb(0xff0000))))
                    .count(),
                1
            )
        });
        page.update(cx, |p, _| p.motion.elapse_for_test(Duration::from_millis(250)));
        frame(cx);
        assert_eq!(red_y(cx), 60.);
        cx.update(|_, cx| assert!(!page.read(cx).motion.is_animating()));
    }
    #[test]
    fn reduced_disabled_scroll_resize_and_cancel_settle() {
        for mode in 0..5 {
            let mut app = TestAppContext::single();
            let (page, cx) = setup(&mut app);
            if mode == 0 {
                cx.update(|_, cx| cx.set_reduce_motion(true));
            }
            page.update(cx, |p, cx| {
                if mode == 1 {
                    p.motion.set_duration(Some(Duration::ZERO));
                }
                p.motion.start();
                p.order = vec![1, 2, 0];
                cx.notify();
            });
            cx.run_until_parked();
            page.update(cx, |p, cx| {
                match mode {
                    2 => p.scroll.y = px(-10.),
                    3 => p.width = 130.,
                    4 => p.motion.cancel(),
                    _ => {}
                }
                cx.notify();
            });
            frame(cx);
            assert_eq!(red_y(cx), 60., "mode {mode}");
            cx.update(|_, cx| assert!(!page.read(cx).motion.is_animating()));
        }
    }
    #[test]
    fn unchanged_layout_and_unseen_items_do_not_schedule_motion() {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app);
        page.update(cx, |p, cx| {
            p.motion.start();
            cx.notify();
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert!(!page.read(cx).motion.is_animating()));
        page.update(cx, |p, cx| {
            p.motion.start();
            p.order = vec![3, 4, 5];
            cx.notify();
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert!(!page.read(cx).motion.is_animating()));
    }
    #[test]
    fn visible_cache_is_bounded_and_configuration_restores_the_live_look() {
        let motion = DropLayoutMotion::<u32>::default();
        let bounds = Bounds::new(point(px(0.), px(0.)), gpui::size(px(120.), px(90.)));
        motion.begin(bounds, Point::default(), false);
        for key in 0..10_000 {
            motion.offset(&key, bounds, bounds);
        }
        assert_eq!(motion.tracked_items(), 512);
        let original = motion.resolved();
        motion.set_duration(Some(Duration::from_millis(800)));
        motion.configure(
            ResolvedMotion {
                duration: Duration::from_millis(350),
                source: MotionSource::Look { name: "Test".into(), field: "common.drag_drop.drop_motion_ms" },
            },
            true,
        );
        assert_eq!(motion.resolved().duration, Duration::from_millis(800));
        motion.set_duration(None);
        assert_eq!(motion.resolved().duration, Duration::from_millis(350));
        motion.configure(original, false);
        assert!(motion.resolved().duration.is_zero());
        let weak = Rc::downgrade(&motion.0);
        drop(motion);
        assert!(weak.upgrade().is_none());
    }
}
