//! Accordion control exposition — single, multiple, and interactive content samples.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::accordion::{AccordionContent, AccordionControl, AccordionEvent, AccordionItem, AccordionTrigger};
use gpui_luma::controls::textfield::{TextField, TextFieldEvent};
use gpui_luma::vstack;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::accordion_inspector_adapter::{AccordionInspectorAdapter, ACCORDION_INSPECTOR_SPEC};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::shell_theme_inspectors::AccordionThemeInspector;
use super::template::render_control_exposition_card;

pub struct AccordionControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<AccordionExpositionLeftPane>,
    theme_inspector: Entity<AccordionThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct AccordionExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    single: Entity<AccordionControl>,
    multiple: Entity<AccordionControl>,
    interactive: Entity<AccordionControl>,
    interactive_field: TextField,
    event_stream: Entity<ControlEventStream>,
}

impl AccordionExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for entity in [&self.single, &self.multiple, &self.interactive] {
            entity.update(cx, |_, cx| cx.notify());
        }
        self.interactive_field.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for AccordionExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let muted = look.chrome().muted_text;

            let preview = div()
                .w_full()
                .max_w(px(420.0))
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(demo_section("Single expansion", muted, self.single.clone()))
                .child(demo_section("Multiple expansion", muted, self.multiple.clone()))
                .child(demo_section("Context-aware content", muted, self.interactive.clone()))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-accordion-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl AccordionControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("accordion").expect("accordion catalog entry");

        let interactive_field = look
            .textfield("controls-doc-accordion-interactive-field")
            .value("Edit me")
            .full_width(true)
            .spawn(cx);
        let field_for_content = interactive_field.clone();
        let interactive = look
            .accordion("controls-doc-accordion-interactive")
            .single()
            .item(
                AccordionItem::new(
                    "interactive",
                    AccordionTrigger::new("Interactive form"),
                    AccordionContent::custom(move |_window, cx| {
                        let value = field_for_content.read(cx).value();
                        vstack! {
                            gap=8;
                            field_for_content.clone(),
                            div().text_sm().child(format!("Live value: {value}")),
                        }
                        .into_any_element()
                    }),
                )
                .expanded(true),
            )
            .spawn(cx);

        let single = look
            .accordion("controls-doc-accordion-single")
            .single()
            .item(demo_item("general", "General", LucideIcon::Settings, "General settings content."))
            .item(demo_item("billing", "Billing", LucideIcon::CreditCard, "Billing and payment details."))
            .item(demo_item("team", "Team", LucideIcon::Users, "Team member management.").expanded(true))
            .item(demo_item("legacy", "Legacy", LucideIcon::Archive, "Legacy configuration.").enabled(false))
            .spawn(cx);
        let multiple = look
            .accordion("controls-doc-accordion-multiple")
            .multiple()
            .items([
                demo_item("faq-1", "What is GPUI-Luma?", LucideIcon::CircleQuestionMark, "A GPUI component library."),
                demo_item(
                    "faq-2",
                    "How do I theme controls?",
                    LucideIcon::Palette,
                    "Use ShadcnLook helpers and custom templates.",
                ),
                demo_item(
                    "faq-3",
                    "Can I disable items?",
                    LucideIcon::Ban,
                    "Yes — disabled triggers are skipped during keyboard navigation.",
                ),
            ])
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-accordion-event-log",
                "Expand or collapse sections; AccordionEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|_| AccordionExpositionLeftPane {
            look: look.clone(),
            entry,
            single: single.clone(),
            multiple: multiple.clone(),
            interactive: interactive.clone(),
            interactive_field: interactive_field.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-accordion-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &ACCORDION_INSPECTOR_SPEC,
            AccordionInspectorAdapter::shared(),
        );

        let mut subscriptions = Vec::new();
        for entity in [single.clone(), multiple.clone(), interactive.clone()] {
            let event_stream = event_stream.clone();
            subscriptions.push(cx.subscribe(&entity, move |_, _, event: &AccordionEvent, cx| {
                if let Some(line) = format_accordion_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }));
        }
        subscriptions.push(cx.subscribe(&interactive_field, {
            let interactive = interactive.clone();
            move |_, _, event: &TextFieldEvent, cx| {
                if matches!(event, TextFieldEvent::Change { .. }) {
                    interactive.update(cx, |_, cx| cx.notify());
                }
            }
        }));

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: subscriptions }
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
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for AccordionControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-accordion-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn format_accordion_event(event: &AccordionEvent) -> Option<String> {
    match event {
        AccordionEvent::ExpandedChanged { item_id, expanded } => {
            Some(format!("AccordionEvent::ExpandedChanged {{ item_id: \"{item_id}\", expanded: {expanded} }}"))
        }
        AccordionEvent::ItemFocused { item_id } => {
            Some(format!("AccordionEvent::ItemFocused {{ item_id: \"{item_id}\" }}"))
        }
        AccordionEvent::FocusChanged { focused } => {
            Some(format!("AccordionEvent::FocusChanged {{ focused: {focused} }}"))
        }
        AccordionEvent::ItemHoverChanged { item_id, hovered } => {
            Some(format!("AccordionEvent::ItemHoverChanged {{ item_id: \"{item_id}\", hovered: {hovered} }}"))
        }
        AccordionEvent::EnabledChanged { enabled } => {
            Some(format!("AccordionEvent::EnabledChanged {{ enabled: {enabled} }}"))
        }
        _ => None,
    }
}

fn demo_section(title: &'static str, title_color: gpui::Hsla, content: impl IntoElement) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(section_label(title, title_color))
        .child(div().w_full().child(content))
}

fn section_label(label: &'static str, color: gpui::Hsla) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(color)
        .child(label)
}

fn demo_item(id: &str, label: &str, icon: LucideIcon, body: &str) -> AccordionItem {
    let body = body.to_string();
    AccordionItem::new(
        id.to_string(),
        AccordionTrigger::new(label.to_string()).icon(icon),
        AccordionContent::custom(move |_, _| div().text_sm().child(body.clone()).into_any_element()),
    )
}
