use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::accordion::{AccordionContent, AccordionControl, AccordionEvent, AccordionItem, AccordionTrigger};
use gpui_luma::controls::textfield::{TextField, TextFieldEvent};
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnTextSize};
use gpui_luma::vstack;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_accordion_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector_description, notify_entity, InspectorToggleRegistry};

const ACCORDION_DESCRIPTION: &str = concat!(
    "Accordion groups collapsible sections with single or multiple expansion modes. ",
    "Use arrow keys to move between triggers and activate to expand or collapse."
);

#[derive(Clone)]
pub(in crate::gallery) struct AccordionPane {
    single: Entity<AccordionControl>,
    multiple: Entity<AccordionControl>,
    interactive: Entity<AccordionControl>,
    interactive_field: TextField,
    last_event: SharedString,
    inspector: Entity<ColorInspectorShell>,
}

impl AccordionPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree =
            spawn_color_inspector_tree("accordion-inspector-tree", look.clone(), build_accordion_inspect_tree, cx);
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "accordion-inspector",
                "accordion-inspector-split",
                "accordion-inspector-detail",
                build_accordion_inspect_tree,
                cx,
            )
        });

        let interactive_field =
            look.textfield("accordion-interactive-field").value("Edit me").full_width(true).spawn(cx);
        let field_for_content = interactive_field.clone();
        let interactive = look
            .accordion("accordion-interactive")
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

        Self {
            single: look
                .accordion("accordion-single")
                .single()
                .item(demo_item("general", "General", LucideIcon::Settings, "General settings content."))
                .item(demo_item("billing", "Billing", LucideIcon::CreditCard, "Billing and payment details."))
                .item(demo_item("team", "Team", LucideIcon::Users, "Team member management.").expanded(true))
                .item(demo_item("legacy", "Legacy", LucideIcon::Archive, "Legacy configuration.").enabled(false))
                .spawn(cx),
            multiple: look
                .accordion("accordion-multiple")
                .multiple()
                .items([
                    demo_item(
                        "faq-1",
                        "What is GPUI-Luma?",
                        LucideIcon::CircleQuestionMark,
                        "A GPUI component library.",
                    ),
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
                .spawn(cx),
            interactive,
            interactive_field,
            last_event: "None".into(),
            inspector,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.single, |app, _, event: &AccordionEvent, cx| {
            app.panes.accordion.handle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.multiple, |app, _, event: &AccordionEvent, cx| {
            app.panes.accordion.handle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.interactive_field, |app, _, event: &TextFieldEvent, cx| {
            if matches!(event, TextFieldEvent::Change { .. }) {
                notify_entity(&app.panes.accordion.interactive, cx);
            }
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        let chrome = look.chrome();

        gallery_pane_with_inspector_description(
            "accordion",
            "Accordion",
            Some(ACCORDION_DESCRIPTION),
            div()
                .w(px(420.0))
                .flex()
                .flex_col()
                .gap_6()
                .child(example_block("Single expansion", self.single.clone(), chrome.muted_text))
                .child(example_block("Multiple expansion", self.multiple.clone(), chrome.muted_text))
                .child(example_block("Context-aware content", self.interactive.clone(), chrome.muted_text))
                .child(
                    div()
                        .typography_style(look.typography_scale(ShadcnTextSize::Sm))
                        .text_color(chrome.muted_text)
                        .child(format!("Last event: {}", self.last_event)),
                )
                .into_any_element(),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.single, cx);
        notify_entity(&self.multiple, cx);
        notify_entity(&self.interactive, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn handle_event(&mut self, event: &AccordionEvent, cx: &mut Context<GalleryApp>) {
        self.last_event = match event {
            AccordionEvent::ExpandedChanged { item_id, expanded } => {
                format!("{} {}", item_id, if *expanded { "expanded" } else { "collapsed" }).into()
            }
        };
        cx.notify();
    }
}

fn example_block(title: &str, accordion: Entity<AccordionControl>, label_color: gpui::Hsla) -> impl IntoElement {
    let title = title.to_string();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_sm().text_color(label_color).child(title))
        .child(accordion)
}

fn demo_item(id: &str, label: &str, icon: LucideIcon, body: &str) -> AccordionItem {
    let body = body.to_string();
    AccordionItem::new(
        id.to_string(),
        AccordionTrigger::new(label.to_string()).icon(icon),
        AccordionContent::custom(move |_, _| div().text_sm().child(body.clone()).into_any_element()),
    )
}
