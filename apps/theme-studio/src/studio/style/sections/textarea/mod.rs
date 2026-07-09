use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString, TextRun, Window, div, font, prelude::*, px};
use gpui_luma::controls::textarea::{
    TextAreaLineMetric, TextAreaRenderModel, TextAreaState, TextAreaTemplate, TextAreaTheme, ThemedTextAreaTemplate,
};
use gpui_luma::theme::{ControlSize, StandardBoxScale};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::style::shared::preview_handlers::input_textarea_handlers;
use crate::studio::style::shared::shell::section_shell_with_width;

#[derive(Clone, Copy)]
struct InputTextAreaSample {
    id: &'static str,
    label: &'static str,
    state: TextAreaState,
    enabled: bool,
}

pub(crate) fn render_textarea_template_section(look: Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();

    section_shell_with_width(
        960.0,
        "Text Area",
        "Default, hover, focus, active, and disabled states.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        render_input_textarea_body(&look, window, cx),
    )
}

fn render_input_textarea_body(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let theme = look.textarea_theme();
    let template: Arc<dyn TextAreaTemplate> = Arc::new(ThemedTextAreaTemplate::new(theme.clone()));
    let samples = input_textarea_samples();

    div()
        .flex()
        .flex_wrap()
        .items_start()
        .justify_center()
        .gap(px(12.0))
        .children(samples.iter().copied().map(|sample| {
            render_input_textarea_sample(&template, theme.clone(), sample, chrome.muted_text, window, cx)
        }))
        .into_any_element()
}

fn render_input_textarea_sample(
    template: &Arc<dyn TextAreaTemplate>,
    theme: Arc<dyn TextAreaTheme>,
    sample: InputTextAreaSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("theme-studio-textarea-preview-{}", sample.id));
    let placeholder = SharedString::from("Placeholder");
    let value = SharedString::from("Preview\nText area");
    let line_metrics = input_textarea_line_metrics(value.as_ref(), theme, sample.state, sample.enabled, window);
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
        .child(template.render(&model, input_textarea_handlers(), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn input_textarea_samples() -> [InputTextAreaSample; 5] {
    [
        InputTextAreaSample { id: "default", label: "Default", state: TextAreaState::default(), enabled: true },
        InputTextAreaSample {
            id: "hover",
            label: "Hover",
            state: TextAreaState { hovered: true, ..TextAreaState::default() },
            enabled: true,
        },
        InputTextAreaSample {
            id: "focus",
            label: "Focus",
            state: TextAreaState { focused: true, focus_visible: true, cursor: 7, ..TextAreaState::default() },
            enabled: true,
        },
        InputTextAreaSample {
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
        InputTextAreaSample { id: "disabled", label: "Disabled", state: TextAreaState::default(), enabled: false },
    ]
}

fn input_textarea_line_metrics(
    value: &str,
    theme: Arc<dyn TextAreaTheme>,
    state: TextAreaState,
    enabled: bool,
    window: &mut Window,
) -> Vec<TextAreaLineMetric> {
    let look = input_textarea_look(&theme, state, enabled, window);
    let mut metrics = Vec::new();
    let mut start = 0usize;
    let mut current = String::new();

    for (ix, ch) in value.chars().enumerate() {
        if ch == '\n' {
            metrics.push(input_textarea_shape_metric(
                start,
                ix,
                std::mem::take(&mut current),
                metrics.len(),
                &look,
                window,
            ));
            start = ix + 1;
        } else {
            current.push(ch);
        }
    }
    metrics.push(input_textarea_shape_metric(start, value.chars().count(), current, metrics.len(), &look, window));
    metrics
}

fn input_textarea_shape_metric(
    start: usize,
    end: usize,
    text: String,
    line_ix: usize,
    look: &gpui_luma::controls::textarea::TextAreaLook,
    window: &mut Window,
) -> TextAreaLineMetric {
    let run = TextRun {
        len: text.len(),
        font: {
            let mut font = font(".SystemUIFont");
            font.weight = look.typography.weight;
            font
        },
        color: look.foreground,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line =
        window
            .text_system()
            .shape_line(SharedString::from(text.to_owned()), px(look.typography.size), &[run], None);
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
        y: line_ix as f32 * look.typography.line_height,
        height: look.typography.line_height,
        character_offsets,
    }
}

fn input_textarea_look(
    theme: &Arc<dyn TextAreaTheme>,
    state: TextAreaState,
    enabled: bool,
    window: &Window,
) -> gpui_luma::controls::textarea::TextAreaLook {
    let scale = StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), window.scale_factor());
    theme.resolve_look(state, enabled, &scale)
}
