//! Checkbox control exposition — live primary/secondary previews and event stream.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::checkbox::{Checkbox, CheckboxEvent};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::checkbox_theme_inspector::CheckboxThemeInspector;
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "CheckboxEvent::Change { checked }",
        trigger: "Pointer click or keyboard activate",
        notes: "Emitted when the user toggles the control. Assign local state from the payload.",
    },
    EventReferenceSpec {
        event: "CheckboxEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the control",
        notes: "Useful for form-level focus rings or screen-reader coordination.",
    },
    EventReferenceSpec {
        event: "CheckboxEvent::HoverChanged { hovered }",
        trigger: "Pointer enters or leaves the enabled control",
        notes: "Emitted only when hover state actually changes.",
    },
    EventReferenceSpec {
        event: "CheckboxEvent::EnabledChanged { enabled }",
        trigger: "Checkbox::set_enabled changes enabled state",
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
        symbol: "Checkbox",
        surface: "Type",
        notes: "Entity<CheckboxControl> — boolean toggle with checkbox chrome.",
    },
    PublicInterfaceSpec {
        symbol: "CheckboxEvent",
        surface: "Event",
        notes: "Non-exhaustive enum: Change, FocusChanged, HoverChanged, EnabledChanged.",
    },
    PublicInterfaceSpec {
        symbol: "Checkbox::new(id)",
        surface: "Factory",
        notes: "Starts a CheckboxBuilder with the default checkbox template.",
    },
    PublicInterfaceSpec {
        symbol: "CheckboxBuilder::with_data(checked)",
        surface: "Builder",
        notes: "Initial checked state before spawn.",
    },
    PublicInterfaceSpec {
        symbol: "CheckboxBuilder::content(...)",
        surface: "Builder",
        notes: "HasPresenter helper — label slot beside the indicator.",
    },
    PublicInterfaceSpec {
        symbol: "CheckboxBuilder::indicator_only",
        surface: "Builder",
        notes: "Renders only the indicator — use in tables and compact rows.",
    },
    PublicInterfaceSpec {
        symbol: "CheckboxBuilder::size / enabled / tab_stop",
        surface: "Builder",
        notes: "ControlSize and initial interaction state.",
    },
    PublicInterfaceSpec {
        symbol: "CheckboxBuilder::compact / without_elevation",
        surface: "Builder",
        notes: "Density and chrome toggles for embedded layouts.",
    },
    PublicInterfaceSpec {
        symbol: "CheckboxBuilder::template / spawn(cx)",
        surface: "Builder",
        notes: "Custom template hook and entity materialization.",
    },
    PublicInterfaceSpec {
        symbol: "Checkbox::set_data / data",
        surface: "Entity",
        notes: "Read or programmatically update checked state.",
    },
    PublicInterfaceSpec {
        symbol: "Checkbox::set_enabled / set_presenter / set_template",
        surface: "Entity",
        notes: "Lifecycle and presentation updates after spawn.",
    },
    PublicInterfaceSpec {
        symbol: "look.primary_checkbox(id) / secondary_checkbox(id)",
        surface: "Look",
        notes: "ShadcnLookControlExt style factories for emphasis tiers.",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CheckboxVariant {
    Primary,
    Secondary,
}

impl CheckboxVariant {
    fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
        }
    }
}

pub struct CheckboxControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_checkbox: Checkbox,
    primary_checkbox: Checkbox,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<CheckboxExpositionLeftPane>,
    theme_inspector: Entity<CheckboxThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct CheckboxExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_checkbox: Checkbox,
    primary_checkbox: Checkbox,
    event_stream: Entity<ControlEventStream>,
}

impl CheckboxExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for checkbox in [&self.secondary_checkbox, &self.primary_checkbox] {
            checkbox.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for CheckboxExpositionLeftPane {
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
                        .child(self.secondary_checkbox.clone())
                        .child(self.primary_checkbox.clone()),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-checkbox-left-pane")
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

impl CheckboxControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("checkbox").expect("checkbox catalog entry");
        let secondary_checkbox = look
            .secondary_checkbox("controls-doc-checkbox-secondary")
            .with_data(true)
            .content(|_, _| div().child("Secondary").into_any_element())
            .spawn(cx);
        let primary_checkbox = look
            .primary_checkbox("controls-doc-checkbox-primary")
            .with_data(false)
            .content(|_, _| div().child("Primary").into_any_element())
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-checkbox-event-log",
                "Toggle the preview checkboxes; all emitted CheckboxEvent variants appear in the stream below.",
            )
        });

        let left_pane = cx.new(|_| CheckboxExpositionLeftPane {
            look: look.clone(),
            entry,
            secondary_checkbox: secondary_checkbox.clone(),
            primary_checkbox: primary_checkbox.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-checkbox-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &super::checkbox_inspector_adapter::CHECKBOX_INSPECTOR_SPEC,
            super::checkbox_inspector_adapter::CheckboxInspectorAdapter::shared(),
        );

        let mut subscriptions = Vec::new();
        subscriptions.extend(subscribe_checkbox(
            &secondary_checkbox,
            CheckboxVariant::Secondary,
            "controls-doc-checkbox-secondary",
            event_stream.clone(),
            cx,
        ));
        subscriptions.extend(subscribe_checkbox(
            &primary_checkbox,
            CheckboxVariant::Primary,
            "controls-doc-checkbox-primary",
            event_stream.clone(),
            cx,
        ));

        Self {
            look,
            entry,
            secondary_checkbox,
            primary_checkbox,
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
        for checkbox in [&self.secondary_checkbox, &self.primary_checkbox] {
            checkbox.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for CheckboxControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let inspector_split = self.inspector_split.clone();
        with_look(&self.look, || {
            div()
                .id("controls-doc-checkbox-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(inspector_split)
        })
    }
}

fn subscribe_checkbox(
    checkbox: &Checkbox,
    variant: CheckboxVariant,
    checkbox_id: &'static str,
    event_stream: Entity<ControlEventStream>,
    cx: &mut Context<CheckboxControlExposition>,
) -> Vec<Subscription> {
    vec![cx.subscribe(checkbox, move |_, _, event: &CheckboxEvent, cx| {
        let line = format!("{} - {} (\"{checkbox_id}\")", format_checkbox_event(event), variant.label());
        event_stream.update(cx, |stream, cx| {
            stream.append_line(&line, cx);
            cx.notify();
        });
    })]
}

fn format_checkbox_event(event: &CheckboxEvent) -> String {
    match event {
        CheckboxEvent::Change { checked } => format!("CheckboxEvent::Change {{ checked: {checked} }}"),
        CheckboxEvent::FocusChanged { focused } => format!("CheckboxEvent::FocusChanged {{ focused: {focused} }}"),
        CheckboxEvent::EnabledChanged { enabled } => format!("CheckboxEvent::EnabledChanged {{ enabled: {enabled} }}"),
        CheckboxEvent::HoverChanged { hovered } => format!("CheckboxEvent::HoverChanged {{ hovered: {hovered} }}"),
        _ => "CheckboxEvent::(unknown)".to_string(),
    }
}
