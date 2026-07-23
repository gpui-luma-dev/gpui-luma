//! Reference control exposition template — copy this module when adding a new control doc.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_reference::render_event_reference_section;
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec};
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "ButtonEvent::Click",
        trigger: "Pointer click (primary button)",
        notes: "Emitted when the control is enabled. Subscribe in the parent with cx.subscribe.",
        sampled: true,
    },
    EventReferenceSpec {
        event: "ButtonEvent::Click",
        trigger: "Keyboard activate (Space, Enter)",
        notes: "Same variant when focused and ActivateControl runs.",
        sampled: true,
    },
    EventReferenceSpec {
        event: "ButtonEvent::HoverChanged { hovered }",
        trigger: "Pointer enters or leaves the enabled control.",
        notes: "Emitted only when hover state actually changes.",
        sampled: true,
    },
    EventReferenceSpec {
        event: "ButtonEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the control.",
        notes: "Emitted once per effective focus transition.",
        sampled: true,
    },
    EventReferenceSpec {
        event: "ButtonEvent::EnabledChanged { enabled }",
        trigger: "Button::set_enabled changes enabled state.",
        notes: "Programmatic state transition; repeated same-value setters are silent.",
        sampled: false,
    },
    EventReferenceSpec {
        event: "(none)",
        trigger: "Disabled interaction",
        notes: "Pointer and keyboard activation are ignored while disabled.",
        sampled: false,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
}

impl ButtonVariant {
    fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::Outline => "Outline",
            Self::Ghost => "Ghost",
        }
    }
}

pub struct ButtonControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    primary: Entity<Button>,
    secondary: Entity<Button>,
    outline: Entity<Button>,
    ghost: Entity<Button>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ButtonControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("button").expect("button catalog entry");
        let primary = look.primary_button("controls-doc-button-primary").label("Primary").spawn(cx);
        let secondary = look.secondary_button("controls-doc-button-secondary").label("Secondary").spawn(cx);
        let outline = look.outline_button("controls-doc-button-outline").label("Outline").spawn(cx);
        let ghost = look.ghost_button("controls-doc-button-ghost").label("Ghost").spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-button-event-log",
                "Subscribe with cx.subscribe and filter ButtonEvent::Click for activation-only behavior.",
            )
        });

        let mut subscriptions = Vec::new();
        subscriptions.extend(subscribe_button(
            &primary,
            ButtonVariant::Primary,
            "controls-doc-button-primary",
            event_stream.clone(),
            cx,
        ));
        subscriptions.extend(subscribe_button(
            &secondary,
            ButtonVariant::Secondary,
            "controls-doc-button-secondary",
            event_stream.clone(),
            cx,
        ));
        subscriptions.extend(subscribe_button(
            &outline,
            ButtonVariant::Outline,
            "controls-doc-button-outline",
            event_stream.clone(),
            cx,
        ));
        subscriptions.extend(subscribe_button(
            &ghost,
            ButtonVariant::Ghost,
            "controls-doc-button-ghost",
            event_stream.clone(),
            cx,
        ));

        Self { look, entry, primary, secondary, outline, ghost, event_stream, _subscriptions: subscriptions }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for button in [&self.primary, &self.secondary, &self.outline, &self.ghost] {
            button.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ButtonControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(
                    div().w_full().flex().justify_center().child(
                        div()
                            .flex()
                            .flex_wrap()
                            .justify_center()
                            .gap(px(12.0))
                            .child(self.primary.clone())
                            .child(self.secondary.clone())
                            .child(self.outline.clone())
                            .child(self.ghost.clone()),
                    ),
                )
                .child(self.event_stream.clone());

            render_control_exposition_card(
                &self.look,
                self.entry,
                preview.into_any_element(),
                Some(render_event_reference_section(&self.look, EVENT_SPECS)),
                ControlExpositionLayout::BORDERLESS,
            )
        })
    }
}

fn subscribe_button(
    button: &Entity<Button>,
    variant: ButtonVariant,
    button_id: &'static str,
    event_stream: Entity<ControlEventStream>,
    cx: &mut Context<ButtonControlExposition>,
) -> Vec<Subscription> {
    vec![cx.subscribe(button, move |_, _, event: &ButtonEvent, cx| {
        let line = match event {
            ButtonEvent::Click => format!("ButtonEvent::Click - {} (\"{button_id}\")", variant.label()),
            ButtonEvent::FocusChanged { focused } => {
                format!("ButtonEvent::FocusChanged {{ focused: {focused} }} - {} (\"{button_id}\")", variant.label())
            }
            ButtonEvent::EnabledChanged { enabled } => {
                format!("ButtonEvent::EnabledChanged {{ enabled: {enabled} }} - {} (\"{button_id}\")", variant.label())
            }
            ButtonEvent::HoverChanged { hovered } => {
                format!("ButtonEvent::HoverChanged {{ hovered: {hovered} }} - {} (\"{button_id}\")", variant.label())
            }
        };
        event_stream.update(cx, |stream, cx| {
            stream.append_line(&line, cx);
            cx.notify();
        });
    })]
}
