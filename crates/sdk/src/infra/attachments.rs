//! Shared owner storage and entity modifiers for attached behavior.
use std::sync::Arc;
use gpui::{App, Bounds, Context, Div, Entity, AppContext, Pixels, SharedString, Stateful, Element, prelude::*};
use crate::{
    controls::tooltip::{Attachment, Tooltip, TooltipTheme, default_tooltip_theme},
    theme::InteractionState,
};

/// Controls expose shared attachment storage and may provide a moving anchor region.
/// No tooltip-specific methods or layout wrappers are required on a control.
pub trait AttachmentTarget: Sized + 'static {
    fn attachments(&self) -> &AttachmentHost;
    fn attachments_mut(&mut self) -> &mut AttachmentHost;
    fn attachment_anchor(&self, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        bounds
    }
}

/// Owner-held storage. Cloning an entity never clones its attachments.
#[derive(Default)]
pub struct AttachmentHost {
    tooltip: Option<Attachment>,
    accessible_role: Option<gpui::Role>,
    checked: Option<bool>,
    tooltip_theme: Option<Arc<dyn TooltipTheme>>,
}

impl AttachmentHost {
    pub(crate) fn with_accessible_role(role: gpui::Role) -> Self {
        Self { accessible_role: Some(role), ..Default::default() }
    }

    pub(crate) fn set_accessible_role(&mut self, role: gpui::Role) {
        self.accessible_role = Some(role);
    }

    pub(crate) fn set_checked(&mut self, checked: bool) {
        self.checked = Some(checked);
    }

    /// Bind the control's look. Explicit tooltip themes override this default.
    pub fn set_tooltip_theme(&mut self, theme: Arc<dyn TooltipTheme>) {
        self.tooltip_theme = Some(theme);
    }

    /// Apply attachments to a control's existing render root without changing layout.
    /// App-owned targets pass their current hover/focus/enabled state here.
    pub fn render<T: AttachmentTarget>(
        &self,
        root: Stateful<Div>,
        state: InteractionState,
        cx: &Context<T>,
    ) -> Stateful<Div> {
        let root = match (root.a11y_role(), self.accessible_role) {
            (None, Some(role)) => root.role(role),
            _ => root,
        };
        let root = match self.checked {
            Some(checked) => root.aria_toggled(if checked {
                gpui::Toggled::True
            } else {
                gpui::Toggled::False
            }),
            None => root,
        };
        match &self.tooltip {
            Some(tip) => {
                let root = match tip.description(cx) {
                    Some(description) => root.aria_description(description),
                    None => root,
                };
                tip.attach(root, state, cx.weak_entity())
            }
            None => root,
        }
    }

    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn description(&self, cx: &App) -> Option<SharedString> {
        self.tooltip.as_ref().and_then(|tip| tip.description(cx))
    }

    pub(crate) fn dismiss(&self, cx: &mut App) {
        if let Some(tip) = &self.tooltip {
            tip.dismiss(cx);
        }
    }

    fn clear(&mut self, cx: &mut App) {
        if let Some(tip) = self.tooltip.take() {
            tip.retire(cx);
        }
    }
}

/// Retained tooltip owner for a template-rendered item that has no separate control
/// entity, such as a tab. Create once, keep with the template, and apply to the
/// item's existing render root. Clones share presentation and dismissal history.
#[derive(Clone)]
pub struct TooltipHandle {
    owner: Entity<TemplateTooltipOwner>,
}

struct TemplateTooltipOwner {
    attachments: AttachmentHost,
}
impl AttachmentTarget for TemplateTooltipOwner {
    fn attachments(&self) -> &AttachmentHost {
        &self.attachments
    }
    fn attachments_mut(&mut self) -> &mut AttachmentHost {
        &mut self.attachments
    }
}
impl TooltipHandle {
    pub fn new(config: Tooltip, cx: &mut App) -> Self {
        Self {
            owner: cx.new(|_| TemplateTooltipOwner { attachments: AttachmentHost::default() }).tooltip(config, cx),
        }
    }

    /// Preserve the item's layout, focus and input handlers. The supplied state
    /// belongs to this item, rather than to the complete composite control.
    /// Give the root its accessible role/label for the help description association.
    pub fn render(&self, root: Stateful<Div>, state: InteractionState, cx: &App) -> Stateful<Div> {
        let host = self.owner.read(cx).attachments();
        match &host.tooltip {
            Some(tip) => {
                let root = match tip.description(cx) {
                    Some(description) => root.aria_description(description),
                    None => root,
                };
                tip.attach(root, state, self.owner.downgrade())
            }
            None => root,
        }
    }
}

/// Attach help to an existing SDK entity while preserving its type and identity.
pub trait TooltipEntityExt: Sized {
    /// Bind the default look for subsequent tooltip attachments. Existing attachments
    /// retain their theme; replace them to apply a newly bound theme.
    fn with_tooltip_theme(self, theme: Arc<dyn TooltipTheme>, cx: &mut App) -> Self;
    fn help(self, text: impl Into<SharedString>, cx: &mut App) -> Self;
    fn tooltip(self, tooltip: Tooltip, cx: &mut App) -> Self;
    /// Cancel pending/visible help. Other control behavior remains intact.
    fn clear_tooltip(self, cx: &mut App) -> Self;
}

impl<T: AttachmentTarget> TooltipEntityExt for Entity<T> {
    fn with_tooltip_theme(self, theme: Arc<dyn TooltipTheme>, cx: &mut App) -> Self {
        self.update(cx, |owner, _| owner.attachments_mut().set_tooltip_theme(theme));
        self
    }
    fn help(self, text: impl Into<SharedString>, cx: &mut App) -> Self {
        self.tooltip(Tooltip::new(text), cx)
    }

    fn tooltip(self, config: Tooltip, cx: &mut App) -> Self {
        let theme = self.read(cx).attachments().tooltip_theme.clone().unwrap_or_else(default_tooltip_theme);
        let attachment = Attachment::new(config, theme, &self, cx);
        self.update(cx, |owner, cx| {
            let host = owner.attachments_mut();
            host.clear(cx);
            host.tooltip = Some(attachment);
            cx.notify();
        });
        self
    }

    fn clear_tooltip(self, cx: &mut App) -> Self {
        self.update(cx, |owner, cx| {
            owner.attachments_mut().clear(cx);
            cx.notify();
        });
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
pub(crate) fn assert_control_semantics(host: &AttachmentHost, role: gpui::Role, checked: bool) {
    assert_eq!(host.accessible_role, Some(role));
    assert_eq!(host.checked, Some(checked));
}

// Button-backed controls forward attachments to their existing interactive owner.
// Invoke inside the control module so the inner button stays private.
macro_rules! delegate_tooltips {
    ($control:ty, $role:expr) => {
        impl crate::infra::attachments::TooltipEntityExt for gpui::Entity<$control> {
            fn with_tooltip_theme(
                self,
                theme: std::sync::Arc<dyn crate::controls::tooltip::TooltipTheme>,
                cx: &mut gpui::App,
            ) -> Self {
                self.read(cx).button.clone().with_tooltip_theme(theme, cx);
                self
            }
            fn help(self, text: impl Into<gpui::SharedString>, cx: &mut gpui::App) -> Self {
                self.tooltip(crate::controls::tooltip::Tooltip::new(text), cx)
            }
            fn tooltip(self, tooltip: crate::controls::tooltip::Tooltip, cx: &mut gpui::App) -> Self {
                self.read(cx).button.clone().tooltip(tooltip, cx);
                self
            }
            fn clear_tooltip(self, cx: &mut gpui::App) -> Self {
                self.read(cx).button.clone().clear_tooltip(cx);
                self
            }
        }
        #[cfg(all(test, feature = "test-support"))]
        mod tooltip_attachment_tests {
            use super::*;
            use crate::infra::attachments::{AttachmentTarget, TooltipEntityExt};
            #[test]
            fn attached_help_uses_inner_owner_and_keeps_input_behavior() {
                let mut app = gpui::TestAppContext::single();
                let (control, cx) = app.add_window_view(|window, cx| {
                    window.activate_window();
                    // Render the public control rather than just its internal Button.
                    <$control>::from_builder(new("tooltip-choice").animated(false), cx)
                });
                let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
                let log = events.clone();
                cx.update(|_, app| {
                    control.clone().tooltip(
                        crate::controls::tooltip::Tooltip::new("Choice help")
                            .delay(std::time::Duration::ZERO)
                            .on_event(move |event| log.lock().unwrap().push(event)),
                        app,
                    );
                });
                cx.run_until_parked();
                cx.update(|_, app| {
                    let button = control.read(app).button.clone();
                    let host = button.read(app).attachments();
                    assert_eq!(host.description(app).as_deref(), Some("Choice help"));
                    crate::infra::attachments::assert_control_semantics(host, $role, false);
                });
                cx.simulate_event(gpui::MouseMoveEvent {
                    position: gpui::point(gpui::px(10.0), gpui::px(10.0)),
                    ..Default::default()
                });
                cx.run_until_parked();
                assert_eq!(events.lock().unwrap().as_slice(), &[crate::controls::tooltip::TooltipEvent::Shown]);
                cx.simulate_click(gpui::point(gpui::px(10.0), gpui::px(10.0)), Default::default());
                cx.run_until_parked();
                assert_eq!(events.lock().unwrap().last(), Some(&crate::controls::tooltip::TooltipEvent::Hidden));
                cx.update(|_, app| {
                    assert!(*control.read(app).data());
                    control.clone().clear_tooltip(app);
                    let host = control.read(app).button.read(app).attachments();
                    assert!(host.description(app).is_none());
                    crate::infra::attachments::assert_control_semantics(host, $role, true);
                });
            }
        }
    };
}
pub(crate) use delegate_tooltips;

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{AppContext, TestAppContext};
    use crate::controls::{
        button::Button,
        slider::{SliderBuilder, SliderControl},
    };

    #[test]
    fn modifiers_keep_entity_identity_and_share_storage() {
        let app = TestAppContext::single();
        app.update(|cx| {
            let button = cx.new(|cx| Button::from_builder(Button::new("help"), cx));
            let id = button.entity_id();
            let clone = button.clone();
            let returned: Entity<Button> = button.help("First", cx);
            assert_eq!(returned.entity_id(), id);
            assert!(clone.read(cx).attachments().tooltip.is_some());
            clone.clone().tooltip(Tooltip::new("Replacement").show_once(), cx);
            assert!(returned.read(cx).attachments().tooltip.is_some());
            returned.clone().clear_tooltip(cx);
            assert!(clone.read(cx).attachments().tooltip.is_none());
            let slider =
                cx.new(|cx| SliderControl::from_builder(SliderBuilder::new("range"), cx)).help("Slider help", cx);
            assert!(slider.read(cx).attachments().tooltip.is_some());
        });
    }

    #[test]
    fn keyboard_dismissal_is_scoped_to_the_presenting_window() {
        use std::{sync::Mutex, time::Duration};
        use crate::controls::tooltip::{TooltipEvent, TooltipDismissal};
        let mut app = TestAppContext::single();
        let events = Arc::new(Mutex::new(Vec::new()));
        let log = events.clone();
        let (button, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Button::from_builder(Button::new("window-a"), cx)
        });
        let window_a = cx.update(|window, app| {
            button.clone().tooltip(
                Tooltip::new("A")
                    .delay(Duration::ZERO)
                    .dismissal(TooltipDismissal::Permanently)
                    .on_event(move |event| log.lock().unwrap().push(event)),
                app,
            );
            window.window_handle()
        });
        cx.simulate_event(gpui::MouseMoveEvent {
            position: gpui::point(gpui::px(10.0), gpui::px(10.0)),
            ..Default::default()
        });
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().as_slice(), &[TooltipEvent::Shown]);
        let (_, other) = app.add_window_view(|_, cx| Button::from_builder(Button::new("window-b"), cx));
        other.simulate_keystrokes("escape enter space");
        other.run_until_parked();
        assert_eq!(events.lock().unwrap().as_slice(), &[TooltipEvent::Shown]);
        let mut original = gpui::VisualTestContext::from_window(window_a, &app);
        original.simulate_keystrokes("escape");
        original.run_until_parked();
        assert_eq!(events.lock().unwrap().as_slice(), &[TooltipEvent::Shown, TooltipEvent::Hidden]);
    }

    #[test]
    fn removing_cached_target_cancels_visible_and_pending_help() {
        use gpui::{Render, Window, IntoElement, div, ParentElement};
        use std::{sync::Mutex, time::Duration};
        use crate::controls::tooltip::TooltipEvent;
        struct Page {
            button: Entity<Button>,
            show: bool,
        }
        impl Render for Page {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().when(self.show, |root| root.child(self.button.clone()))
            }
        }
        let mut app = TestAppContext::single();
        let events = Arc::new(Mutex::new(Vec::new()));
        let log = events.clone();
        let (page, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Page {
                button: Button::new("cached-target").spawn(cx).tooltip(
                    Tooltip::new("Cached")
                        .delay(Duration::from_millis(100))
                        .on_event(move |event| log.lock().unwrap().push(event)),
                    cx,
                ),
                show: true,
            }
        });
        let hover = |cx: &mut gpui::VisualTestContext| {
            cx.simulate_event(gpui::MouseMoveEvent {
                position: gpui::point(gpui::px(10.0), gpui::px(10.0)),
                ..Default::default()
            });
            cx.run_until_parked();
        };
        let show = |cx: &mut gpui::VisualTestContext, visible| {
            cx.update(|_, app| {
                page.update(app, |page, cx| {
                    page.show = visible;
                    cx.notify();
                })
            });
            cx.run_until_parked();
            // Flush entity releases queued by the completed frame.
            cx.update(|_, _| {});
            cx.run_until_parked();
        };
        hover(cx);
        show(cx, false);
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        assert!(events.lock().unwrap().is_empty());
        show(cx, true);
        hover(cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().as_slice(), &[TooltipEvent::Shown]);
        show(cx, false);
        assert_eq!(events.lock().unwrap().as_slice(), &[TooltipEvent::Shown, TooltipEvent::Hidden]);
        show(cx, true);
        hover(cx);
        assert_eq!(events.lock().unwrap().len(), 2);
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().last(), Some(&TooltipEvent::Shown));
        cx.deactivate_window();
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().last(), Some(&TooltipEvent::Hidden));
        cx.update(|window, _| window.activate_window());
        hover(cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().last(), Some(&TooltipEvent::Shown));
        cx.simulate_visibility_change(gpui::WindowVisibility::Hidden);
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().last(), Some(&TooltipEvent::Hidden));
        let count = events.lock().unwrap().len();
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().len(), count);
    }

    #[test]
    fn help_preserves_semantics_and_checked_state_after_clear() {
        use gpui::{Render, Window, IntoElement};
        struct Page {
            button: Entity<Button>,
            description: Option<&'static str>,
        }
        impl Render for Page {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                self.button.update(cx, |button, cx| {
                    let root = button.attachments().render(
                        gpui::div().id("semantic-target").role(gpui::Role::Switch).aria_label("Notifications"),
                        InteractionState::default(),
                        cx,
                    );
                    assert_eq!(root.a11y_role(), Some(gpui::Role::Switch));
                    let mut node = gpui::accesskit::Node::new(gpui::Role::Switch);
                    root.write_a11y_info(&mut node);
                    assert_eq!(node.label(), Some("Notifications"));
                    assert_eq!(node.description(), self.description);
                    assert_eq!(node.toggled(), Some(gpui::Toggled::True));
                    root
                })
            }
        }
        let mut app = TestAppContext::single();
        let (page, cx) = app.add_window_view(|_, cx| {
            let button = Button::new("semantic-target").spawn(cx).help("Additional help", cx);
            button.update(cx, |button, _| button.attachments_mut().set_checked(true));
            Page { button, description: Some("Additional help") }
        });
        cx.run_until_parked();
        cx.update(|_, app| {
            page.update(app, |page, cx| {
                page.button.clone().clear_tooltip(cx);
                page.description = None;
                cx.notify();
            });
        });
        cx.run_until_parked();
    }

    #[test]
    fn template_item_handle_keeps_history_across_renders_and_clones() {
        use gpui::{Render, Window, IntoElement, div, ParentElement};
        use crate::controls::tooltip::{TooltipEvent, bubble};
        use std::time::Duration;
        struct Page {
            tip: TooltipHandle,
        }
        impl Render for Page {
            fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let root =
                    div().id("template-item").w(gpui::px(100.0)).h(gpui::px(40.0)).role(gpui::Role::Tab).child("Tab");
                self.tip.clone().render(
                    root,
                    InteractionState {
                        hovered: window.mouse_position().x < gpui::px(100.0)
                            && window.mouse_position().y < gpui::px(40.0),
                        ..Default::default()
                    },
                    cx,
                )
            }
        }
        let mut app = TestAppContext::single();
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let log = events.clone();
        let (page, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Page {
                tip: TooltipHandle::new(
                    Tooltip::new("Tab help")
                        .show_once()
                        .delay(Duration::from_millis(50))
                        .on_event(move |event| log.lock().unwrap().push(event))
                        .template(|model| {
                            assert_eq!(model.text.as_ref(), "Tab help");
                            bubble(model)
                        }),
                    cx,
                ),
            }
        });
        cx.run_until_parked();
        cx.simulate_event(gpui::MouseMoveEvent {
            position: gpui::point(gpui::px(10.0), gpui::px(10.0)),
            ..Default::default()
        });
        cx.executor().advance_clock(Duration::from_millis(50));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().as_slice(), &[TooltipEvent::Shown]);
        cx.update(|_, app| page.update(app, |_, cx| cx.notify()));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().as_slice(), &[TooltipEvent::Shown]);
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
        cx.simulate_event(gpui::MouseMoveEvent {
            position: gpui::point(gpui::px(200.0), gpui::px(100.0)),
            ..Default::default()
        });
        cx.simulate_event(gpui::MouseMoveEvent {
            position: gpui::point(gpui::px(10.0), gpui::px(10.0)),
            ..Default::default()
        });
        cx.executor().advance_clock(Duration::from_millis(50));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().as_slice(), &[TooltipEvent::Shown, TooltipEvent::Hidden]);
    }

    #[test]
    fn textfield_help_preserves_editing_and_replaces_accessible_description() {
        use crate::controls::textfield::TextFieldBuilder;
        use gpui::{Render, Window, IntoElement, div, ParentElement};
        struct Page {
            field: crate::controls::textfield::TextField,
        }
        impl Render for Page {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().child(self.field.clone())
            }
        }
        let mut app = TestAppContext::single();
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let log = events.clone();
        let (page, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            crate::key_handling::bind_default_control_keys(cx);
            Page {
                field: TextFieldBuilder::new("help-field").spawn(cx).tooltip(
                    Tooltip::new("Enter a name")
                        .shortcut("Hint")
                        .delay(std::time::Duration::ZERO)
                        .on_event(move |event| log.lock().unwrap().push(event)),
                    cx,
                ),
            }
        });
        cx.run_until_parked();
        let field = cx.update(|_, app| page.read(app).field.clone());
        cx.update(|_, app| {
            assert_eq!(field.read(app).attachments().description(app).as_deref(), Some("Enter a name (Hint)"))
        });
        cx.simulate_event(gpui::MouseMoveEvent {
            position: gpui::point(gpui::px(10.0), gpui::px(10.0)),
            ..Default::default()
        });
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().as_slice(), &[crate::controls::tooltip::TooltipEvent::Shown]);
        cx.simulate_click(gpui::point(gpui::px(10.0), gpui::px(10.0)), Default::default());
        cx.simulate_input("Ada");
        cx.run_until_parked();
        cx.update(|_, app| {
            assert_eq!(field.read(app).value(), "Ada");
            field.clone().help("Replacement", app);
            assert_eq!(field.read(app).attachments().description(app).as_deref(), Some("Replacement"));
            field.clone().tooltip(Tooltip::new("Disabled").enabled(false), app);
            assert!(field.read(app).attachments().description(app).is_none());
            field.clone().clear_tooltip(app);
            assert!(field.read(app).attachments().tooltip.is_none());
        });
    }

    #[test]
    fn hover_render_dismissal_and_once_policy_dispatch_headlessly() {
        use std::{
            sync::atomic::{AtomicUsize, Ordering},
            time::Duration,
        };
        use gpui::{MouseMoveEvent, point, px};
        use crate::controls::tooltip::bubble;
        let mut app = TestAppContext::single();
        let rendered = Arc::new(AtomicUsize::new(0));
        let count = rendered.clone();
        let (button, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Button::from_builder(Button::new("hover-owner"), cx)
        });
        cx.update(|_, app| {
            button.clone().tooltip(
                Tooltip::new("Help").show_once().template(move |model| {
                    count.fetch_add(1, Ordering::Relaxed);
                    bubble(model)
                }),
                app,
            );
        });
        cx.run_until_parked();
        cx.simulate_event(MouseMoveEvent { position: point(px(10.0), px(10.0)), ..Default::default() });
        cx.run_until_parked();
        assert_eq!(rendered.load(Ordering::Relaxed), 0);
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        assert!(rendered.load(Ordering::Relaxed) > 0);
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
        let dismissed_count = rendered.load(Ordering::Relaxed);
        cx.simulate_event(MouseMoveEvent { position: point(px(400.0), px(400.0)), ..Default::default() });
        cx.simulate_event(MouseMoveEvent { position: point(px(10.0), px(10.0)), ..Default::default() });
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        assert_eq!(rendered.load(Ordering::Relaxed), dismissed_count);
    }

    #[test]
    fn configured_escape_expiry_and_replacement_dispatch_headlessly() {
        use std::{sync::Mutex, time::Duration};
        use gpui::{MouseMoveEvent, point, px};
        use crate::controls::tooltip::{TooltipDismissal, TooltipEvent};
        let mut app = TestAppContext::single();
        let events = Arc::new(Mutex::new(Vec::new()));
        let (button, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Button::from_builder(Button::new("configured-owner"), cx)
        });
        let attach = |cx: &mut gpui::VisualTestContext, policy, duration| {
            let log = events.clone();
            cx.update(|_, app| {
                button.clone().tooltip(
                    Tooltip::new("Help")
                        .delay(Duration::from_millis(100))
                        .hide_delay(Duration::from_millis(40))
                        .duration(duration)
                        .dismissal(policy)
                        .on_event(move |event| log.lock().unwrap().push(event)),
                    app,
                );
            });
            cx.run_until_parked();
        };
        let enter = |cx: &mut gpui::VisualTestContext| {
            cx.simulate_event(MouseMoveEvent { position: point(px(10.0), px(10.0)), ..Default::default() });
            cx.run_until_parked();
            cx.executor().advance_clock(Duration::from_millis(100));
            cx.run_until_parked();
        };
        let leave = |cx: &mut gpui::VisualTestContext| {
            cx.simulate_event(MouseMoveEvent { position: point(px(400.0), px(400.0)), ..Default::default() });
            cx.run_until_parked();
        };
        attach(cx, TooltipDismissal::UntilLeave, None);
        enter(cx);
        assert_eq!(*events.lock().unwrap(), [TooltipEvent::Shown]);
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
        assert_eq!(*events.lock().unwrap(), [TooltipEvent::Shown, TooltipEvent::Hidden]);
        enter(cx); // staying over the same target cannot reactivate it
        assert_eq!(events.lock().unwrap().len(), 2);
        leave(cx);
        enter(cx);
        assert_eq!(events.lock().unwrap().last(), Some(&TooltipEvent::Shown));
        leave(cx);
        cx.executor().advance_clock(Duration::from_millis(39));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().last(), Some(&TooltipEvent::Shown));
        cx.executor().advance_clock(Duration::from_millis(1));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().last(), Some(&TooltipEvent::Hidden));
        attach(cx, TooltipDismissal::Permanently, None);
        enter(cx);
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
        let count = events.lock().unwrap().len();
        leave(cx);
        enter(cx);
        assert_eq!(events.lock().unwrap().len(), count);
        attach(cx, TooltipDismissal::UntilLeave, Some(Duration::from_millis(80)));
        enter(cx);
        assert_eq!(events.lock().unwrap().last(), Some(&TooltipEvent::Shown));
        cx.executor().advance_clock(Duration::from_millis(80));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().last(), Some(&TooltipEvent::Hidden));
        let count = events.lock().unwrap().len();
        enter(cx);
        assert_eq!(events.lock().unwrap().len(), count);
        leave(cx);
        enter(cx);
        assert_eq!(events.lock().unwrap().last(), Some(&TooltipEvent::Shown));
        cx.update(|_, app| {
            button.clone().clear_tooltip(app);
        });
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().last(), Some(&TooltipEvent::Hidden));
    }

    #[test]
    fn replacement_clear_empty_disabled_click_and_scroll_cancel_help() {
        use std::{sync::Mutex, time::Duration};
        use gpui::{MouseDownEvent, MouseMoveEvent, ScrollWheelEvent, point, px};
        use crate::controls::tooltip::TooltipEvent;
        let mut app = TestAppContext::single();
        let events = Arc::new(Mutex::new(Vec::new()));
        let (button, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Button::from_builder(Button::new("cancellation-owner"), cx)
        });
        let attach = |cx: &mut gpui::VisualTestContext, text: &'static str, enabled: bool, delay| {
            let events = events.clone();
            cx.update(|_, app| {
                button.clone().tooltip(
                    Tooltip::new(text)
                        .enabled(enabled)
                        .delay(Duration::from_millis(delay))
                        .on_event(move |event| events.lock().unwrap().push((text, event))),
                    app,
                );
            });
            cx.run_until_parked();
        };
        let enter = |cx: &mut gpui::VisualTestContext| {
            cx.simulate_event(MouseMoveEvent { position: point(px(10.0), px(10.0)), ..Default::default() });
            cx.run_until_parked();
        };
        attach(cx, "old", true, 500);
        enter(cx);
        attach(cx, "replacement", true, 100);
        enter(cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        assert_eq!(*events.lock().unwrap(), [("replacement", TooltipEvent::Shown)]);
        cx.simulate_event(ScrollWheelEvent { position: point(px(10.0), px(10.0)), ..Default::default() });
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().last(), Some(&("replacement", TooltipEvent::Hidden)));
        let count = events.lock().unwrap().len();
        enter(cx);
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().len(), count);
        attach(cx, "click", true, 100);
        enter(cx);
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().last(), Some(&("click", TooltipEvent::Shown)));
        cx.simulate_event(MouseDownEvent { position: point(px(10.0), px(10.0)), ..Default::default() });
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().last(), Some(&("click", TooltipEvent::Hidden)));
        attach(cx, "pending", true, 500);
        enter(cx);
        cx.update(|_, app| {
            button.clone().clear_tooltip(app);
        });
        cx.run_until_parked();
        let count = events.lock().unwrap().len();
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        assert_eq!(events.lock().unwrap().len(), count);
        for (text, enabled) in [("", true), ("disabled", false)] {
            attach(cx, text, enabled, 100);
            enter(cx);
            cx.executor().advance_clock(Duration::from_millis(100));
            cx.run_until_parked();
            assert_eq!(events.lock().unwrap().len(), count);
        }
    }

    #[test]
    fn keyboard_focus_opens_without_delay_and_blur_hides() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use gpui::{Focusable, MouseMoveEvent, point, px};
        use crate::controls::tooltip::bubble;
        let mut app = TestAppContext::single();
        let rendered = Arc::new(AtomicUsize::new(0));
        let count = rendered.clone();
        let (button, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Button::from_builder(Button::new("focus-owner"), cx)
        });
        cx.update(|_, app| {
            button.clone().tooltip(
                Tooltip::new("Help").template(move |model| {
                    count.fetch_add(1, Ordering::Relaxed);
                    bubble(model)
                }),
                app,
            );
        });
        cx.run_until_parked();
        cx.simulate_event(MouseMoveEvent { position: point(px(400.0), px(400.0)), ..Default::default() });
        cx.simulate_keystrokes("right");
        cx.update(|window, app| button.read(app).focus_handle(app).focus(window, app));
        cx.run_until_parked();
        assert!(rendered.load(Ordering::Relaxed) > 0);
        cx.update(|window, app| window.blur(app));
        cx.run_until_parked();
        cx.executor().advance_clock(std::time::Duration::from_millis(180));
        cx.run_until_parked();
        let blurred_count = rendered.load(Ordering::Relaxed);
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        assert_eq!(rendered.load(Ordering::Relaxed), blurred_count);
    }
}
