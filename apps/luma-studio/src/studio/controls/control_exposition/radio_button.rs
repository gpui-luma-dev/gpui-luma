//! Radio button control exposition — live primary/secondary previews and event stream.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::radio_button::{RadioButton, RadioButtonEvent};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "RadioButtonEvent::Change { selected }",
        trigger: "Pointer click or keyboard activate",
        notes: "Toggles selection on each activation. In a mutually exclusive set, deselect siblings in the parent when selected becomes true.",
    },
    EventReferenceSpec {
        event: "RadioButtonEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the control",
        notes: "Useful for form-level focus rings or screen-reader coordination.",
    },
    EventReferenceSpec {
        event: "RadioButtonEvent::HoverChanged { hovered }",
        trigger: "Pointer enters or leaves the enabled control",
        notes: "Emitted only when hover state actually changes.",
    },
    EventReferenceSpec {
        event: "RadioButtonEvent::EnabledChanged { enabled }",
        trigger: "RadioButton::set_enabled changes enabled state",
        notes: "Programmatic transition; disabling may also emit FocusChanged and HoverChanged false.",
    },
    EventReferenceSpec {
        event: "(none)",
        trigger: "Disabled interaction",
        notes: "Pointer and keyboard activation are ignored while disabled.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "RadioButton",
        surface: "Type",
        notes: "Entity<RadioButtonControl> — boolean selection with radio chrome.",
    },
    PublicInterfaceSpec {
        symbol: "RadioButtonEvent",
        surface: "Event",
        notes: "Non-exhaustive enum: Change, FocusChanged, HoverChanged, EnabledChanged.",
    },
    PublicInterfaceSpec {
        symbol: "RadioButton::new(id)",
        surface: "Factory",
        notes: "Starts a RadioButtonBuilder with the default radio template.",
    },
    PublicInterfaceSpec {
        symbol: "RadioButtonBuilder::with_data(selected)",
        surface: "Builder",
        notes: "Initial selected state before spawn.",
    },
    PublicInterfaceSpec {
        symbol: "RadioButtonBuilder::content(...)",
        surface: "Builder",
        notes: "HasPresenter helper — label slot beside the indicator.",
    },
    PublicInterfaceSpec {
        symbol: "RadioButtonBuilder::size / role / enabled",
        surface: "Builder",
        notes: "ButtonSize, ButtonFamilyRole, and initial interaction state.",
    },
    PublicInterfaceSpec {
        symbol: "RadioButtonBuilder::compact / without_elevation",
        surface: "Builder",
        notes: "Density and chrome toggles for embedded layouts.",
    },
    PublicInterfaceSpec {
        symbol: "RadioButtonBuilder::template / spawn(cx)",
        surface: "Builder",
        notes: "Custom template hook and entity materialization.",
    },
    PublicInterfaceSpec {
        symbol: "RadioButton::set_data / data",
        surface: "Entity",
        notes: "Read or programmatically update selected state; click toggles selection.",
    },
    PublicInterfaceSpec {
        symbol: "RadioButton::set_enabled / set_presenter / set_template",
        surface: "Entity",
        notes: "Lifecycle and presentation updates after spawn.",
    },
    PublicInterfaceSpec {
        symbol: "look.primary_radio(id) / secondary_radio(id)",
        surface: "Look",
        notes: "ShadcnLookControlExt style factories for emphasis tiers.",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RadioButtonVariant {
    Primary,
    Secondary,
}

impl RadioButtonVariant {
    fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
        }
    }
}

pub struct RadioButtonControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_radio: RadioButton,
    primary_radio: RadioButton,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl RadioButtonControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("radio-button").expect("radio-button catalog entry");
        let secondary_radio = look
            .secondary_radio("controls-doc-radio-secondary")
            .with_data(false)
            .content(|_, _| div().child("Secondary").into_any_element())
            .spawn(cx);
        let primary_radio = look
            .primary_radio("controls-doc-radio-primary")
            .with_data(false)
            .content(|_, _| div().child("Primary").into_any_element())
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-radio-event-log",
                "Select the preview radio buttons; all emitted RadioButtonEvent variants appear in the stream below.",
            )
        });

        let mut subscriptions = Vec::new();
        subscriptions.extend(subscribe_radio(
            &secondary_radio,
            RadioButtonVariant::Secondary,
            "controls-doc-radio-secondary",
            event_stream.clone(),
            cx,
        ));
        subscriptions.extend(subscribe_radio(
            &primary_radio,
            RadioButtonVariant::Primary,
            "controls-doc-radio-primary",
            event_stream.clone(),
            cx,
        ));
        subscriptions.push(cx.subscribe(&secondary_radio, |this, _, event, cx| {
            if matches!(event, RadioButtonEvent::Change { selected: true }) {
                this.primary_radio.update(cx, |radio, cx| radio.set_data(false, cx));
            }
        }));
        subscriptions.push(cx.subscribe(&primary_radio, |this, _, event, cx| {
            if matches!(event, RadioButtonEvent::Change { selected: true }) {
                this.secondary_radio.update(cx, |radio, cx| radio.set_data(false, cx));
            }
        }));

        Self { look, entry, secondary_radio, primary_radio, event_stream, _subscriptions: subscriptions }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for radio in [&self.secondary_radio, &self.primary_radio] {
            radio.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for RadioButtonControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_start()
                        .gap(px(12.0))
                        .child(self.primary_radio.clone())
                        .child(self.secondary_radio.clone()),
                )
                .child(self.event_stream.clone());

            render_control_exposition_card(
                &self.look,
                self.entry,
                preview.into_any_element(),
                Some(render_exposition_doc_sections(&self.look, EVENT_SPECS, PUBLIC_INTERFACE_SPECS)),
                ControlExpositionLayout::BORDERLESS,
            )
        })
    }
}

fn subscribe_radio(
    radio: &RadioButton,
    variant: RadioButtonVariant,
    radio_id: &'static str,
    event_stream: Entity<ControlEventStream>,
    cx: &mut Context<RadioButtonControlExposition>,
) -> Vec<Subscription> {
    vec![cx.subscribe(radio, move |_, _, event: &RadioButtonEvent, cx| {
        let line = format!("{} - {} (\"{radio_id}\")", format_radio_event(event), variant.label());
        event_stream.update(cx, |stream, cx| {
            stream.append_line(&line, cx);
            cx.notify();
        });
    })]
}

fn format_radio_event(event: &RadioButtonEvent) -> String {
    match event {
        RadioButtonEvent::Change { selected } => format!("RadioButtonEvent::Change {{ selected: {selected} }}"),
        RadioButtonEvent::FocusChanged { focused } => {
            format!("RadioButtonEvent::FocusChanged {{ focused: {focused} }}")
        }
        RadioButtonEvent::EnabledChanged { enabled } => {
            format!("RadioButtonEvent::EnabledChanged {{ enabled: {enabled} }}")
        }
        RadioButtonEvent::HoverChanged { hovered } => {
            format!("RadioButtonEvent::HoverChanged {{ hovered: {hovered} }}")
        }
        _ => "RadioButtonEvent::(unknown)".to_string(),
    }
}
