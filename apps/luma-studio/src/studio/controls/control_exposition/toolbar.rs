//! Toolbar control exposition — gallery-aligned editing toolbar with focus strategy toggles.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::control_group::ControlGroupFocusStrategy;
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::selector::SelectorItem;
use gpui_luma::controls::toolbar::{ToolbarControl, ToolbarEvent, ToolbarValue};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "ToolbarEvent::Click { id }",
        trigger: "Pointer activate on a toolbar button item",
        notes: "Emitted for command-style items without a bound value.",
    },
    EventReferenceSpec {
        event: "ToolbarEvent::Change { id, value }",
        trigger: "Toggle, selector, or textfield item changes value",
        notes: "ToolbarValue carries bool or string payloads from hosted items.",
    },
    EventReferenceSpec { event: "(none)", trigger: "Disabled interaction", notes: "Ignored while disabled." },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "ToolbarControl",
        surface: "Type",
        notes: "Entity<Toolbar> — horizontal control_group specialization with hosted items.",
    },
    PublicInterfaceSpec {
        symbol: "ToolbarEvent",
        surface: "Event",
        notes: "Click, Change, FocusChanged, and item lifecycle fan-in from child controls.",
    },
    PublicInterfaceSpec { symbol: "look.toolbar(id)", surface: "Look", notes: "ShadcnLookControlExt factory." },
    PublicInterfaceSpec {
        symbol: "ShadcnToolbarItemExt",
        surface: "Look",
        notes: "toolbar_button, toolbar_toggle, toolbar_menu, toolbar_selector, toolbar_textfield factories.",
    },
    PublicInterfaceSpec {
        symbol: "ToolbarBuilder::roving_item_focus / set_focus_strategy",
        surface: "Builder",
        notes: "RovingItemFocus (default) or ActiveDescendant keyboard traversal.",
    },
    PublicInterfaceSpec {
        symbol: "ToolbarBuilder::item / separator / spawn(cx)",
        surface: "Builder",
        notes: "Compose hosted controls and materialize the toolbar entity.",
    },
];

pub struct ToolbarControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    toolbar: ToolbarControl,
    roving_focus_button: Entity<Button<()>>,
    sequential_focus_button: Entity<Button<()>>,
    event_stream: Entity<ControlEventStream>,
    status: String,
    focus_mode: &'static str,
    _subscriptions: Vec<Subscription>,
}

impl ToolbarControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("toolbar").expect("toolbar catalog entry");

        let roving_focus_button =
            look.ghost_button("controls-doc-toolbar-focus-roving").label("Roving focus").spawn(cx);
        let sequential_focus_button =
            look.ghost_button("controls-doc-toolbar-focus-sequential").label("Sequential focus").spawn(cx);

        let toolbar = look
            .toolbar("controls-doc-toolbar")
            .roving_item_focus()
            .item(look.toolbar_menu(
                "assistant",
                "Assistant",
                LucideIcon::WandSparkles,
                [
                    MenuItem::new("rewrite").label("Rewrite"),
                    MenuItem::new("summarize").label("Summarize"),
                    MenuItem::new("continue").label("Continue writing"),
                ],
                cx,
            ))
            .separator("assistant-separator")
            .item(look.toolbar_selector(
                "style",
                "Paragraph",
                [
                    SelectorItem::new("paragraph").label("Paragraph").icon(LucideIcon::Pilcrow),
                    SelectorItem::new("heading").label("Heading").icon(LucideIcon::Type),
                    SelectorItem::new("quote").label("Quote").icon(LucideIcon::BookType),
                ],
                "paragraph",
                cx,
            ))
            .separator("style-separator")
            .item(look.toolbar_toggle("bold", LucideIcon::Bold, cx))
            .item(look.toolbar_toggle("italic", LucideIcon::Italic, cx))
            .item(look.toolbar_toggle("underline", LucideIcon::Underline, cx))
            .item(look.toolbar_label_menu(
                "color",
                "A",
                [
                    MenuItem::new("default").label("Default color"),
                    MenuItem::new("muted").label("Muted"),
                    MenuItem::new("accent").label("Accent"),
                ],
                cx,
            ))
            .separator("format-separator")
            .item(look.toolbar_menu(
                "alignment",
                "Alignment",
                LucideIcon::TextAlignStart,
                [
                    MenuItem::new("left").label("Align left").icon(LucideIcon::TextAlignStart),
                    MenuItem::new("center").label("Align center").icon(LucideIcon::TextAlignCenter),
                    MenuItem::new("right").label("Align right").icon(LucideIcon::TextAlignEnd),
                    MenuItem::new("justify").label("Justify").icon(LucideIcon::TextAlignJustify),
                ],
                cx,
            ))
            .separator("insert-separator")
            .item(look.toolbar_button("link", LucideIcon::Link, cx))
            .item(look.toolbar_button("image", LucideIcon::Image, cx))
            .item(look.toolbar_button("video", LucideIcon::Video, cx))
            .item(look.toolbar_menu(
                "more",
                "More",
                LucideIcon::Ellipsis,
                [
                    MenuItem::new("clear").label("Clear formatting"),
                    MenuItem::new("divider").label("Insert divider"),
                    MenuItem::new("comment").label("Add comment"),
                ],
                cx,
            ))
            .separator("code-separator")
            .item(look.toolbar_textfield("search").placeholder("Search...").spawn(cx).on_change(|_value, _cx| {}))
            .item(look.toolbar_button("code", LucideIcon::Code, cx))
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-toolbar-event-log",
                "Interact with toolbar items; ToolbarEvent variants appear below.",
            )
        });

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&toolbar, {
            let event_stream = event_stream.clone();
            move |this, _, event: &ToolbarEvent, cx| {
                this.status = match event {
                    ToolbarEvent::Click { id } => format!("{id} clicked"),
                    ToolbarEvent::Change { id, value } => match value {
                        ToolbarValue::Bool(on) => format!("{id}: {}", if *on { "on" } else { "off" }),
                        ToolbarValue::String(value) => format!("{id}: {value}"),
                    },
                    _ => this.status.clone(),
                };
                if let Some(line) = format_toolbar_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
                cx.notify();
            }
        }));
        subscriptions.push(cx.subscribe(&roving_focus_button, {
            move |this, _, event: &ButtonEvent, cx| {
                if matches!(event, ButtonEvent::Click) {
                    this.toolbar.update(cx, |toolbar, cx| {
                        toolbar.set_focus_strategy(ControlGroupFocusStrategy::RovingItemFocus, cx);
                    });
                    this.focus_mode = "roving";
                    this.status = "Focus strategy: roving".to_string();
                    cx.notify();
                }
            }
        }));
        subscriptions.push(cx.subscribe(&sequential_focus_button, {
            move |this, _, event: &ButtonEvent, cx| {
                if matches!(event, ButtonEvent::Click) {
                    this.toolbar.update(cx, |toolbar, cx| {
                        toolbar.set_focus_strategy(ControlGroupFocusStrategy::ActiveDescendant, cx);
                    });
                    this.focus_mode = "sequential";
                    this.status = "Focus strategy: sequential".to_string();
                    cx.notify();
                }
            }
        }));

        Self {
            look,
            entry,
            toolbar,
            roving_focus_button,
            sequential_focus_button,
            event_stream,
            status: "Paragraph editing toolbar".to_string(),
            focus_mode: "roving",
            _subscriptions: subscriptions,
        }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.toolbar.update(cx, |toolbar, cx| toolbar.notify_items(cx));
        self.roving_focus_button.update(cx, |_, cx| cx.notify());
        self.sequential_focus_button.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ToolbarControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(self.toolbar.clone())
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_center()
                        .gap(px(8.0))
                        .child(self.roving_focus_button.clone())
                        .child(self.sequential_focus_button.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.muted_text)
                                .child(format!("mode: {}", self.focus_mode)),
                        ),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(chrome.muted_text)
                        .child(self.status.clone()),
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

fn format_toolbar_event(event: &ToolbarEvent) -> Option<String> {
    match event {
        ToolbarEvent::Click { id } => Some(format!("ToolbarEvent::Click {{ id: \"{id}\" }}")),
        ToolbarEvent::Change { id, value } => {
            let payload = match value {
                ToolbarValue::Bool(on) => format!("bool({on})"),
                ToolbarValue::String(value) => format!("string(\"{value}\")"),
            };
            Some(format!("ToolbarEvent::Change {{ id: \"{id}\", value: {payload} }}"))
        }
        _ => None,
    }
}
