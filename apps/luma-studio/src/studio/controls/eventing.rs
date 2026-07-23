use std::sync::Arc;

use gpui::{Context, Entity, FontWeight, Render, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::ButtonEvent;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use super::doc_card::controls_mono_font;
use super::event_log_view::{EventLogView, EventLogViewLookExt, shadcn_event_log_theme};

#[derive(Clone, Copy)]
struct ButtonEventSpec {
    event: &'static str,
    trigger: &'static str,
    notes: &'static str,
    sampled: bool,
}

const BUTTON_EVENT_SPECS: &[ButtonEventSpec] = &[
    ButtonEventSpec {
        event: "ButtonEvent::Click",
        trigger: "Pointer click (primary button)",
        notes: "Emitted when the control is enabled. Subscribe in the parent with cx.subscribe.",
        sampled: true,
    },
    ButtonEventSpec {
        event: "ButtonEvent::Click",
        trigger: "Keyboard activate (Space, Enter)",
        notes: "Same variant when focused and ActivateControl runs.",
        sampled: true,
    },
    ButtonEventSpec {
        event: "(none)",
        trigger: "Hover / focus changes",
        notes: "InteractionState updates for rendering only; buttons do not emit hover or focus events.",
        sampled: false,
    },
    ButtonEventSpec {
        event: "(none)",
        trigger: "Disabled interaction",
        notes: "Pointer and keyboard activation are ignored while disabled.",
        sampled: false,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonVariant {
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

pub struct ButtonEventStream {
    look: Arc<ShadcnLook>,
    event_log: Entity<EventLogView>,
    next_index: usize,
}

impl ButtonEventStream {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let event_log = look
            .event_log_view("controls-button-event-log")
            .placeholder("Event history will appear here…")
            .full_width(true)
            .rows(6)
            .font_family(controls_mono_font())
            .spawn(cx);

        Self { look, event_log, next_index: 1 }
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.event_log.update(cx, |log, cx| {
            log.set_theme(shadcn_event_log_theme(look), cx);
            log.set_font_family(controls_mono_font(), cx);
        });
        cx.notify();
    }

    pub fn record_event(
        &mut self,
        variant: ButtonVariant,
        button_id: &'static str,
        event: &ButtonEvent,
        cx: &mut Context<Self>,
    ) {
        let ButtonEvent::Click = event;
        let line = format!("[{}] ButtonEvent::Click — {} (\"{button_id}\")", self.next_index, variant.label());
        self.next_index += 1;
        self.event_log.update(cx, |log, cx| log.append_line(&line, cx));
    }
}

impl Render for ButtonEventStream {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let body_style = self.look.typography_scale(ShadcnTextSize::Xs);
            let title_style = self.look.typography_scale(ShadcnTextSize::Sm);

            div()
                .id("controls-button-event-stream")
                .w_full()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .typography_style(title_style)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(chrome.title_text)
                                .child("Eventing"),
                        )
                        .child(
                            div()
                                .w_full()
                                .typography_style(body_style)
                                .text_color(chrome.muted_text)
                                .child("Subscribe with cx.subscribe and handle ButtonEvent::Click in the parent."),
                        ),
                )
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .typography_style(body_style)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(chrome.title_text)
                                .child("Event stream"),
                        )
                        .child(self.event_log.clone()),
                )
        })
    }
}

pub(crate) fn render_button_possible_events_section(look: &ShadcnLook) -> gpui::AnyElement {
    let chrome = look.chrome();
    let body_style = look.typography_scale(ShadcnTextSize::Xs);
    let mono = controls_mono_font();

    render_event_reference_table(body_style, mono, chrome.title_text, chrome.muted_text)
}

fn render_event_reference_table(
    body_style: gpui_luma::theme::LumaTextStyle,
    mono: gpui::SharedString,
    title_color: gpui::Hsla,
    muted_text: gpui::Hsla,
) -> gpui::AnyElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .typography_style(body_style)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child("Possible events"),
        )
        .child(
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(render_event_reference_row(
                    "Event",
                    "Trigger",
                    "Notes",
                    body_style,
                    mono.clone(),
                    title_color,
                    true,
                    None,
                ))
                .children(BUTTON_EVENT_SPECS.iter().map(|spec| {
                    render_event_reference_row(
                        spec.event,
                        spec.trigger,
                        spec.notes,
                        body_style,
                        mono.clone(),
                        muted_text,
                        false,
                        Some(spec.sampled),
                    )
                })),
        )
        .into_any_element()
}

fn render_event_reference_row(
    event: &'static str,
    trigger: &'static str,
    notes: &'static str,
    body_style: gpui_luma::theme::LumaTextStyle,
    mono: gpui::SharedString,
    text_color: gpui::Hsla,
    header: bool,
    sampled: Option<bool>,
) -> gpui::AnyElement {
    div()
        .w_full()
        .grid()
        .grid_cols(12)
        .gap(px(10.0))
        .items_start()
        .child(
            div()
                .col_span(3)
                .font_family(mono)
                .typography_style(body_style)
                .font_weight(if header {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(text_color)
                .child(event),
        )
        .child(
            div()
                .col_span(4)
                .typography_style(body_style)
                .font_weight(if header {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(text_color)
                .child(trigger),
        )
        .child(
            div()
                .col_span(5)
                .flex()
                .items_start()
                .gap(px(8.0))
                .child(div().flex_1().min_w(px(0.0)).typography_style(body_style).text_color(text_color).child(notes))
                .when_some(sampled.and_then(|sampled| sampled.then_some(())), |row, _| {
                    row.child(
                        div()
                            .flex_shrink_0()
                            .typography_style(body_style)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_color)
                            .opacity(0.72)
                            .child("sampled"),
                    )
                }),
        )
        .into_any_element()
}
