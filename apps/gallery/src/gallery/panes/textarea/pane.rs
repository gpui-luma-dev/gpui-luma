use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, TextRun, Window, div, font,
    prelude::*, px,
};
use gpui_luma::controls::button::{Button, ButtonEvent};
use gpui_luma::controls::checkbox::{Checkbox, CheckboxEvent};
use gpui_luma::controls::textarea::{
    TextArea, TextAreaClickHandler, TextAreaEvent, TextAreaHoverHandler, TextAreaKeyDownHandler, TextAreaLineMetric,
    TextAreaMouseDownHandler, TextAreaMouseMoveHandler, TextAreaMouseUpHandler, TextAreaRenderModel, TextAreaState,
    TextAreaTemplate, TextAreaTemplateHandlers, Validator,
};
use gpui_luma::theme::TextAreaTheme;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage_description, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct TextAreaPane {
    text_area: Entity<TextArea>,
    state_preview: Entity<TextAreaStatePreview>,
    set_sample_button: Entity<Button>,
    clear_button: Entity<Button>,
    enabled_checkbox: Entity<Checkbox>,
    clean_on_escape_checkbox: Entity<Checkbox>,
    validation_checkbox: Entity<Checkbox>,
    enabled: bool,
    clean_on_escape: bool,
    strict_validation: bool,
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
        let checkbox_template = theme.checkbox_template();
        let textarea_template = theme.textarea_template();

        Self {
            text_area: TextArea::new("gallery-textarea")
                .placeholder("Write a multiline message")
                .full_width(true)
                .rows(6)
                .clean_on_escape(true)
                .select_all_on_tab_focus(true)
                .template(textarea_template.clone())
                .theme(theme.textarea_theme())
                .spawn(cx),
            state_preview: cx.new(|_| TextAreaStatePreview::new(theme, textarea_template.clone())),
            set_sample_button: action_button("textarea-set-sample", "Set Sample", &button_template, cx),
            clear_button: action_button("textarea-clear", "Clear", &button_template, cx),
            enabled_checkbox: Checkbox::new("textarea-enabled")
                .label("Enabled")
                .checked(true)
                .template(checkbox_template.clone())
                .spawn(cx),
            clean_on_escape_checkbox: Checkbox::new("textarea-clean-on-escape")
                .label("Escape clears")
                .checked(true)
                .template(checkbox_template.clone())
                .spawn(cx),
            validation_checkbox: Checkbox::new("textarea-validation")
                .label("Strict validation")
                .template(checkbox_template)
                .spawn(cx),
            enabled: true,
            clean_on_escape: true,
            strict_validation: false,
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
        subscriptions.push(cx.subscribe(&self.enabled_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.textarea.handle_option_changed(TextAreaOption::Enabled, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.clean_on_escape_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.textarea.handle_option_changed(TextAreaOption::CleanOnEscape, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.validation_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.textarea.handle_option_changed(TextAreaOption::StrictValidation, event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();
        let value = self.value.as_ref();
        let line_count = if value.is_empty() { 0 } else { value.lines().count() };

        gallery_pane_with_usage_description(
            "TextArea",
            Some("Multiline input with hard-line editing, selection, escape-clear, validation, and fixed row height."),
            "TextArea",
            div()
                .w(px(620.0))
                .max_w_full()
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
                .child(self.state_preview.clone())
                .child(render_telemetry(
                    &[
                        format!("Enabled: {}", self.enabled),
                        format!("Escape clear: {}", self.clean_on_escape),
                        format!("Strict validation: {}", self.strict_validation),
                        format!("Focused: {}", self.focused),
                        format!("Length: {}", value.chars().count()),
                        format!("Lines: {}", line_count),
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
        notify_entity(&self.state_preview, cx);
        notify_entity(&self.set_sample_button, cx);
        notify_entity(&self.clear_button, cx);
        notify_entity(&self.enabled_checkbox, cx);
        notify_entity(&self.clean_on_escape_checkbox, cx);
        notify_entity(&self.validation_checkbox, cx);
    }

    fn current_validator(&self) -> Option<Validator> {
        if !self.strict_validation {
            return None;
        }

        Some(Arc::new(|value: &str| {
            value.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == ' ' || ch == '\n')
        }))
    }

    fn sync_text_area_settings(&mut self, cx: &mut Context<GalleryApp>) {
        let enabled = self.enabled;
        let clean_on_escape = self.clean_on_escape;
        let validator = self.current_validator();

        self.text_area.update(cx, move |text_area, cx| {
            text_area.set_enabled(enabled, cx);
            text_area.set_clean_on_escape(clean_on_escape, cx);
            text_area.set_validator(validator, cx);
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
        let value = SharedString::from(
            "Short line\nSeveral hard lines\nA very long line that should exceed the visible viewport width before wrapping is implemented\nMultibyte smoke test: café 東京\nEnough lines\nTo make vertical space\nFeel like a textarea",
        );
        self.value = value.clone();
        self.text_area.update(cx, |text_area, cx| text_area.set_value(value.as_ref(), cx));
        cx.notify();
    }

    fn clear_value(&mut self, cx: &mut Context<GalleryApp>) {
        self.value = SharedString::default();
        self.text_area.update(cx, |text_area, cx| text_area.set_value("", cx));
        cx.notify();
    }

    fn handle_option_changed(&mut self, option: TextAreaOption, event: &CheckboxEvent, cx: &mut Context<GalleryApp>) {
        let CheckboxEvent::Change { checked } = event;
        match option {
            TextAreaOption::Enabled => self.enabled = *checked,
            TextAreaOption::CleanOnEscape => self.clean_on_escape = *checked,
            TextAreaOption::StrictValidation => self.strict_validation = *checked,
        }
        self.sync_text_area_settings(cx);
        cx.notify();
    }
}

#[derive(Clone, Copy)]
enum TextAreaOption {
    Enabled,
    CleanOnEscape,
    StrictValidation,
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

#[derive(Clone)]
struct TextAreaStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn TextAreaTemplate>,
}

#[derive(Clone, Copy)]
struct TextAreaStateSample {
    id: &'static str,
    label: &'static str,
    state: TextAreaState,
    enabled: bool,
}

impl TextAreaStatePreview {
    fn new(theme: &GalleryThemePack, template: Arc<dyn TextAreaTemplate>) -> Self {
        Self { theme: theme.clone(), template }
    }
}

impl Render for TextAreaStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            TextAreaStateSample { id: "default", label: "Default", state: TextAreaState::default(), enabled: true },
            TextAreaStateSample {
                id: "hover",
                label: "Hover",
                state: TextAreaState { hovered: true, ..TextAreaState::default() },
                enabled: true,
            },
            TextAreaStateSample {
                id: "focus",
                label: "Focus",
                state: TextAreaState { focused: true, focus_visible: true, cursor: 7, ..TextAreaState::default() },
                enabled: true,
            },
            TextAreaStateSample {
                id: "selection",
                label: "Selection",
                state: TextAreaState {
                    hovered: true,
                    focused: true,
                    focus_visible: true,
                    cursor: 18,
                    selection_anchor: Some(4),
                    ..TextAreaState::default()
                },
                enabled: true,
            },
            TextAreaStateSample { id: "disabled", label: "Disabled", state: TextAreaState::default(), enabled: false },
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(10.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template state preview"),
            )
            .child(div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                samples.into_iter().map(|sample| {
                    render_state_sample(
                        &self.template,
                        self.theme.textarea_theme(),
                        sample,
                        chrome.muted_text,
                        window,
                        cx,
                    )
                }),
            ))
    }
}

fn render_state_sample(
    template: &Arc<dyn TextAreaTemplate>,
    theme: Arc<dyn TextAreaTheme>,
    sample: TextAreaStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("textarea-preview-{}", sample.id));
    let placeholder = SharedString::from("Placeholder");
    let value = SharedString::from("Preview\nText area");
    let line_metrics = textarea_line_metrics(value.as_ref(), theme, sample.state, sample.enabled, window);
    let model = TextAreaRenderModel {
        id: &id,
        placeholder: &placeholder,
        value: &value,
        enabled: sample.enabled,
        full_width: true,
        rows: 3,
        state: sample.state,
        caret_visible: sample.state.focused && sample.enabled,
        vertical_scroll: 0.0,
        line_metrics,
    };

    div()
        .w(px(180.0))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, textarea_preview_handlers(), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn textarea_line_metrics(
    value: &str,
    theme: Arc<dyn TextAreaTheme>,
    state: TextAreaState,
    enabled: bool,
    window: &mut Window,
) -> Vec<TextAreaLineMetric> {
    let appearance = theme.resolve(state, enabled);
    let mut metrics = Vec::new();
    let mut start = 0usize;
    let mut current = String::new();

    for (ix, ch) in value.chars().enumerate() {
        if ch == '\n' {
            metrics.push(shape_metric(start, ix, std::mem::take(&mut current), metrics.len(), &appearance, window));
            start = ix + 1;
        } else {
            current.push(ch);
        }
    }
    metrics.push(shape_metric(start, value.chars().count(), current, metrics.len(), &appearance, window));

    metrics
}

fn shape_metric(
    start: usize,
    end: usize,
    text: String,
    line_ix: usize,
    appearance: &gpui_luma::theme::TextAreaAppearance,
    window: &mut Window,
) -> TextAreaLineMetric {
    let run = TextRun {
        len: text.len(),
        font: {
            let mut font = font(".SystemUIFont");
            font.weight = appearance.typography.weight;
            font
        },
        color: appearance.foreground,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = window.text_system().shape_line(
        SharedString::from(text.to_owned()),
        px(appearance.typography.size),
        &[run],
        None,
    );
    let chars = text.chars().count();
    let mut character_offsets = Vec::with_capacity(chars + 1);
    for char_offset in 0..=chars {
        let byte_offset = text.chars().take(char_offset).map(char::len_utf8).sum();
        character_offsets.push(line.x_for_index(byte_offset).as_f32());
    }

    TextAreaLineMetric {
        start,
        end,
        text,
        y: line_ix as f32 * appearance.typography.line_height,
        height: appearance.typography.line_height,
        character_offsets,
    }
}

fn textarea_preview_handlers() -> TextAreaTemplateHandlers {
    TextAreaTemplateHandlers {
        hover: Box::new(noop_hover) as TextAreaHoverHandler,
        mouse_down: Box::new(noop_mouse_down) as TextAreaMouseDownHandler,
        mouse_move: Box::new(noop_mouse_move) as TextAreaMouseMoveHandler,
        mouse_up: Box::new(noop_mouse_up) as TextAreaMouseUpHandler,
        mouse_up_out: Box::new(noop_mouse_up) as TextAreaMouseUpHandler,
        click: Box::new(noop_click) as TextAreaClickHandler,
        key_down: Box::new(noop_key_down) as TextAreaKeyDownHandler,
        drag_move: Box::new(noop_drag_move),
    }
}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_mouse_down(_: &gpui::MouseDownEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_move(_: &gpui::MouseMoveEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_up(_: &gpui::MouseUpEvent, _: &mut Window, _: &mut App) {}

fn noop_click(_: &gpui::ClickEvent, _: &mut Window, _: &mut App) {}

fn noop_key_down(_: &gpui::KeyDownEvent, _: &mut Window, _: &mut App) {}

fn noop_drag_move(_: &gpui::DragMoveEvent<gpui_luma::controls::textarea::TextAreaDrag>, _: &mut Window, _: &mut App) {}
