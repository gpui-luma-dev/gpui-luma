//! Radio button control exposition — live primary/secondary previews and event stream.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::radio_button::{RadioButton, RadioButtonEvent};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::radio_button_theme_inspector::RadioButtonThemeInspector;
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
        symbol: "look.primary_radio(id) / secondary_radio(id) / content_only_radio(id)",
        surface: "Look",
        notes: "ShadcnLookControlExt style factories for emphasis tiers and indicator-only chrome.",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RadioButtonVariant {
    Primary,
    Secondary,
    ContentOnly,
}

impl RadioButtonVariant {
    fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::ContentOnly => "Content Only",
        }
    }
}

pub struct RadioButtonControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_radio: RadioButton,
    primary_radio: RadioButton,
    content_only_radio: RadioButton,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<RadioButtonExpositionLeftPane>,
    theme_inspector: Entity<RadioButtonThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct RadioButtonExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_radio: RadioButton,
    primary_radio: RadioButton,
    content_only_radio: RadioButton,
    event_stream: Entity<ControlEventStream>,
}

impl RadioButtonExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for radio in [&self.secondary_radio, &self.primary_radio, &self.content_only_radio] {
            radio.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for RadioButtonExpositionLeftPane {
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
                        .child(self.secondary_radio.clone())
                        .child(self.content_only_radio.clone()),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-radio-button-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    &self.look,
                    self.entry,
                    preview.into_any_element(),
                    Some(render_exposition_doc_sections(&self.look, EVENT_SPECS, PUBLIC_INTERFACE_SPECS)),
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
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
        let content_only_radio = look.content_only_radio("controls-doc-radio-content-only").with_data(false).spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-radio-event-log",
                "Select the preview radio buttons; all emitted RadioButtonEvent variants appear in the stream below.",
            )
        });

        let left_pane = cx.new(|_| RadioButtonExpositionLeftPane {
            look: look.clone(),
            entry,
            secondary_radio: secondary_radio.clone(),
            primary_radio: primary_radio.clone(),
            content_only_radio: content_only_radio.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-radio-button-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &super::radio_button_inspector_adapter::RADIO_BUTTON_INSPECTOR_SPEC,
            super::radio_button_inspector_adapter::RadioButtonInspectorAdapter::shared(),
        );

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
        subscriptions.extend(subscribe_radio(
            &content_only_radio,
            RadioButtonVariant::ContentOnly,
            "controls-doc-radio-content-only",
            event_stream.clone(),
            cx,
        ));
        subscriptions.push(cx.subscribe(&secondary_radio, |this, _, event, cx| {
            if matches!(event, RadioButtonEvent::Change { selected: true }) {
                this.primary_radio.update(cx, |radio, cx| radio.set_data(false, cx));
                this.content_only_radio.update(cx, |radio, cx| radio.set_data(false, cx));
            }
        }));
        subscriptions.push(cx.subscribe(&primary_radio, |this, _, event, cx| {
            if matches!(event, RadioButtonEvent::Change { selected: true }) {
                this.secondary_radio.update(cx, |radio, cx| radio.set_data(false, cx));
                this.content_only_radio.update(cx, |radio, cx| radio.set_data(false, cx));
            }
        }));
        subscriptions.push(cx.subscribe(&content_only_radio, |this, _, event, cx| {
            if matches!(event, RadioButtonEvent::Change { selected: true }) {
                this.secondary_radio.update(cx, |radio, cx| radio.set_data(false, cx));
                this.primary_radio.update(cx, |radio, cx| radio.set_data(false, cx));
            }
        }));

        Self {
            look,
            entry,
            secondary_radio,
            primary_radio,
            content_only_radio,
            event_stream,
            left_pane,
            theme_inspector,
            inspector_split,
            _subscriptions: subscriptions,
        }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for radio in [&self.secondary_radio, &self.primary_radio, &self.content_only_radio] {
            radio.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for RadioButtonControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let inspector_split = self.inspector_split.clone();
        with_look(&self.look, || {
            div()
                .id("controls-doc-radio-button-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(inspector_split)
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
