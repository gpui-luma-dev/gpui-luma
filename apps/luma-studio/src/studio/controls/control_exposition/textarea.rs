use std::sync::Arc;

use gpui::{Context, Entity, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::checkbox::{Checkbox, CheckboxEvent};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::textarea::{TextArea, TextAreaEvent, Validator};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "TextAreaEvent::Change { value }",
        trigger: "User edits text (typing, paste, cut, delete)",
        notes: "Emitted after each committed edit.",
    },
    EventReferenceSpec {
        event: "TextAreaEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the control",
        notes: "Useful for form-level focus coordination.",
    },
    EventReferenceSpec {
        event: "TextAreaEvent::EnabledChanged { enabled }",
        trigger: "TextArea::set_enabled changes enabled state",
        notes: "Programmatic lifecycle transition.",
    },
    EventReferenceSpec {
        event: "(none)",
        trigger: "TextArea::set_value",
        notes: "Programmatic value sync does not emit Change.",
    },
    EventReferenceSpec { event: "(none)", trigger: "Disabled interaction", notes: "Input ignored while disabled." },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "TextArea",
        surface: "Type",
        notes: "Entity<TextAreaControl> — multiline editable text input.",
    },
    PublicInterfaceSpec { symbol: "TextAreaEvent", surface: "Event", notes: "Change, FocusChanged, EnabledChanged." },
    PublicInterfaceSpec { symbol: "look.textarea(id)", surface: "Look", notes: "ShadcnLookControlExt factory." },
    PublicInterfaceSpec {
        symbol: "TextAreaBuilder::placeholder / rows / full_width",
        surface: "Builder",
        notes: "Placeholder copy, fixed row height, and width.",
    },
    PublicInterfaceSpec {
        symbol: "TextAreaBuilder::clean_on_escape / validator",
        surface: "Builder",
        notes: "Escape-to-clear policy and optional validation hook.",
    },
    PublicInterfaceSpec {
        symbol: "TextAreaBuilder::spawn(cx)",
        surface: "Builder",
        notes: "Materialize entity; subscribe for TextAreaEvent.",
    },
];

pub struct TextAreaControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    text_area: Entity<TextArea>,
    event_stream: Entity<ControlEventStream>,
    set_sample_button: Entity<Button>,
    clear_button: Entity<Button>,
    enabled_checkbox: Checkbox,
    clean_on_escape_checkbox: Checkbox,
    validation_checkbox: Checkbox,
    enabled: bool,
    clean_on_escape: bool,
    strict_validation: bool,
    _subscriptions: Vec<Subscription>,
}

impl TextAreaControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("textarea").expect("textarea catalog entry");
        let text_area = look
            .textarea("controls-doc-textarea")
            .placeholder("Write a multiline message")
            .full_width(true)
            .rows(6)
            .clean_on_escape(true)
            .select_all_on_tab_focus(true)
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-textarea-event-log",
                "Edit the text area; TextAreaEvent variants appear below.",
            )
        });
        let set_sample_button = look.secondary_button("controls-textarea-set-sample").label("Set Sample").spawn(cx);
        let clear_button = look.secondary_button("controls-textarea-clear").label("Clear").spawn(cx);
        let enabled_checkbox = look
            .checkbox("controls-textarea-enabled")
            .with_data(true)
            .content(|_, _| div().child("Enabled").into_any_element())
            .spawn(cx);
        let clean_on_escape_checkbox = look
            .checkbox("controls-textarea-clean-on-escape")
            .with_data(true)
            .content(|_, _| div().child("Escape clears").into_any_element())
            .spawn(cx);
        let validation_checkbox = look
            .checkbox("controls-textarea-validation")
            .with_data(false)
            .content(|_, _| div().child("Strict validation").into_any_element())
            .spawn(cx);

        let mut subscriptions = vec![cx.subscribe(&text_area, {
            let event_stream = event_stream.clone();
            move |_, _, event: &TextAreaEvent, cx| {
                if let Some(line) = format_textarea_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        })];
        subscriptions.push(cx.subscribe(&set_sample_button, {
            let text_area = text_area.clone();
            move |_, _, event: &ButtonEvent, cx| {
                if !event.is_click() {
                    return;
                }
                let value = SharedString::from(
                    "Short line\nSeveral hard lines\nA very long line that should exceed the visible viewport width before wrapping is implemented\nMultibyte smoke test: café 東京\nEnough lines\nTo make vertical space\nFeel like a textarea",
                );
                text_area.update(cx, |area, cx| area.set_value(value.as_ref(), cx));
            }
        }));
        subscriptions.push(cx.subscribe(&clear_button, {
            let text_area = text_area.clone();
            move |_, _, event: &ButtonEvent, cx| {
                if !event.is_click() {
                    return;
                }
                text_area.update(cx, |area, cx| area.set_value("", cx));
            }
        }));
        for (checkbox, option) in [
            (enabled_checkbox.clone(), TextAreaOption::Enabled),
            (clean_on_escape_checkbox.clone(), TextAreaOption::CleanOnEscape),
            (validation_checkbox.clone(), TextAreaOption::StrictValidation),
        ] {
            subscriptions.push(cx.subscribe(&checkbox, move |this, _, event: &CheckboxEvent, cx| {
                this.handle_option_changed(option, event, cx);
            }));
        }

        Self {
            look,
            entry,
            text_area,
            event_stream,
            set_sample_button,
            clear_button,
            enabled_checkbox,
            clean_on_escape_checkbox,
            validation_checkbox,
            enabled: true,
            clean_on_escape: true,
            strict_validation: false,
            _subscriptions: subscriptions,
        }
    }

    fn current_validator(&self) -> Option<Validator> {
        if !self.strict_validation {
            return None;
        }
        Some(Arc::new(|value: &str| {
            value.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == ' ' || ch == '\n')
        }))
    }

    fn sync_text_area_settings(&mut self, cx: &mut Context<Self>) {
        let enabled = self.enabled;
        let clean_on_escape = self.clean_on_escape;
        let validator = self.current_validator();
        self.text_area.update(cx, move |text_area, cx| {
            text_area.set_enabled(enabled, cx);
            text_area.set_clean_on_escape(clean_on_escape, cx);
            text_area.set_validator(validator, cx);
        });
    }

    fn handle_option_changed(&mut self, option: TextAreaOption, event: &CheckboxEvent, cx: &mut Context<Self>) {
        let CheckboxEvent::Change { checked } = event else {
            return;
        };
        match option {
            TextAreaOption::Enabled => self.enabled = *checked,
            TextAreaOption::CleanOnEscape => self.clean_on_escape = *checked,
            TextAreaOption::StrictValidation => self.strict_validation = *checked,
        }
        self.sync_text_area_settings(cx);
        cx.notify();
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.text_area.update(cx, |_, cx| cx.notify());
        self.set_sample_button.update(cx, |_, cx| cx.notify());
        self.clear_button.update(cx, |_, cx| cx.notify());
        self.enabled_checkbox.update(cx, |_, cx| cx.notify());
        self.clean_on_escape_checkbox.update(cx, |_, cx| cx.notify());
        self.validation_checkbox.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

#[derive(Clone, Copy)]
enum TextAreaOption {
    Enabled,
    CleanOnEscape,
    StrictValidation,
}

impl Render for TextAreaControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .max_w(px(620.0))
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(self.text_area.clone())
                .child(div().flex().flex_wrap().gap(px(8.0)).children([
                    self.set_sample_button.clone().into_any_element(),
                    self.clear_button.clone().into_any_element(),
                ]))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(14.0))
                        .child(self.enabled_checkbox.clone())
                        .child(self.clean_on_escape_checkbox.clone())
                        .child(self.validation_checkbox.clone()),
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

fn format_textarea_event(event: &TextAreaEvent) -> Option<String> {
    match event {
        TextAreaEvent::Change { value } => {
            Some(format!("TextAreaEvent::Change {{ value: \"{}\" }}", value.replace('\n', "\\n")))
        }
        TextAreaEvent::FocusChanged { focused } => {
            Some(format!("TextAreaEvent::FocusChanged {{ focused: {focused} }}"))
        }
        TextAreaEvent::EnabledChanged { enabled } => {
            Some(format!("TextAreaEvent::EnabledChanged {{ enabled: {enabled} }}"))
        }
        _ => None,
    }
}
