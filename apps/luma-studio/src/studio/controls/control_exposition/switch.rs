//! Switch control exposition — live primary/secondary previews and event stream.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::switch::{Switch, SwitchEvent};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::switch_theme_inspector::SwitchThemeInspector;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "SwitchEvent::Change { on }",
        trigger: "Pointer click or keyboard activate",
        notes: "Emitted when the user toggles the control. Assign local state from the payload.",
    },
    EventReferenceSpec {
        event: "SwitchEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the control",
        notes: "Useful for form-level focus rings or screen-reader coordination.",
    },
    EventReferenceSpec {
        event: "SwitchEvent::HoverChanged { hovered }",
        trigger: "Pointer enters or leaves the enabled control",
        notes: "Emitted only when hover state actually changes.",
    },
    EventReferenceSpec {
        event: "SwitchEvent::EnabledChanged { enabled }",
        trigger: "Switch::set_enabled changes enabled state",
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
        symbol: "Switch",
        surface: "Type",
        notes: "Entity<SwitchControl> — boolean toggle with switch track and thumb.",
    },
    PublicInterfaceSpec {
        symbol: "SwitchEvent",
        surface: "Event",
        notes: "Non-exhaustive enum: Change, FocusChanged, HoverChanged, EnabledChanged.",
    },
    PublicInterfaceSpec {
        symbol: "Switch::new(id)",
        surface: "Factory",
        notes: "Starts a SwitchBuilder with the default switch template.",
    },
    PublicInterfaceSpec {
        symbol: "SwitchBuilder::with_data(on)",
        surface: "Builder",
        notes: "Initial on/off state before spawn.",
    },
    PublicInterfaceSpec {
        symbol: "SwitchBuilder::content(...)",
        surface: "Builder",
        notes: "HasPresenter helper — label slot beside the switch.",
    },
    PublicInterfaceSpec {
        symbol: "SwitchBuilder::orientation / track_content / thumb_content",
        surface: "Builder",
        notes: "Horizontal or vertical layout; optional track and thumb interior slots.",
    },
    PublicInterfaceSpec {
        symbol: "SwitchBuilder::track_length_extra",
        surface: "Builder",
        notes: "Extra length along the track axis for interior labels.",
    },
    PublicInterfaceSpec {
        symbol: "SwitchBuilder::size / enabled / compact",
        surface: "Builder",
        notes: "ButtonSize, interaction state, and density toggles.",
    },
    PublicInterfaceSpec {
        symbol: "SwitchBuilder::template / spawn(cx)",
        surface: "Builder",
        notes: "Custom template hook and entity materialization.",
    },
    PublicInterfaceSpec {
        symbol: "Switch::set_data / data",
        surface: "Entity",
        notes: "Read or programmatically update on/off state.",
    },
    PublicInterfaceSpec {
        symbol: "Switch::set_switch_orientation / set_switch_track_content",
        surface: "Entity",
        notes: "Hot-swap orientation and track/thumb slot content after spawn.",
    },
    PublicInterfaceSpec {
        symbol: "look.primary_switch(id) / secondary_switch(id)",
        surface: "Look",
        notes: "ShadcnLookControlExt style factories for emphasis tiers.",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SwitchVariant {
    Primary,
    Secondary,
}

impl SwitchVariant {
    fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
        }
    }
}

pub struct SwitchControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_switch: Switch,
    primary_switch: Switch,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<SwitchExpositionLeftPane>,
    theme_inspector: Entity<SwitchThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct SwitchExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_switch: Switch,
    primary_switch: Switch,
    event_stream: Entity<ControlEventStream>,
}

impl SwitchExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for switch in [&self.secondary_switch, &self.primary_switch] {
            switch.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for SwitchExpositionLeftPane {
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
                        .child(self.primary_switch.clone())
                        .child(self.secondary_switch.clone()),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-switch-left-pane")
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

impl SwitchControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("switch").expect("switch catalog entry");
        let secondary_switch = look
            .secondary_switch("controls-doc-switch-secondary")
            .with_data(true)
            .content(|_, _| div().into_any_element())
            .spawn(cx);
        let primary_switch = look
            .primary_switch("controls-doc-switch-primary")
            .with_data(false)
            .content(|_, _| div().into_any_element())
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-switch-event-log",
                "Toggle the preview switches; all emitted SwitchEvent variants appear in the stream below.",
            )
        });

        let left_pane = cx.new(|_| SwitchExpositionLeftPane {
            look: look.clone(),
            entry,
            secondary_switch: secondary_switch.clone(),
            primary_switch: primary_switch.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-switch-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &super::switch_inspector_adapter::SWITCH_INSPECTOR_SPEC,
            super::switch_inspector_adapter::SwitchInspectorAdapter::shared(),
        );

        let mut subscriptions = Vec::new();
        subscriptions.extend(subscribe_switch(
            &secondary_switch,
            SwitchVariant::Secondary,
            "controls-doc-switch-secondary",
            event_stream.clone(),
            cx,
        ));
        subscriptions.extend(subscribe_switch(
            &primary_switch,
            SwitchVariant::Primary,
            "controls-doc-switch-primary",
            event_stream.clone(),
            cx,
        ));

        Self {
            look,
            entry,
            secondary_switch,
            primary_switch,
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
        for switch in [&self.secondary_switch, &self.primary_switch] {
            switch.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for SwitchControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let inspector_split = self.inspector_split.clone();
        with_look(&self.look, || {
            div()
                .id("controls-doc-switch-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(inspector_split)
        })
    }
}

fn subscribe_switch(
    switch: &Switch,
    variant: SwitchVariant,
    switch_id: &'static str,
    event_stream: Entity<ControlEventStream>,
    cx: &mut Context<SwitchControlExposition>,
) -> Vec<Subscription> {
    vec![cx.subscribe(switch, move |_, _, event: &SwitchEvent, cx| {
        let line = format!("{} - {} (\"{switch_id}\")", format_switch_event(event), variant.label());
        event_stream.update(cx, |stream, cx| {
            stream.append_line(&line, cx);
            cx.notify();
        });
    })]
}

fn format_switch_event(event: &SwitchEvent) -> String {
    match event {
        SwitchEvent::Change { on } => format!("SwitchEvent::Change {{ on: {on} }}"),
        SwitchEvent::FocusChanged { focused } => format!("SwitchEvent::FocusChanged {{ focused: {focused} }}"),
        SwitchEvent::EnabledChanged { enabled } => format!("SwitchEvent::EnabledChanged {{ enabled: {enabled} }}"),
        SwitchEvent::HoverChanged { hovered } => format!("SwitchEvent::HoverChanged {{ hovered: {hovered} }}"),
        _ => "SwitchEvent::(unknown)".to_string(),
    }
}
