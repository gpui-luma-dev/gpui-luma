use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString, TextRun, Window, div, font, prelude::*, px};
use gpui_luma::controls::textfield::{
    TextFieldRenderModel, TextFieldState, TextFieldTemplate, TextFieldTheme, TextFieldVariant,
};
use gpui_luma::theme::{ControlSize, StandardBoxScale};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::style::shared::preview_handlers::input_textfield_handlers;
use crate::studio::style::shared::shell::section_shell_with_width;

#[derive(Clone, Copy)]
struct InputTextFieldSample {
    id: &'static str,
    label: &'static str,
    state: TextFieldState,
    enabled: bool,
}

pub(crate) fn render_textfield_template_section(
    look: Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    section_shell_with_width(
        960.0,
        "Text Field",
        "Default, hover, focus, active, and disabled states.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        render_input_textfield_body(&look, window, cx),
    )
}

fn render_input_textfield_body(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let template = look.textfield_template();
    let theme = look.textfield_theme();
    let samples = input_textfield_samples();
    let chrome = look.chrome();

    div()
        .flex()
        .flex_wrap()
        .items_start()
        .justify_center()
        .gap(px(12.0))
        .children(samples.iter().copied().map(|sample| {
            render_input_textfield_sample(&template, theme.clone(), sample, chrome.muted_text, window, cx)
        }))
        .into_any_element()
}

fn render_input_textfield_sample(
    template: &Arc<dyn TextFieldTemplate>,
    theme: Arc<dyn TextFieldTheme>,
    sample: InputTextFieldSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("theme-studio-textfield-preview-{}", sample.id));
    let placeholder = SharedString::from("Placeholder");
    let value = SharedString::from("Preview");
    let look = input_textfield_look(&theme, sample.state, sample.enabled, window);
    let character_offsets =
        input_textfield_character_offsets(value.as_ref(), theme, sample.state, sample.enabled, window);
    let model = TextFieldRenderModel {
        id: &id,
        placeholder: &placeholder,
        value: &value,
        prefix_icon: None,
        variant: TextFieldVariant::Standard,
        enabled: sample.enabled,
        full_width: false,
        state: sample.state,
        caret_visible: sample.state.focused && sample.enabled,
        horizontal_scroll: 0.0,
        character_offsets,
        look,
    };

    div()
        .w(px(180.0))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, input_textfield_handlers(), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn input_textfield_samples() -> [InputTextFieldSample; 5] {
    [
        InputTextFieldSample { id: "default", label: "Default", state: TextFieldState::default(), enabled: true },
        InputTextFieldSample {
            id: "hover",
            label: "Hover",
            state: TextFieldState { hovered: true, ..TextFieldState::default() },
            enabled: true,
        },
        InputTextFieldSample {
            id: "focus",
            label: "Focus",
            state: TextFieldState { focused: true, focus_visible: true, cursor: 7, ..TextFieldState::default() },
            enabled: true,
        },
        InputTextFieldSample {
            id: "active",
            label: "Active",
            state: TextFieldState {
                hovered: true,
                focused: true,
                focus_visible: true,
                cursor: 7,
                selection_anchor: Some(0),
                ..TextFieldState::default()
            },
            enabled: true,
        },
        InputTextFieldSample { id: "disabled", label: "Disabled", state: TextFieldState::default(), enabled: false },
    ]
}

fn input_textfield_look(
    theme: &Arc<dyn TextFieldTheme>,
    state: TextFieldState,
    enabled: bool,
    window: &Window,
) -> gpui_luma::controls::textfield::TextFieldLook {
    let scale = StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), window.scale_factor());
    theme.resolve_look(TextFieldVariant::Standard, state, enabled, &scale)
}

fn input_textfield_character_offsets(
    value: &str,
    theme: Arc<dyn TextFieldTheme>,
    state: TextFieldState,
    enabled: bool,
    window: &mut Window,
) -> Vec<f32> {
    let look = input_textfield_look(&theme, state, enabled, window);
    let run = TextRun {
        len: value.len(),
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
            .shape_line(SharedString::from(value.to_owned()), px(look.typography.size), &[run], None);
    let chars = value.chars().count();
    let mut character_offsets = Vec::with_capacity(chars + 1);
    for char_offset in 0..=chars {
        let byte_offset = value.chars().take(char_offset).map(char::len_utf8).sum();
        character_offsets.push(line.x_for_index(byte_offset).as_f32());
    }
    character_offsets
}
