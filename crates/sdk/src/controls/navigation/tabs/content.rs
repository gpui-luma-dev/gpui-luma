//! Content presenters and independently placeable slots of one tabs component.

use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, Subscription, WeakEntity, Window, div, prelude::*};

use super::Tabs;

type ContentPresenter = Arc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// Rebuilds a panel's elements while retaining any entities captured by the presenter.
#[derive(Clone)]
pub struct TabsContent(ContentPresenter);

impl TabsContent {
    pub fn new<F, E>(presenter: F) -> Self
    where
        F: Fn(&mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        Self(Arc::new(move |window, cx| presenter(window, cx).into_any_element()))
    }

    pub(super) fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        (self.0)(window, cx)
    }
}

/// Navigation slot. Created and owned by `Tabs`; render via `Tabs::tab_list`.
pub(super) struct TabsList {
    tabs: WeakEntity<Tabs>,
    _subscription: Subscription,
}

impl TabsList {
    pub(super) fn new(tabs: &Entity<Tabs>, cx: &mut Context<Self>) -> Self {
        Self { tabs: tabs.downgrade(), _subscription: cx.observe(tabs, |_, _, cx| cx.notify()) }
    }
}

impl Render for TabsList {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div();
        if let Some(tabs) = self.tabs.upgrade() {
            let group = tabs.update(cx, |tabs, cx| {
                tabs.sync_list_motion(window, cx);
                tabs.group.clone()
            });
            root = root.child(group);
        }
        root
    }
}

/// Selected-content slot. Shares selection and lifetime with its owning `Tabs`.
pub(super) struct TabsBody {
    tabs: WeakEntity<Tabs>,
    _subscription: Subscription,
}

impl TabsBody {
    pub(super) fn new(tabs: &Entity<Tabs>, cx: &mut Context<Self>) -> Self {
        Self { tabs: tabs.downgrade(), _subscription: cx.observe(tabs, |_, _, cx| cx.notify()) }
    }
}

impl Render for TabsBody {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div().w_full().min_h_0().flex_1().flex().flex_col();
        if let Some(tabs) = self.tabs.upgrade() {
            let (opacity, content) = tabs.update(cx, |tabs, cx| {
                tabs.body_transition.sync();
                tabs.body_transition.schedule_frame(window, cx);
                let content = tabs.active_id.as_ref().and_then(|id| tabs.contents.get(id)).cloned();
                (tabs.body_transition.progress(), content)
            });
            root = root.opacity(opacity);
            if let Some(content) = content {
                root = root.child(content.render(window, cx));
            }
        }
        root
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use std::time::Duration;
    use gpui::{TestAppContext, px};
    use crate::controls::tabs::{TabsItem};

    struct Panel {
        value: usize,
    }
    impl Render for Panel {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().id("retained-panel").debug_selector(|| "retained-panel".into()).child(self.value.to_string())
        }
    }

    struct Page {
        tabs: Entity<Tabs>,
        split: bool,
    }
    impl Render for Page {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let root = div().size_full().flex().flex_col();
            if self.split {
                root.child(self.tabs.read(cx).tab_list()).child(self.tabs.read(cx).body())
            } else {
                root.child(self.tabs.clone())
            }
        }
    }

    #[test]
    fn stacked_and_split_slots_route_content_and_preserve_entities() {
        for split in [false, true] {
            let mut app = TestAppContext::single();
            let (page, cx) = app.add_window_view(|_, cx| {
                let panel = cx.new(|_| Panel { value: 7 });
                let tabs = Tabs::new("content-tabs")
                    .tab("first", "First", panel)
                    .tab_with("second", "Second", |_, _| div().debug_selector(|| "second-panel".into()).child("Second"))
                    .fade_in(Duration::from_millis(300))
                    .spawn(cx);
                Page { tabs, split }
            });
            cx.simulate_resize(gpui::size(px(500.0), px(300.0)));
            cx.run_until_parked();
            assert!(cx.debug_bounds("retained-panel").is_some());
            assert!(cx.debug_bounds("second-panel").is_none());
            let tabs = cx.update(|_, cx| page.read(cx).tabs.clone());
            cx.update(|_, cx| {
                tabs.update(cx, |tabs, cx| {
                    assert_eq!(tabs.body_transition.progress(), 1.0);
                    tabs.set_active("second", cx);
                    assert_eq!(tabs.body_transition.progress(), 0.0);
                    assert!(tabs.body_transition.is_animating());
                    tabs.set_active("second", cx);
                    assert!(tabs.body_transition.is_animating());
                    tabs.set_animated(false, cx);
                    assert_eq!(tabs.body_transition.progress(), 1.0);
                    assert!(!tabs.body_transition.is_animating());
                })
            });
            cx.run_until_parked();
            assert!(cx.debug_bounds("second-panel").is_some());
            assert!(cx.debug_bounds("retained-panel").is_none());
            cx.update(|_, cx| tabs.update(cx, |tabs, cx| tabs.set_active("first", cx)));
            cx.run_until_parked();
            assert!(cx.debug_bounds("retained-panel").is_some());
        }
    }

    #[test]
    fn disabled_animation_and_zero_duration_select_without_fading() {
        for (animated, duration) in [(false, Duration::from_millis(300)), (true, Duration::ZERO)] {
            let app = TestAppContext::single();
            let tabs = app.update(|cx| {
                Tabs::new("instant-tabs")
                    .tab_with("one", "One", |_, _| div())
                    .tab_with("two", "Two", |_, _| div())
                    .animated(animated)
                    .fade_in(duration)
                    .spawn(cx)
            });
            app.update(|cx| {
                tabs.update(cx, |tabs, cx| {
                    tabs.set_active("two", cx);
                    assert_eq!(tabs.active_id().map(|id| id.as_ref()), Some("two"));
                    assert_eq!(tabs.body_transition.progress(), 1.0);
                    assert!(!tabs.body_transition.is_animating());
                })
            });
        }
    }

    #[test]
    fn inherited_motion_survives_runtime_disable_and_configuration_edits() {
        use crate::theme::stylesheet::{CommonStylesheet, MotionSource};
        let app = TestAppContext::single();
        let mut stylesheet = CommonStylesheet::parse("[common.tabs.content]\nfade_in_ms = 300").unwrap();
        let tabs = app.update(|cx| {
            Tabs::new("inherited-tabs")
                .stylesheet(&stylesheet, "test")
                .tab_with("one", "One", |_, _| div())
                .tab_with("two", "Two", |_, _| div())
                .animated(false)
                .spawn(cx)
        });
        stylesheet.tabs.content.fade_in_ms = Some(0);
        assert_eq!(stylesheet.tabs_content_motion("test").duration, Duration::ZERO);
        app.update(|cx| {
            tabs.update(cx, |tabs, cx| {
                assert_eq!(tabs.body_motion().source, MotionSource::AnimationDisabled);
                tabs.set_active("two", cx);
                assert!(!tabs.body_transition.is_animating());
                tabs.set_animated(true, cx);
                assert_eq!(tabs.body_motion().duration, Duration::from_millis(300));
                assert!(matches!(tabs.body_motion().source, MotionSource::Look { .. }));
                tabs.set_active("one", cx);
                assert_eq!(tabs.body_transition.progress(), 0.0);
                assert!(tabs.body_transition.is_animating());
                tabs.set_animated(false, cx);
                assert_eq!(tabs.body_transition.progress(), 1.0);
                assert_eq!(tabs.body_motion().duration, Duration::ZERO);
            });
        });
    }

    #[test]
    fn user_selection_and_rapid_changes_restart_only_the_incoming_body() {
        let app = TestAppContext::single();
        let tabs = app.update(|cx| {
            Tabs::new("motion-tabs")
                .tab_with("one", "One", |_, _| div())
                .tab_content(TabsItem::new("disabled").enabled(false), TabsContent::new(|_, _| div()))
                .tab_with("two", "Two", |_, _| div())
                .fade_in(Duration::from_millis(300))
                .spawn(cx)
        });
        app.update(|cx| {
            tabs.update(cx, |tabs, cx| {
                let event = crate::controls::control_group::ControlGroupEvent::Change {
                    changed_id: "two".into(),
                    selected: true,
                    selected_ids: vec!["two".into()],
                };
                tabs.handle_group_event(&event, cx);
                assert_eq!(tabs.active_id().map(|id| id.as_ref()), Some("two"));
                assert!(tabs.body_transition.is_animating());
                tabs.set_active("disabled", cx);
                assert_eq!(tabs.active_id().map(|id| id.as_ref()), Some("two"));
                tabs.set_active("one", cx);
                assert_eq!(tabs.body_transition.progress(), 0.0);
                assert!(tabs.body_transition.is_animating());
                tabs.set_items([TabsItem::new("two")], cx);
                assert_eq!(tabs.active_id().map(|id| id.as_ref()), Some("two"));
                assert_eq!(tabs.body_transition.progress(), 1.0);
            })
        });
    }
}
