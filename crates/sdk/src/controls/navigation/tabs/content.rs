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
    use gpui::{SharedString, TestAppContext, px};
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
    fn completed_fades_stop_scheduling_frames_in_both_layouts() {
        for split in [false, true] {
            let mut app = TestAppContext::single();
            let (page, cx) = app.add_window_view(|_, cx| Page {
                tabs: Tabs::new("frame-tabs")
                    .tab_with("one", "One", |_, _| div())
                    .tab_with("two", "Two", |_, _| div())
                    .fade_in(Duration::from_millis(200))
                    .spawn(cx),
                split,
            });
            cx.simulate_resize(gpui::size(px(500.0), px(300.0)));
            cx.run_until_parked();
            assert_eq!(cx.update(|window, cx| window.simulate_next_frame(cx)), 0);
            let tabs = cx.update(|_, cx| page.read(cx).tabs.clone());
            cx.update(|_, cx| {
                tabs.update(cx, |tabs, cx| {
                    tabs.set_active("two", cx);
                    tabs.set_active("one", cx);
                    tabs.set_active("two", cx);
                })
            });
            cx.run_until_parked();
            assert!(cx.update(|window, cx| window.simulate_next_frame(cx)) > 0);
            // Motion uses wall time rather than the test executor's virtual clock.
            std::thread::sleep(Duration::from_millis(250));
            cx.run_until_parked();
            cx.update(|window, cx| window.simulate_next_frame(cx));
            cx.run_until_parked();
            assert_eq!(cx.update(|window, cx| window.simulate_next_frame(cx)), 0);
            cx.update(|_, cx| {
                tabs.update(cx, |tabs, cx| {
                    assert_eq!(tabs.active_id().map(|id| id.as_ref()), Some("two"));
                    assert_eq!(tabs.body_transition.progress(), 1.0);
                    tabs.set_active("two", cx);
                    tabs.refresh_content(cx);
                })
            });
            cx.run_until_parked();
            assert_eq!(cx.update(|window, cx| window.simulate_next_frame(cx)), 0);
        }
    }

    #[test]
    fn mouse_keyboard_and_programmatic_selection_share_the_same_state() {
        use gpui::Focusable;
        use super::super::TabsEvent;
        for split in [false, true] {
            let mut app = TestAppContext::single();
            app.update(crate::key_handling::bind_default_control_keys);
            let tabs = app.update(|cx| {
                Tabs::new("input-tabs")
                    .tab_with("one", "One", |_, _| div())
                    .tab_content(TabsItem::new("disabled").enabled(false), TabsContent::new(|_, _| div()))
                    .tab_with("two", "Two", |_, _| div())
                    .animated(false)
                    .spawn(cx)
            });
            let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
            let observed = events.clone();
            let _subscription = app.update(|cx| {
                cx.subscribe(&tabs, move |_, event: &TabsEvent, _| {
                    observed.borrow_mut().push(event.clone());
                })
            });
            let (_, cx) = app.add_window_view(|window, _| {
                window.activate_window();
                Page { tabs: tabs.clone(), split }
            });
            cx.simulate_resize(gpui::size(px(500.0), px(300.0)));
            cx.run_until_parked();
            let bounds = events
                .borrow()
                .iter()
                .find_map(|event| match event {
                    TabsEvent::ItemBoundsChanged { tab_id, bounds } if tab_id == "two" => Some(*bounds),
                    _ => None,
                })
                .expect("second tab bounds");
            events.borrow_mut().clear();
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
            let selection_events = || {
                events
                    .borrow()
                    .iter()
                    .filter_map(|event| match event {
                        TabsEvent::Change { tab_id, .. } => Some(("change", tab_id.to_string())),
                        TabsEvent::Activate { tab_id, .. } => Some(("activate", tab_id.to_string())),
                        TabsEvent::Reactivate { tab_id, .. } => Some(("reactivate", tab_id.to_string())),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(selection_events(), vec![("change", "two".into()), ("activate", "two".into())]);
            events.borrow_mut().clear();
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
            assert_eq!(selection_events(), vec![("reactivate", "two".into()), ("activate", "two".into())]);
            events.borrow_mut().clear();
            // Programmatic selection must also move the keyboard navigation anchor.
            cx.update(|window, cx| {
                tabs.update(cx, |tabs, cx| tabs.set_active("one", cx));
                tabs.read(cx).focus_handle(cx).focus(window, cx);
            });
            cx.run_until_parked();
            assert!(selection_events().is_empty());
            cx.simulate_keystrokes("right");
            cx.run_until_parked();
            assert_eq!(cx.update(|_, cx| tabs.read(cx).active_id().cloned()), Some("two".into()));
            assert_eq!(selection_events(), vec![("change", "two".into()), ("activate", "two".into())]);
            events.borrow_mut().clear();
            cx.simulate_keystrokes("enter");
            cx.run_until_parked();
            assert_eq!(selection_events(), vec![("reactivate", "two".into()), ("activate", "two".into())]);
        }
    }

    struct ScrollPanel {
        focus: gpui::FocusHandle,
        scroll: gpui::ScrollHandle,
        value: usize,
    }

    impl Render for ScrollPanel {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .id("scroll-panel")
                .debug_selector(|| "scroll-panel".into())
                .track_focus(&self.focus)
                .h(px(100.0))
                .w_full()
                .overflow_y_scroll()
                .track_scroll(&self.scroll)
                .child(div().h(px(1000.0)).child(self.value.to_string()))
        }
    }

    #[test]
    fn retained_pages_preserve_scroll_and_support_explicit_focus_restoration() {
        use gpui::Focusable;
        for split in [false, true] {
            let mut app = TestAppContext::single();
            let panel = app.update(|cx| {
                cx.new(|cx| ScrollPanel { focus: cx.focus_handle(), scroll: gpui::ScrollHandle::new(), value: 7 })
            });
            let (page, cx) = app.add_window_view(|_, cx| Page {
                tabs: Tabs::new("scroll-tabs")
                    .tab("one", "One", panel.clone())
                    .tab_with("two", "Two", |_, _| div())
                    .animated(false)
                    .spawn(cx),
                split,
            });
            cx.simulate_resize(gpui::size(px(500.0), px(300.0)));
            cx.run_until_parked();
            let tabs = cx.update(|_, cx| page.read(cx).tabs.clone());
            cx.update(|window, cx| {
                panel.update(cx, |panel, cx| {
                    panel.value = 42;
                    panel.scroll.set_offset(gpui::point(px(0.0), px(-200.0)));
                    panel.focus.focus(window, cx);
                    cx.notify();
                });
            });
            cx.run_until_parked();
            cx.update(|window, cx| {
                assert!(panel.read(cx).focus.is_focused(window));
                tabs.read(cx).focus_handle(cx).focus(window, cx);
                tabs.update(cx, |tabs, cx| tabs.set_active("two", cx));
            });
            cx.run_until_parked();
            assert!(cx.debug_bounds("scroll-panel").is_none());
            cx.update(|window, cx| {
                tabs.update(cx, |tabs, cx| tabs.set_active("one", cx));
                assert!(tabs.read(cx).focus_handle(cx).is_focused(window));
            });
            cx.run_until_parked();
            assert!(cx.debug_bounds("scroll-panel").is_some());
            cx.update(|window, cx| {
                assert_eq!(panel.read(cx).value, 42);
                assert_eq!(panel.read(cx).scroll.offset().y, px(-200.0));
                assert!(!panel.read(cx).focus.is_focused(window));
                // The app chooses when focus enters its page, using the same handle.
                panel.read(cx).focus.clone().focus(window, cx);
                assert!(panel.read(cx).focus.is_focused(window));
            });
        }
    }

    #[test]
    fn runtime_content_updates_and_refresh_preserve_selection_and_motion() {
        for split in [false, true] {
            let mut app = TestAppContext::single();
            let value = std::rc::Rc::new(std::cell::Cell::new(1));
            let rendered = std::rc::Rc::new(std::cell::Cell::new(0));
            let (page, cx) = app.add_window_view(|_, cx| {
                let value = value.clone();
                let rendered = rendered.clone();
                let tabs = Tabs::new("runtime-content")
                    .tab_with("one", "One", move |_, _| {
                        rendered.set(value.get());
                        div().debug_selector(|| "original-panel".into())
                    })
                    .item(TabsItem::new("two"))
                    .fade_in(Duration::ZERO)
                    .spawn(cx);
                Page { tabs, split }
            });
            cx.simulate_resize(gpui::size(px(500.0), px(300.0)));
            cx.run_until_parked();
            assert_eq!(rendered.get(), 1);
            let tabs = cx.update(|_, cx| page.read(cx).tabs.clone());
            value.set(2);
            cx.update(|_, cx| tabs.update(cx, |tabs, cx| tabs.refresh_content(cx)));
            cx.run_until_parked();
            assert_eq!(rendered.get(), 2);
            cx.update(|_, cx| {
                tabs.update(cx, |tabs, cx| {
                    assert!(!tabs.set_content("missing", TabsContent::new(|_, _| div()), cx));
                    assert!(tabs.set_content(
                        "one",
                        TabsContent::new(|_, _| { div().debug_selector(|| "replacement-panel".into()) }),
                        cx
                    ));
                    assert_eq!(tabs.active_id().map(|id| id.as_ref()), Some("one"));
                    assert_eq!(tabs.body_transition.progress(), 1.0);
                })
            });
            cx.run_until_parked();
            assert!(cx.debug_bounds("replacement-panel").is_some());
            assert!(cx.debug_bounds("original-panel").is_none());
            cx.update(|_, cx| {
                tabs.update(cx, |tabs, cx| {
                    assert!(tabs.remove_content("one", cx));
                    assert!(!tabs.remove_content("one", cx));
                    assert_eq!(tabs.active_id().map(|id| id.as_ref()), Some("one"));
                })
            });
            cx.run_until_parked();
            assert!(cx.debug_bounds("replacement-panel").is_none());
            // A navigation-only stacked control must attach its body again.
            cx.update(|_, cx| {
                tabs.update(cx, |tabs, cx| {
                    assert!(tabs.set_content(
                        "one",
                        TabsContent::new(|_, _| { div().debug_selector(|| "reattached-panel".into()) }),
                        cx
                    ));
                })
            });
            cx.run_until_parked();
            assert!(cx.debug_bounds("reattached-panel").is_some());
        }
    }

    #[test]
    fn item_updates_retain_view_state_and_programmatic_selection_is_silent() {
        let app = TestAppContext::single();
        let panel = app.update(|cx| cx.new(|_| Panel { value: 7 }));
        let tabs = app.update(|cx| {
            Tabs::new("retention")
                .tab("one", "One", panel.clone())
                .tab_with("two", "Two", |_, _| div())
                .spawn(cx)
        });
        let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let observed = events.clone();
        let _subscription = app.update(|cx| {
            cx.subscribe(&tabs, move |_, event: &super::super::TabsEvent, _| {
                observed.borrow_mut().push(event.clone());
            })
        });
        app.update(|cx| {
            panel.update(cx, |panel, _| panel.value = 42);
            tabs.update(cx, |tabs, cx| {
                tabs.set_active("two", cx);
                tabs.set_items([TabsItem::new("one"), TabsItem::new("two").enabled(false)], cx);
                assert_eq!(tabs.active_id().map(|id| id.as_ref()), Some("one"));
                assert!(tabs.contents.contains_key(&SharedString::from("one")));
                assert!(tabs.contents.contains_key(&SharedString::from("two")));
                tabs.set_items([TabsItem::new("two")], cx);
                assert!(!tabs.contents.contains_key(&SharedString::from("one")));
                assert_eq!(tabs.active_id().map(|id| id.as_ref()), Some("two"));
                tabs.set_items([TabsItem::new("two").enabled(false)], cx);
                assert!(tabs.active_id().is_none());
                tabs.set_items([], cx);
                assert!(tabs.contents.is_empty());
                assert!(tabs.active_id().is_none());
            });
            assert_eq!(panel.read(cx).value, 42);
        });
        app.run_until_parked();
        assert!(events.borrow().is_empty());
    }

    #[test]
    fn runtime_motion_changes_do_not_restart_running_fades() {
        use crate::theme::stylesheet::{CommonStylesheet, MotionSource};
        let app = TestAppContext::single();
        let stylesheet = CommonStylesheet::parse("[common.tabs.content]\nfade_in_ms = 300").unwrap();
        let tabs = app.update(|cx| {
            Tabs::new("runtime-motion")
                .stylesheet(&stylesheet, "test")
                .tab_with("one", "One", |_, _| div())
                .tab_with("two", "Two", |_, _| div())
                .spawn(cx)
        });
        app.update(|cx| {
            tabs.update(cx, |tabs, cx| {
                tabs.set_active("two", cx);
                let transition = format!("{:?}", tabs.body_transition);
                tabs.set_active("two", cx);
                tabs.set_active("missing", cx);
                tabs.refresh_content(cx);
                tabs.set_content("two", TabsContent::new(|_, _| div()), cx);
                tabs.set_fade_in(Some(Duration::ZERO), cx);
                assert_eq!(format!("{:?}", tabs.body_transition), transition);
                assert_eq!(tabs.body_motion().source, MotionSource::InstanceOverride);
                tabs.set_active("one", cx);
                assert!(!tabs.body_transition.is_animating());
                tabs.set_fade_in(None, cx);
                assert_eq!(tabs.body_motion().duration, Duration::from_millis(300));
                tabs.set_active("two", cx);
                assert!(tabs.body_transition.is_animating());
                tabs.set_animated(false, cx);
                assert!(!tabs.body_transition.is_animating());
                tabs.set_fade_in(Some(Duration::from_millis(50)), cx);
                assert_eq!(tabs.body_motion().source, MotionSource::AnimationDisabled);
                tabs.set_animated(true, cx);
                assert!(!tabs.body_transition.is_animating());
                assert_eq!(tabs.body_motion().duration, Duration::from_millis(50));
            })
        });
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
