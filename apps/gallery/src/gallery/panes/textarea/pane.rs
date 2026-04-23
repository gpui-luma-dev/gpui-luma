use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::button::{Button, ButtonEvent};
use gpui_luma::controls::textarea::{TextArea, TextAreaEvent, Validator};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage_description, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct TextAreaPane {
    text_area: Entity<TextArea>,
    set_sample_button: Entity<Button>,
    clear_button: Entity<Button>,
    rows_down_button: Entity<Button>,
    rows_up_button: Entity<Button>,
    toggle_enabled_button: Entity<Button>,
    toggle_clean_on_escape_button: Entity<Button>,
    toggle_validation_button: Entity<Button>,
    enabled: bool,
    clean_on_escape: bool,
    strict_validation: bool,
    rows: usize,
    value: SharedString,
    change_count: usize,
    focus_count: usize,
    blur_count: usize,
    focused: bool,
    last_event: SharedString,
}

impl TextAreaPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let button_template = theme.button_template();

        Self {
            text_area: TextArea::new("gallery-textarea")
                .placeholder("Type multiple lines. Enter inserts newline.")
                .rows(5)
                .clean_on_escape(true)
                .template(theme.textarea_template())
                .spawn(cx),
            set_sample_button: action_button("textarea-set-sample", "Set Sample", &button_template, cx),
            clear_button: action_button("textarea-clear", "Clear", &button_template, cx),
            rows_down_button: action_button("textarea-rows-down", "Rows -", &button_template, cx),
            rows_up_button: action_button("textarea-rows-up", "Rows +", &button_template, cx),
            toggle_enabled_button: action_button("textarea-toggle-enabled", "Toggle Enabled", &button_template, cx),
            toggle_clean_on_escape_button: action_button(
                "textarea-toggle-clean-on-escape",
                "Toggle Escape Clear",
                &button_template,
                cx,
            ),
            toggle_validation_button: action_button(
                "textarea-toggle-validation",
                "Toggle Validation",
                &button_template,
                cx,
            ),
            enabled: true,
            clean_on_escape: true,
            strict_validation: false,
            rows: 5,
            value: SharedString::default(),
            change_count: 0,
            focus_count: 0,
            blur_count: 0,
            focused: false,
            last_event: SharedString::from("None"),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.text_area, |app, _, event: &TextAreaEvent, cx| {
            app.panes.textarea.handle_text_area_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.set_sample_button, |app, _, _: &ButtonEvent, cx| {
            app.panes.textarea.set_sample_value(cx);
        }));
        subscriptions.push(cx.subscribe(&self.clear_button, |app, _, _: &ButtonEvent, cx| {
            app.panes.textarea.clear_value(cx);
        }));
        subscriptions.push(cx.subscribe(&self.rows_down_button, |app, _, _: &ButtonEvent, cx| {
            app.panes.textarea.decrease_rows(cx);
        }));
        subscriptions.push(cx.subscribe(&self.rows_up_button, |app, _, _: &ButtonEvent, cx| {
            app.panes.textarea.increase_rows(cx);
        }));
        subscriptions.push(cx.subscribe(&self.toggle_enabled_button, |app, _, _: &ButtonEvent, cx| {
            app.panes.textarea.toggle_enabled(cx);
        }));
        subscriptions.push(cx.subscribe(&self.toggle_clean_on_escape_button, |app, _, _: &ButtonEvent, cx| {
            app.panes.textarea.toggle_clean_on_escape(cx);
        }));
        subscriptions.push(cx.subscribe(&self.toggle_validation_button, |app, _, _: &ButtonEvent, cx| {
            app.panes.textarea.toggle_validation(cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage_description(
            "TextArea",
            Some("Multiline input with configurable rows, escape-clear, validation, and focus telemetry."),
            "TextArea",
            div()
                .w(px(560.0))
                .max_w_full()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(self.text_area.clone())
                .child(div().flex().flex_wrap().gap(px(8.0)).children([
                    self.set_sample_button.clone().into_any_element(),
                    self.clear_button.clone().into_any_element(),
                    self.rows_down_button.clone().into_any_element(),
                    self.rows_up_button.clone().into_any_element(),
                    self.toggle_enabled_button.clone().into_any_element(),
                    self.toggle_clean_on_escape_button.clone().into_any_element(),
                    self.toggle_validation_button.clone().into_any_element(),
                ]))
                .child(render_telemetry(
                    &[
                        format!("Enabled: {}", self.enabled),
                        format!("Escape clear: {}", self.clean_on_escape),
                        format!("Strict validation: {}", self.strict_validation),
                        format!("Focused: {}", self.focused),
                        format!("Rows: {}", self.rows),
                        format!("Lines/chars: {}/{}", self.value.lines().count(), self.value.chars().count()),
                        format!(
                            "Events: changes={}, focuses={}, blurs={}",
                            self.change_count, self.focus_count, self.blur_count
                        ),
                        format!("Last event: {}", self.last_event),
                    ],
                    chrome.border,
                    chrome.panel_background,
                    chrome.body_text,
                    chrome.muted_text,
                ))
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.text_area, cx);
        notify_entity(&self.set_sample_button, cx);
        notify_entity(&self.clear_button, cx);
        notify_entity(&self.rows_down_button, cx);
        notify_entity(&self.rows_up_button, cx);
        notify_entity(&self.toggle_enabled_button, cx);
        notify_entity(&self.toggle_clean_on_escape_button, cx);
        notify_entity(&self.toggle_validation_button, cx);
    }

    fn current_validator(&self) -> Option<Validator> {
        if !self.strict_validation {
            return None;
        }

        Some(Arc::new(|value: &str| {
            value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == ' ' || ch == '\n' || ch == '\t' || ch == '.')
        }))
    }

    fn sync_text_area_settings(&mut self, cx: &mut Context<GalleryApp>) {
        let enabled = self.enabled;
        let clean_on_escape = self.clean_on_escape;
        let validator = self.current_validator();
        let rows = self.rows;

        self.text_area.update(cx, move |text_area, cx| {
            text_area.set_enabled(enabled, cx);
            text_area.set_clean_on_escape(clean_on_escape, cx);
            text_area.set_validator(validator, cx);
            text_area.set_rows(rows, cx);
        });
    }

    fn handle_text_area_event(&mut self, event: &TextAreaEvent, cx: &mut Context<GalleryApp>) {
        match event {
            TextAreaEvent::Change { value } => {
                self.value = value.clone().into();
                self.change_count += 1;
                self.last_event = format!("Change: {} chars", value.chars().count()).into();
            }
            TextAreaEvent::Focus => {
                self.focused = true;
                self.focus_count += 1;
                self.last_event = SharedString::from("Focus");
            }
            TextAreaEvent::Blur => {
                self.focused = false;
                self.blur_count += 1;
                self.last_event = SharedString::from("Blur");
            }
        }

        cx.notify();
    }

    fn set_sample_value(&mut self, cx: &mut Context<GalleryApp>) {
        let value = SharedString::from("Line one\nLine two\nLine three");
        self.value = value.clone();
        self.text_area.update(cx, |text_area, cx| text_area.set_value(value.as_ref(), cx));
        cx.notify();
    }

    fn clear_value(&mut self, cx: &mut Context<GalleryApp>) {
        self.value = SharedString::default();
        self.text_area.update(cx, |text_area, cx| text_area.set_value("", cx));
        cx.notify();
    }

    fn increase_rows(&mut self, cx: &mut Context<GalleryApp>) {
        self.rows = (self.rows + 1).min(12);
        self.sync_text_area_settings(cx);
        cx.notify();
    }

    fn decrease_rows(&mut self, cx: &mut Context<GalleryApp>) {
        self.rows = self.rows.saturating_sub(1).max(2);
        self.sync_text_area_settings(cx);
        cx.notify();
    }

    fn toggle_enabled(&mut self, cx: &mut Context<GalleryApp>) {
        self.enabled = !self.enabled;
        self.sync_text_area_settings(cx);
        cx.notify();
    }

    fn toggle_clean_on_escape(&mut self, cx: &mut Context<GalleryApp>) {
        self.clean_on_escape = !self.clean_on_escape;
        self.sync_text_area_settings(cx);
        cx.notify();
    }

    fn toggle_validation(&mut self, cx: &mut Context<GalleryApp>) {
        self.strict_validation = !self.strict_validation;
        self.sync_text_area_settings(cx);
        cx.notify();
    }
}

fn action_button(
    id: &'static str,
    label: &'static str,
    template: &Arc<dyn gpui_luma::controls::button::ButtonTemplate>,
    cx: &mut Context<GalleryApp>,
) -> Entity<Button> {
    Button::new(id).label(label).template(template.clone()).spawn(cx)
}

fn render_telemetry(
    lines: &[String],
    border: gpui::Hsla,
    background: gpui::Hsla,
    body: gpui::Hsla,
    muted: gpui::Hsla,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .border_1()
        .border_color(border)
        .rounded(px(8.0))
        .bg(background)
        .p(px(12.0))
        .child(div().text_size(px(11.0)).line_height(px(16.0)).text_color(muted).child("Telemetry"))
        .children(lines.iter().map(|line| {
            div()
                .font_family("Monaco")
                .text_size(px(11.0))
                .line_height(px(16.0))
                .text_color(body)
                .child(line.clone())
        }))
        .into_any_element()
}
