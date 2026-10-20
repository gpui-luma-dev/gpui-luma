//! Content presenters and independently placeable slots of one tabs component.

use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, Subscription, WeakEntity, Window, div, prelude::*};

use super::Tabs;
use crate::infra::element_ext::ElementExt;

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
    viewport: std::rc::Rc<std::cell::Cell<Option<gpui::Bounds<gpui::Pixels>>>>,
    panel_focus: std::collections::HashMap<gpui::SharedString, gpui::FocusHandle>,
    renderer: Option<Box<dyn super::TabsTransitionRenderer>>,
}

impl TabsBody {
    pub(super) fn new(tabs: &Entity<Tabs>, cx: &mut Context<Self>) -> Self {
        Self {
            tabs: tabs.downgrade(),
            _subscription: cx.observe(tabs, |_, _, cx| cx.notify()),
            viewport: Default::default(),
            panel_focus: Default::default(),
            renderer: None,
        }
    }

    fn panel(
        &mut self,
        slot: (gpui::SharedString, TabsContent),
        outgoing: bool,
        layout: super::TabsContentLayout,
        navigation: gpui::FocusHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> super::TabsTransitionPanel {
        let (id, content) = slot;
        let focus = self.panel_focus.entry(id.clone()).or_insert_with(|| cx.focus_handle());
        let mut element = div()
            .id((id.clone(), usize::MAX))
            .relative()
            .w_full()
            .flex()
            .flex_col()
            .when(layout == super::TabsContentLayout::Fill, |root| root.h_full())
            .track_focus(focus)
            .child(content.render(window, cx));
        if outgoing {
            element = element
                .capture_key_down(move |_, window, cx| {
                    navigation.focus(window, cx);
                    cx.stop_propagation();
                })
                .child(div().absolute().inset_0().occlude());
        }
        super::TabsTransitionPanel { id, element: element.into_any_element() }
    }
}

impl Render for TabsBody {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use gpui::Focusable;
        let mut root = div().id("tabs-body").w_full().flex().flex_col();
        if let Some(tabs) = self.tabs.upgrade() {
            let (progress, content, outgoing, direction, config, active_id, animating, changes, layout, navigation) =
                tabs.update(cx, |tabs, cx| {
                    tabs.body_transition.sync();
                    let animating = tabs.body_transition.is_animating();
                    if !animating {
                        tabs.outgoing_content.clear();
                    }
                    let content = tabs.active_id.as_ref().and_then(|id| tabs.contents.get(id)).cloned();
                    (
                        tabs.body_transition.progress().clamp(0.0, 1.0),
                        content,
                        tabs.outgoing_content.clone(),
                        tabs.content_direction,
                        tabs.content_transition.clone(),
                        tabs.active_id.clone(),
                        animating,
                        std::mem::take(&mut tabs.transition_changes),
                        tabs.content_layout,
                        tabs.group.read(cx).focus_handle(cx),
                    )
                });
            // Animation frames invalidate the body slot, never the navigation owner.
            if animating {
                cx.on_next_frame(window, |_, _, cx| cx.notify());
            }
            if layout != super::TabsContentLayout::Content {
                root = root.flex_1().min_h_0();
            }
            if layout == super::TabsContentLayout::Fill {
                root = root.relative().overflow_hidden();
            }
            if let Some(config) = config {
                let navigation = active_id
                    .as_ref()
                    .and_then(|id| config.focus.as_ref().and_then(|target| target(id, window, cx)))
                    .unwrap_or(navigation);
                if self
                    .panel_focus
                    .iter()
                    .any(|(id, focus)| Some(id) != active_id.as_ref() && focus.contains_focused(window, cx))
                {
                    navigation.focus(window, cx);
                }
                self.panel_focus.retain(|id, _| {
                    Some(id) == active_id.as_ref() || outgoing.iter().any(|(previous, _)| previous == id)
                });
                if self.renderer.is_none() {
                    self.renderer = Some(config.create());
                }
                if let Some(renderer) = &mut self.renderer {
                    for change in changes {
                        renderer.selection_changed(change);
                    }
                }
                let outgoing = outgoing
                    .into_iter()
                    .map(|(id, content)| self.panel((id, content), true, layout, navigation.clone(), window, cx))
                    .collect();
                let incoming = active_id
                    .zip(content)
                    .map(|(id, content)| self.panel((id, content), false, layout, navigation, window, cx));
                let frame = super::TabsTransitionFrame {
                    incoming,
                    outgoing,
                    progress,
                    direction,
                    animating,
                    viewport: if config.measure { self.viewport.get() } else { None },
                };
                if config.measure {
                    let viewport = self.viewport.clone();
                    let body = cx.weak_entity();
                    root = root.relative().on_prepaint(move |bounds, window, cx| {
                        if viewport.replace(Some(bounds)) != Some(bounds) {
                            let _ = body.update(cx, |_, cx| cx.on_next_frame(window, |_, _, cx| cx.notify()));
                        }
                    });
                }
                if let Some(renderer) = &mut self.renderer {
                    root = root.child(renderer.render(frame, window, cx));
                }
            } else {
                root = root.opacity(progress);
                if let Some(content) = content {
                    root = root.child(content.render(window, cx));
                }
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

    #[test]
    fn captured_content_role_survives_disable_and_later_configuration_edits() {
        use crate::motion::{Transition, Easing};
        let app = TestAppContext::single();
        let mut defaults = crate::theme::MotionDefaults::default().tabs;
        defaults.content = Transition::timed(Duration::from_millis(300), Easing::Linear);
        let expected = defaults.content.description();
        let tabs = app.update(|cx| {
            Tabs::new("snapshot")
                .motion_defaults(defaults.clone())
                .motion_enabled(false)
                .tab_with("one", "One", |_, _| div())
                .tab_with("two", "Two", |_, _| div())
                .spawn(cx)
        });
        defaults.content = Transition::instant();
        app.update(|cx| {
            tabs.update(cx, |tabs, cx| {
                assert_eq!(tabs.motion_profile().content.description(), expected);
                tabs.set_active("two", cx);
                assert!(!tabs.body_transition.is_animating());
                tabs.set_motion_enabled(true, cx);
                tabs.set_active("one", cx);
                assert!(tabs.body_transition.is_animating());
                let running = format!("{:?}", tabs.body_transition);
                tabs.set_active("one", cx);
                tabs.set_active("missing", cx);
                tabs.refresh_content(cx);
                assert_eq!(format!("{:?}", tabs.body_transition), running);
                tabs.set_motion_enabled(false, cx);
                assert!(!tabs.body_transition.is_animating());
                assert_eq!(tabs.body_transition.progress(), 1.0);
                assert_eq!(tabs.motion_profile().content.description(), expected);
            })
        });
    }

    #[test]
    fn role_profile_drives_content_and_animation_disable_wins() {
        let app = TestAppContext::single();
        let tabs = app.update(|cx| {
            Tabs::new("role-motion")
                .motion(|m| m.content = crate::motion::Transition::spring(200.0, 18.0))
                .tab_with("one", "One", |_, _| div())
                .tab_with("two", "Two", |_, _| div())
                .spawn(cx)
        });
        app.update(|cx| {
            tabs.update(cx, |tabs, cx| {
                tabs.set_active("two", cx);
                tabs.body_transition.elapse_for_test(Duration::from_millis(100));
                assert!(tabs.body_transition.sync());
                assert!(tabs.body_transition.state().velocity > 0.0);
                tabs.body_transition.elapse_for_test(Duration::from_secs(10));
                assert!(!tabs.body_transition.sync());
                assert_eq!(tabs.body_transition.progress(), 1.0);
                tabs.set_motion_enabled(false, cx);
                tabs.set_active("one", cx);
                assert!(!tabs.body_transition.is_animating());
                assert_eq!(tabs.body_transition.progress(), 1.0);
                tabs.set_motion_enabled(true, cx);
                tabs.set_active("two", cx);
                assert!(tabs.body_transition.is_animating());
            })
        });
    }

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
                    .motion(move |m| {
                        m.content = crate::motion::Transition::timed(
                            Duration::from_millis(300),
                            crate::motion::Easing::OutCubic,
                        )
                    })
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
                    tabs.set_motion_enabled(false, cx);
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
                    .motion(move |m| {
                        m.content = crate::motion::Transition::timed(
                            Duration::from_millis(200),
                            crate::motion::Easing::OutCubic,
                        )
                    })
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
                    .motion_enabled(false)
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
                    .motion_enabled(false)
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
                    .motion(move |m| {
                        m.content = crate::motion::Transition::timed(Duration::ZERO, crate::motion::Easing::OutCubic)
                    })
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

    struct BodyOnly {
        tabs: Entity<Tabs>,
    }
    impl Render for BodyOnly {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().flex().flex_col().child(self.tabs.read(cx).body())
        }
    }

    #[test]
    fn custom_frames_notify_only_body_and_default_measurement_adds_no_frame() {
        let mut app = TestAppContext::single();
        let renders = std::rc::Rc::new(std::cell::Cell::new(0));
        let changes = std::rc::Rc::new(std::cell::Cell::new(0));
        let (page, cx) = app.add_window_view(|_, cx| {
            let renders = renders.clone();
            BodyOnly {
                tabs: Tabs::new("body-frames")
                    .motion(|m| {
                        m.content =
                            crate::motion::Transition::timed(Duration::from_millis(200), crate::motion::Easing::Linear)
                    })
                    .content_layout(super::super::TabsContentLayout::Fill)
                    .content_transition(super::super::TabsContentTransition::new(move |frame, _, _| {
                        renders.set(renders.get() + 1);
                        assert!(frame.viewport.is_none());
                        div().size_full().children(frame.incoming.map(|panel| panel.element))
                    }))
                    .tab_with("one", "One", |_, _| div())
                    .tab_with("two", "Two", |_, _| div())
                    .spawn(cx),
            }
        });
        cx.run_until_parked();
        assert_eq!(cx.update(|window, cx| window.simulate_next_frame(cx)), 0);
        let tabs = cx.update(|_, cx| page.read(cx).tabs.clone());
        let _observer = cx.update(|_, cx| {
            cx.observe(&tabs, {
                let changes = changes.clone();
                move |_, _| changes.set(changes.get() + 1)
            })
        });
        cx.update(|_, cx| {
            tabs.update(cx, |tabs, cx| {
                tabs.set_active("two", cx);
                tabs.body_transition.elapse_for_test(Duration::from_millis(20));
            })
        });
        cx.run_until_parked();
        changes.set(0);
        let before = renders.get();
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        assert!(renders.get() > before);
        assert_eq!(changes.get(), 0, "animation must not notify Tabs observers");
    }

    #[test]
    fn custom_content_layout_preserves_intrinsic_height() {
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(|_, cx| BodyOnly {
            tabs: Tabs::new("intrinsic-effect")
                .content_layout(super::super::TabsContentLayout::Content)
                .content_transition(super::super::TabsContentTransition::new(|frame, _, _| {
                    div().children(frame.incoming.map(|panel| panel.element))
                }))
                .tab_with("one", "One", |_, _| div().h(px(80.0)).debug_selector(|| "intrinsic-panel".into()))
                .spawn(cx),
        });
        cx.run_until_parked();
        assert_eq!(cx.debug_bounds("intrinsic-panel").unwrap().size.height, px(80.0));
    }

    #[test]
    fn incoming_accepts_clicks_while_retiring_content_is_shielded() {
        let mut app = TestAppContext::single();
        let outgoing_clicks = std::rc::Rc::new(std::cell::Cell::new(0));
        let incoming_clicks = std::rc::Rc::new(std::cell::Cell::new(0));
        let (page, cx) = app.add_window_view(|_, cx| {
            let outgoing = outgoing_clicks.clone();
            let incoming = incoming_clicks.clone();
            BodyOnly {
                tabs: Tabs::new("interactive-effect")
                    .motion(|m| {
                        m.content =
                            crate::motion::Transition::timed(Duration::from_millis(200), crate::motion::Easing::Linear)
                    })
                    .content_layout(super::super::TabsContentLayout::Fill)
                    .content_transition(super::super::TabsContentTransition::new(|frame, _, _| {
                        div().size_full().relative().children(
                            frame
                                .outgoing
                                .into_iter()
                                .chain(frame.incoming)
                                .map(|panel| div().id(panel.id).absolute().inset_0().child(panel.element)),
                        )
                    }))
                    .tab_with("one", "One", move |_, _| {
                        let count = outgoing.clone();
                        div()
                            .id("old-content")
                            .size_full()
                            .debug_selector(|| "old-content".into())
                            .on_mouse_down(gpui::MouseButton::Left, move |_, _, _| count.set(count.get() + 1))
                    })
                    .tab_with("two", "Two", move |_, _| {
                        let count = incoming.clone();
                        div()
                            .id("new-content")
                            .w(px(100.0))
                            .h(px(100.0))
                            .debug_selector(|| "new-content".into())
                            .on_mouse_down(gpui::MouseButton::Left, move |_, _, _| count.set(count.get() + 1))
                    })
                    .spawn(cx),
            }
        });
        cx.simulate_resize(gpui::size(px(500.0), px(300.0)));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let tabs = page.read(cx).tabs.clone();
            tabs.update(cx, |tabs, cx| {
                tabs.set_active("two", cx);
                tabs.body_transition.elapse_for_test(Duration::from_millis(20));
            });
        });
        cx.run_until_parked();
        let incoming_center = cx.debug_bounds("new-content").unwrap().center();
        cx.simulate_click(incoming_center, Default::default());
        cx.run_until_parked();
        assert_eq!(incoming_clicks.get(), 1);
        cx.simulate_click(gpui::point(px(300.0), px(200.0)), Default::default());
        cx.run_until_parked();
        assert_eq!(outgoing_clicks.get(), 0);
    }

    #[test]
    fn custom_renderer_receives_lifecycle_bounds_and_releases_inactive_focus() {
        let samples = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut app = TestAppContext::single();
        let mut panel_focus = None;
        let (page, cx) = app.add_window_view(|_, cx| {
            let focus = cx.focus_handle();
            panel_focus = Some(focus.clone());
            let samples = samples.clone();
            let renderer = super::super::TabsContentTransition::new(move |frame, _, _| {
                samples.borrow_mut().push((
                    frame.progress,
                    frame.animating,
                    frame.outgoing.first().map(|panel| panel.id.clone()),
                    frame.viewport,
                ));
                // A completely different consumer effect: incoming-only opacity.
                div().size_full().opacity(frame.progress).children(frame.incoming.map(|panel| panel.element))
            });
            let renderer = renderer.measure_viewport(true);
            Page {
                tabs: Tabs::new("custom-fade")
                    .content_transition(renderer)
                    .content_layout(super::super::TabsContentLayout::Fill)
                    .tab_with("one", "One", move |_, _| div().id("focus-child").size_full().track_focus(&focus))
                    .tab_with("two", "Two", |_, _| div().size_full())
                    .motion(|m| {
                        m.content =
                            crate::motion::Transition::timed(Duration::from_millis(240), crate::motion::Easing::Linear)
                    })
                    .spawn(cx),
                split: true,
            }
        });
        cx.simulate_resize(gpui::size(px(500.0), px(300.0)));
        cx.run_until_parked();
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        assert!(
            samples
                .borrow()
                .iter()
                .any(|(_, _, _, bounds)| bounds.is_some_and(|bounds| bounds.size.width == px(500.0)))
        );
        let focus = panel_focus.unwrap();
        let tabs = cx.update(|window, cx| {
            focus.focus(window, cx);
            let tabs = page.read(cx).tabs.clone();
            tabs.update(cx, |tabs, cx| {
                tabs.set_active("two", cx);
                tabs.body_transition.elapse_for_test(Duration::from_millis(120));
            });
            tabs
        });
        cx.run_until_parked();
        assert!(samples.borrow().iter().any(|(progress, running, outgoing, _)| *running
            && *progress == 0.5
            && outgoing.as_deref() == Some("one")));
        cx.update(|window, cx| {
            use gpui::Focusable;
            assert!(!focus.is_focused(window));
            assert!(tabs.read(cx).group.read(cx).focus_handle(cx).is_focused(window));
        });
        cx.update(|_, cx| tabs.update(cx, |tabs, cx| tabs.set_motion_enabled(false, cx)));
        cx.run_until_parked();
        let samples = samples.borrow();
        let (progress, running, outgoing, _) = samples.last().unwrap();
        assert_eq!(*progress, 1.0);
        assert!(!running);
        assert!(outgoing.is_none());
    }

    #[test]
    fn custom_zero_duration_does_not_retain_outgoing() {
        let app = TestAppContext::single();
        app.update(|cx| {
            let tabs = Tabs::new("instant-custom")
                .content_transition(test_transition())
                .content_layout(super::super::TabsContentLayout::Fill)
                .tab_with("one", "One", |_, _| div())
                .tab_with("two", "Two", |_, _| div())
                .motion(|m| m.content = crate::motion::Transition::timed(Duration::ZERO, crate::motion::Easing::Linear))
                .spawn(cx);
            tabs.update(cx, |tabs, cx| {
                tabs.set_active("two", cx);
                assert_eq!(tabs.body_transition.progress(), 1.0);
                assert!(tabs.outgoing_content.is_empty());
            });
        });
    }

    // Consumer-defined renderer: no slide implementation exists in Tabs itself.
    fn test_transition() -> super::super::TabsContentTransition {
        super::super::TabsContentTransition::new(|frame, _, _| {
            let direction = if frame.direction == super::super::TabsTransitionDirection::Forward {
                1.0
            } else {
                -1.0
            };
            div()
                .size_full()
                .children(frame.outgoing.into_iter().map(|panel| {
                    div()
                        .id(panel.id)
                        .absolute()
                        .top_0()
                        .left(gpui::relative(-direction * frame.progress))
                        .size_full()
                        .child(panel.element)
                }))
                .children(frame.incoming.map(|panel| {
                    div()
                        .id(panel.id)
                        .absolute()
                        .top_0()
                        .left(gpui::relative(direction * (1.0 - frame.progress)))
                        .size_full()
                        .child(panel.element)
                }))
        })
    }

    #[test]
    fn horizontal_slide_positions_both_panels_and_releases_outgoing() {
        for split in [false, true] {
            let mut app = TestAppContext::single();
            let (page, cx) = app.add_window_view(|_, cx| Page {
                tabs: Tabs::new("slide-layout")
                    .content_transition(test_transition())
                    .content_layout(super::super::TabsContentLayout::Fill)
                    .tab_with("one", "One", |_, _| div().size_full().debug_selector(|| "slide-one".into()))
                    .tab_with("two", "Two", |_, _| div().size_full().debug_selector(|| "slide-two".into()))
                    .motion(|m| {
                        m.content = crate::motion::Transition::timed(
                            Duration::from_millis(240),
                            crate::motion::Easing::OutCubic,
                        )
                    })
                    .spawn(cx),
                split,
            });
            cx.simulate_resize(gpui::size(px(500.0), px(300.0)));
            cx.run_until_parked();
            let tabs = cx.update(|_, cx| page.read(cx).tabs.clone());
            cx.update(|_, cx| {
                tabs.update(cx, |tabs, cx| {
                    tabs.set_active("two", cx);
                    tabs.body_transition.elapse_for_test(Duration::from_millis(120));
                })
            });
            cx.run_until_parked();
            let outgoing = cx.debug_bounds("slide-one").expect("outgoing panel");
            let incoming = cx.debug_bounds("slide-two").expect("incoming panel");
            assert!(outgoing.origin.x < px(0.0));
            assert!(incoming.origin.x > px(0.0));
            assert_eq!(outgoing.size.width, px(500.0));
            assert!(incoming.size.height > px(0.0));
            cx.update(|_, cx| {
                tabs.update(cx, |tabs, cx| {
                    tabs.body_transition.elapse_for_test(Duration::from_millis(300));
                    cx.notify();
                })
            });
            cx.run_until_parked();
            assert!(cx.debug_bounds("slide-one").is_none());
            assert_eq!(cx.debug_bounds("slide-two").expect("settled panel").origin.x, px(0.0));
            cx.update(|_, cx| assert!(tabs.read(cx).outgoing_content.is_empty()));
        }
    }

    #[test]
    fn horizontal_slide_tracks_direction_retargets_and_settles() {
        let app = TestAppContext::single();
        let tabs = app.update(|cx| {
            Tabs::new("slide-tabs")
                .content_transition(test_transition())
                .content_layout(super::super::TabsContentLayout::Fill)
                .tab_with("one", "One", |_, _| div())
                .tab_with("two", "Two", |_, _| div())
                .tab_with("three", "Three", |_, _| div())
                .motion(|m| {
                    m.content =
                        crate::motion::Transition::timed(Duration::from_millis(240), crate::motion::Easing::OutCubic)
                })
                .spawn(cx)
        });
        app.update(|cx| {
            tabs.update(cx, |tabs, cx| {
                tabs.set_active("three", cx);
                assert_eq!(tabs.content_direction, super::super::TabsTransitionDirection::Forward);
                assert_eq!(tabs.outgoing_content.last().map(|(id, _)| id.as_ref()), Some("one"));
                tabs.body_transition.elapse_for_test(Duration::from_millis(120));
                tabs.body_transition.sync();
                assert!(tabs.body_transition.progress() > 0.0);
                tabs.set_active("two", cx);
                assert_eq!(tabs.outgoing_content.len(), 2, "both retiring panels survive interruption");
                assert!(tabs.transition_changes.last().is_some_and(|change| change.previous_progress > 0.0));
                assert_eq!(tabs.content_direction, super::super::TabsTransitionDirection::Backward);
                assert_eq!(tabs.outgoing_content.last().map(|(id, _)| id.as_ref()), Some("three"));
                tabs.set_active("two", cx);
                assert_eq!(tabs.outgoing_content.last().map(|(id, _)| id.as_ref()), Some("three"));
                tabs.set_active("missing", cx);
                assert_eq!(tabs.active_id().map(|id| id.as_ref()), Some("two"));
                tabs.set_motion_enabled(false, cx);
                assert!(tabs.outgoing_content.is_empty());
                assert_eq!(tabs.body_transition.progress(), 1.0);
                tabs.set_active("one", cx);
                assert!(tabs.outgoing_content.is_empty());
                tabs.set_motion_enabled(true, cx);
                tabs.set_active("three", cx);
                tabs.set_items([TabsItem::new("one")], cx);
                assert!(tabs.outgoing_content.is_empty());
                assert_eq!(tabs.body_transition.progress(), 1.0);
            })
        });
    }

    #[test]
    fn disabled_animation_and_zero_duration_select_without_fading() {
        for (motion_enabled, duration) in [(false, Duration::from_millis(300)), (true, Duration::ZERO)] {
            let app = TestAppContext::single();
            let tabs = app.update(|cx| {
                Tabs::new("instant-tabs")
                    .tab_with("one", "One", |_, _| div())
                    .tab_with("two", "Two", |_, _| div())
                    .motion_enabled(motion_enabled)
                    .motion(move |m| {
                        m.content = crate::motion::Transition::timed(duration, crate::motion::Easing::OutCubic)
                    })
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
    fn user_selection_and_rapid_changes_restart_only_the_incoming_body() {
        let app = TestAppContext::single();
        let tabs = app.update(|cx| {
            Tabs::new("motion-tabs")
                .tab_with("one", "One", |_, _| div())
                .tab_content(TabsItem::new("disabled").enabled(false), TabsContent::new(|_, _| div()))
                .tab_with("two", "Two", |_, _| div())
                .motion(move |m| {
                    m.content =
                        crate::motion::Transition::timed(Duration::from_millis(300), crate::motion::Easing::OutCubic)
                })
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
