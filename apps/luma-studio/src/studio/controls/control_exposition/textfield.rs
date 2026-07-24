use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::textfield::{TextField, TextFieldEvent};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "TextFieldEvent::Change { value }",
        trigger: "User edits text (typing, paste, cut, delete)",
        notes: "Emitted after each committed edit. Assign local model state from the payload.",
    },
    EventReferenceSpec {
        event: "TextFieldEvent::Submit { value }",
        trigger: "Enter key while focused",
        notes: "Use for form submission or committing a filter query.",
    },
    EventReferenceSpec {
        event: "TextFieldEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the control",
        notes: "Useful for form-level focus rings or clearing sibling field errors.",
    },
    EventReferenceSpec {
        event: "TextFieldEvent::EnabledChanged { enabled }",
        trigger: "TextField::set_enabled changes enabled state",
        notes: "Programmatic transition; disabling clears hover and mouse selection state.",
    },
    EventReferenceSpec {
        event: "(none)",
        trigger: "TextField::set_value",
        notes: "Programmatic value sync updates the field visually but does not emit Change.",
    },
    EventReferenceSpec {
        event: "(none)",
        trigger: "Disabled interaction",
        notes: "Pointer and keyboard input are ignored while disabled.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "TextField",
        surface: "Type",
        notes: "Entity<TextFieldControl> — single-line editable text input.",
    },
    PublicInterfaceSpec {
        symbol: "TextFieldEvent",
        surface: "Event",
        notes: "Non-exhaustive enum: Change, Submit, FocusChanged, EnabledChanged.",
    },
    PublicInterfaceSpec {
        symbol: "textfield::new(id)",
        surface: "Factory",
        notes: "Starts a TextFieldBuilder with the default textfield template.",
    },
    PublicInterfaceSpec {
        symbol: "TextFieldBuilder::placeholder / value",
        surface: "Builder",
        notes: "Initial placeholder and text content before spawn.",
    },
    PublicInterfaceSpec {
        symbol: "TextFieldBuilder::full_width / size / variant",
        surface: "Builder",
        notes: "Layout width, ControlSize, and TextFieldVariant emphasis.",
    },
    PublicInterfaceSpec {
        symbol: "TextFieldBuilder::prefix_icon / validator",
        surface: "Builder",
        notes: "Leading icon slot and optional input validation hook.",
    },
    PublicInterfaceSpec {
        symbol: "TextFieldBuilder::clean_on_escape / select_all_on_tab_focus",
        surface: "Builder",
        notes: "Keyboard behavior toggles for escape-to-clear and tab focus select-all.",
    },
    PublicInterfaceSpec {
        symbol: "TextFieldBuilder::template / with_template_modifier",
        surface: "Builder",
        notes: "First- and second-tier customization ladder hooks on the template root.",
    },
    PublicInterfaceSpec {
        symbol: "TextFieldBuilder::spawn(cx)",
        surface: "Builder",
        notes: "Materializes the GPUI entity; subscribe with cx.subscribe for TextFieldEvent.",
    },
    PublicInterfaceSpec {
        symbol: "TextField::value / set_value",
        surface: "Entity",
        notes: "Read or programmatically update the current text content.",
    },
    PublicInterfaceSpec {
        symbol: "TextField::set_enabled / set_placeholder / set_validator",
        surface: "Entity",
        notes: "Lifecycle and validation updates after spawn.",
    },
    PublicInterfaceSpec {
        symbol: "TextField::is_focused / state",
        surface: "Entity",
        notes: "Focus snapshot and TextFieldState for diagnostics or tests.",
    },
    PublicInterfaceSpec {
        symbol: "look.textfield(id)",
        surface: "Look",
        notes: "ShadcnLookControlExt factory — binds the default Shadcn textfield template.",
    },
];

pub struct TextFieldControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: TextField,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl TextFieldControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("textfield").expect("textfield catalog entry");
        let preview = look
            .textfield("controls-doc-textfield-preview")
            .placeholder("Email address")
            .full_width(true)
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-textfield-event-log",
                "Edit the preview field; all emitted TextFieldEvent variants appear in the stream below.",
            )
        });

        let subscription = cx.subscribe(&preview, {
            let event_stream = event_stream.clone();
            move |_, _, event: &TextFieldEvent, cx| {
                let line = format_textfield_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                    cx.notify();
                });
            }
        });

        Self { look, entry, preview, event_stream, _subscriptions: vec![subscription] }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for TextFieldControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(div().w_full().child(self.preview.clone()))
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

fn format_textfield_event(event: &TextFieldEvent) -> String {
    match event {
        TextFieldEvent::Change { value } => format!("TextFieldEvent::Change {{ value: \"{value}\" }}"),
        TextFieldEvent::Submit { value } => format!("TextFieldEvent::Submit {{ value: \"{value}\" }}"),
        TextFieldEvent::FocusChanged { focused } => {
            format!("TextFieldEvent::FocusChanged {{ focused: {focused} }}")
        }
        TextFieldEvent::EnabledChanged { enabled } => {
            format!("TextFieldEvent::EnabledChanged {{ enabled: {enabled} }}")
        }
        _ => "TextFieldEvent::(unknown)".to_string(),
    }
}
