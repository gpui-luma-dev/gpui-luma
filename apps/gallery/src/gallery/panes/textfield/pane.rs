use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, TextRun, Window, div, font,
    prelude::*, px,
};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::textfield::{
    TextField, TextFieldClickHandler, TextFieldEvent, TextFieldHoverHandler, TextFieldKeyDownHandler,
    TextFieldMouseDownHandler, TextFieldMouseMoveHandler, TextFieldMouseUpHandler, TextFieldRenderModel,
    TextFieldState, TextFieldTemplate, TextFieldTemplateHandlers, TextFieldVariant, Validator,
};
use gpui_luma::controls::textfield::TextFieldTheme;
use gpui_luma::theme::{ControlSize, StandardBoxScale};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_textfield_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct TextFieldPane {
    text_field: TextField,
    state_preview: Entity<TextFieldStatePreview>,
    inspector: Entity<ColorInspectorShell>,
    set_sample_button: Entity<Button>,
    clear_button: Entity<Button>,
    enabled_checkbox: Entity<Button<bool>>,
    clean_on_escape_checkbox: Entity<Button<bool>>,
    validation_checkbox: Entity<Button<bool>>,
    enabled: bool,
    clean_on_escape: bool,
    strict_validation: bool,
    value: SharedString,
    change_count: usize,
    submit_count: usize,
    focus_count: usize,
    blur_count: usize,
    focused: bool,
    last_event: SharedString,
}

impl TextFieldPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree =
            spawn_color_inspector_tree("textfield-inspector-tree", look.clone(), build_textfield_inspect_tree, cx);
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "textfield-inspector",
                "textfield-inspector-split",
                "textfield-inspector-detail",
                build_textfield_inspect_tree,
                cx,
            )
        });

        Self {
            text_field: look
                .textfield("gallery-textfield")
                .placeholder("Type and press Enter")
                .prefix_icon(LucideIcon::Search)
                .variant(TextFieldVariant::Standard)
                .full_width(true)
                .clean_on_escape(true)
                .select_all_on_tab_focus(true)
                .spawn(cx),
            state_preview: cx.new(|_| TextFieldStatePreview::new(look.clone())),
            inspector,
            set_sample_button: action_button("textfield-set-sample", "Set Sample", &look, cx),
            clear_button: action_button("textfield-clear", "Clear", &look, cx),
            enabled_checkbox: look
                .checkbox("textfield-enabled")
                .with_data(true)
                .content(|_, _| div().child("Enabled").into_any_element())
                .spawn(cx),
            clean_on_escape_checkbox: look
                .checkbox("textfield-clean-on-escape")
                .with_data(true)
                .content(|_, _| div().child("Escape clears").into_any_element())
                .spawn(cx),
            validation_checkbox: look
                .checkbox("textfield-validation")
                .with_data(false)
                .content(|_, _| div().child("Strict validation").into_any_element())
                .spawn(cx),
            enabled: true,
            clean_on_escape: true,
            strict_validation: false,
            value: SharedString::default(),
            change_count: 0,
            submit_count: 0,
            focus_count: 0,
            blur_count: 0,
            focused: false,
            last_event: SharedString::from("None"),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.text_field, |app, _, event: &TextFieldEvent, cx| {
            app.panes.textfield.handle_text_field_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.set_sample_button, |app, _, _: &ButtonEvent, cx| {
            app.panes.textfield.set_sample_value(cx);
        }));
        subscriptions.push(cx.subscribe(&self.clear_button, |app, _, _: &ButtonEvent, cx| {
            app.panes.textfield.clear_value(cx);
        }));
        subscriptions.push(cx.subscribe(&self.enabled_checkbox, |app, _, event: &ButtonEvent, cx| {
            app.panes.textfield.handle_option_changed(TextFieldOption::Enabled, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.clean_on_escape_checkbox, |app, _, event: &ButtonEvent, cx| {
            app.panes.textfield.handle_option_changed(TextFieldOption::CleanOnEscape, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.validation_checkbox, |app, _, event: &ButtonEvent, cx| {
            app.panes.textfield.handle_option_changed(TextFieldOption::StrictValidation, event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();

        gallery_pane_with_inspector(
            "TextField",
            div()
                .w(px(560.0))
                .max_w_full()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(
                    div()
                        .max_w(px(560.0))
                        .text_size(px(13.0))
                        .line_height(px(18.0))
                        .text_color(chrome.muted_text)
                        .child(
                            "Single-line input with selection, submit on Enter, escape-clear, validation, and optional prefix icon.",
                        ),
                )
                .child(self.text_field.clone())
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
                        format!("Value: {:?}", self.value),
                        format!(
                            "Events: changes={}, submits={}, focuses={}, blurs={}",
                            self.change_count, self.submit_count, self.focus_count, self.blur_count
                        ),
                        format!("Last event: {}", self.last_event),
                    ],
                    chrome.border,
                    chrome.panel_background,
                    chrome.body_text,
                    chrome.muted_text,
                ))
                .into_any_element(),
            self.inspector.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.text_field, cx);
        notify_entity(&self.state_preview, cx);
        notify_entity(&self.set_sample_button, cx);
        notify_entity(&self.clear_button, cx);
        notify_entity(&self.enabled_checkbox, cx);
        notify_entity(&self.clean_on_escape_checkbox, cx);
        notify_entity(&self.validation_checkbox, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn current_validator(&self) -> Option<Validator> {
        if !self.strict_validation {
            return None;
        }

        Some(Arc::new(|value: &str| value.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == ' ')))
    }

    fn sync_text_field_settings(&mut self, cx: &mut Context<GalleryApp>) {
        let enabled = self.enabled;
        let clean_on_escape = self.clean_on_escape;
        let validator = self.current_validator();

        self.text_field.update(cx, move |text_field, cx| {
            text_field.set_enabled(enabled, cx);
            text_field.set_clean_on_escape(clean_on_escape, cx);
            text_field.set_validator(validator, cx);
        });
    }

    fn handle_text_field_event(&mut self, event: &TextFieldEvent, cx: &mut Context<GalleryApp>) {
        match event {
            TextFieldEvent::Change { value } => {
                self.value = value.clone().into();
                self.change_count += 1;
                self.last_event = format!("Change: {value}").into();
            }
            TextFieldEvent::Submit { value } => {
                self.value = value.clone().into();
                self.submit_count += 1;
                self.last_event = format!("Submit: {value}").into();
            }
            TextFieldEvent::Focus => {
                self.focused = true;
                self.focus_count += 1;
                self.last_event = SharedString::from("Focus");
            }
            TextFieldEvent::Blur => {
                self.focused = false;
                self.blur_count += 1;
                self.last_event = SharedString::from("Blur");
            }
        }

        cx.notify();
    }

    fn set_sample_value(&mut self, cx: &mut Context<GalleryApp>) {
        let value = SharedString::from("Hello GPUI Luma");
        self.value = value.clone();
        self.text_field.update(cx, |text_field, cx| text_field.set_value(value.as_ref(), cx));
        cx.notify();
    }

    fn clear_value(&mut self, cx: &mut Context<GalleryApp>) {
        self.value = SharedString::default();
        self.text_field.update(cx, |text_field, cx| text_field.set_value("", cx));
        cx.notify();
    }

    fn handle_option_changed(&mut self, option: TextFieldOption, _event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match option {
            TextFieldOption::Enabled => {
                self.enabled = !self.enabled;
                self.enabled_checkbox.update(cx, |b, cx| b.set_data(self.enabled, cx));
                self.enabled
            }
            TextFieldOption::CleanOnEscape => {
                self.clean_on_escape = !self.clean_on_escape;
                self.clean_on_escape_checkbox.update(cx, |b, cx| b.set_data(self.clean_on_escape, cx));
                self.clean_on_escape
            }
            TextFieldOption::StrictValidation => {
                self.strict_validation = !self.strict_validation;
                self.validation_checkbox.update(cx, |b, cx| b.set_data(self.strict_validation, cx));
                self.strict_validation
            }
        };
        self.sync_text_field_settings(cx);
        cx.notify();
    }
}

#[derive(Clone, Copy)]
enum TextFieldOption {
    Enabled,
    CleanOnEscape,
    StrictValidation,
}

fn action_button(
    id: &'static str,
    label: &'static str,
    look: &Arc<ShadcnLook>,
    cx: &mut Context<GalleryApp>,
) -> Entity<Button> {
    look.secondary_button(id).label(label).spawn(cx)
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
struct TextFieldStatePreview {
    look: Arc<ShadcnLook>,
    template: Arc<dyn TextFieldTemplate>,
}

#[derive(Clone, Copy)]
struct TextFieldStateSample {
    id: &'static str,
    label: &'static str,
    state: TextFieldState,
    enabled: bool,
}

impl TextFieldStatePreview {
    fn new(look: Arc<ShadcnLook>) -> Self {
        Self { look: look.clone(), template: look.textfield_template() }
    }
}

impl Render for TextFieldStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let surface_textfield_theme = self.look.textfield_theme();
        let soft_textfield_theme = self.look.soft_textfield_theme();
        let samples = [
            TextFieldStateSample { id: "default", label: "Default", state: TextFieldState::default(), enabled: true },
            TextFieldStateSample {
                id: "hover",
                label: "Hover",
                state: TextFieldState { hovered: true, ..TextFieldState::default() },
                enabled: true,
            },
            TextFieldStateSample {
                id: "focus",
                label: "Focus",
                state: TextFieldState { focused: true, focus_visible: true, cursor: 7, ..TextFieldState::default() },
                enabled: true,
            },
            TextFieldStateSample {
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
            TextFieldStateSample {
                id: "disabled",
                label: "Disabled",
                state: TextFieldState::default(),
                enabled: false,
            },
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
                    .child("Surface variant state preview"),
            )
            .child(div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                samples.into_iter().map(|sample| {
                    render_state_sample(
                        &self.template,
                        surface_textfield_theme.clone(),
                        sample,
                        chrome.muted_text,
                        window,
                        cx,
                    )
                }),
            ))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Soft variant state preview"),
            )
            .child(div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                samples.into_iter().map(|sample| {
                    render_state_sample(
                        &self.template,
                        soft_textfield_theme.clone(),
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
    template: &Arc<dyn TextFieldTemplate>,
    theme: Arc<dyn TextFieldTheme>,
    sample: TextFieldStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let variant = TextFieldVariant::Standard;
    let id = SharedString::from(format!("textfield-preview-{}", sample.id));
    let placeholder = SharedString::from("Placeholder");
    let value = SharedString::from("Preview");
    let appearance = preview_textfield_appearance(&theme, sample.state, sample.enabled, window);
    let character_offsets = textfield_character_offsets(value.as_ref(), theme, sample.state, sample.enabled, window);
    let model = TextFieldRenderModel {
        id: &id,
        placeholder: &placeholder,
        value: &value,
        prefix_icon: None,
        variant,
        enabled: sample.enabled,
        full_width: false,
        state: sample.state,
        caret_visible: sample.state.focused && sample.enabled,
        horizontal_scroll: 0.0,
        character_offsets,
        appearance,
    };

    div()
        .w(px(180.0))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, textfield_preview_handlers(), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn textfield_character_offsets(
    value: &str,
    theme: Arc<dyn TextFieldTheme>,
    state: TextFieldState,
    enabled: bool,
    window: &mut Window,
) -> Vec<f32> {
    let appearance = preview_textfield_appearance(&theme, state, enabled, window);
    let run = TextRun {
        len: value.len(),
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
        SharedString::from(value.to_owned()),
        px(appearance.typography.size),
        &[run],
        None,
    );
    let chars = value.chars().count();
    let mut character_offsets = Vec::with_capacity(chars + 1);
    for char_offset in 0..=chars {
        let byte_offset = value.chars().take(char_offset).map(char::len_utf8).sum();
        character_offsets.push(line.x_for_index(byte_offset).as_f32());
    }

    character_offsets
}

fn textfield_preview_handlers() -> TextFieldTemplateHandlers {
    TextFieldTemplateHandlers {
        hover: Box::new(noop_hover) as TextFieldHoverHandler,
        mouse_down: Box::new(noop_mouse_down) as TextFieldMouseDownHandler,
        mouse_move: Box::new(noop_mouse_move) as TextFieldMouseMoveHandler,
        mouse_up: Box::new(noop_mouse_up) as TextFieldMouseUpHandler,
        mouse_up_out: Box::new(noop_mouse_up) as TextFieldMouseUpHandler,
        click: Box::new(noop_click) as TextFieldClickHandler,
        key_down: Box::new(noop_key_down) as TextFieldKeyDownHandler,
    }
}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_mouse_down(_: &gpui::MouseDownEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_move(_: &gpui::MouseMoveEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_up(_: &gpui::MouseUpEvent, _: &mut Window, _: &mut App) {}

fn noop_click(_: &gpui::ClickEvent, _: &mut Window, _: &mut App) {}

fn preview_textfield_appearance(
    theme: &Arc<dyn TextFieldTheme>,
    state: TextFieldState,
    enabled: bool,
    window: &Window,
) -> gpui_luma::controls::textfield::TextFieldAppearance {
    let scale = StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), window.scale_factor());
    theme.resolve_appearance(TextFieldVariant::Standard, state, enabled, &scale)
}

fn noop_key_down(_: &gpui::KeyDownEvent, _: &mut Window, _: &mut App) {}
