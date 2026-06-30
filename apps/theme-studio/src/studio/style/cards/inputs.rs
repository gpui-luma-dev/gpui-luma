use super::super::*;

pub(in crate::studio::style::style_guide) fn render_input_controls_template_section(
    look: Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    section_shell_with_width(
        960.0,
        "Input Controls",
        "Scrollbar, slider, text field, and text area states.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .children([
                render_input_scrollbar_preview(&look, window, cx),
                render_input_slider_preview(&look, window, cx),
                render_input_textfield_preview(&look, window, cx),
                render_input_textarea_preview(&look, window, cx),
            ])
            .into_any_element(),
    )
}

pub(super) fn render_input_section_heading(title: &'static str, muted_text: gpui::Hsla) -> AnyElement {
    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(muted_text)
        .child(title)
        .into_any_element()
}

fn render_input_scrollbar_preview(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let samples = input_interaction_samples();
    let template = look.scrollbar_template();

    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(render_input_section_heading("Scrollbar", chrome.muted_text))
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(12.0))
                .child(render_input_scrollbar_row(
                    &template,
                    "Horizontal",
                    ScrollbarOrientation::Horizontal,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ))
                .child(render_input_scrollbar_row(
                    &template,
                    "Vertical",
                    ScrollbarOrientation::Vertical,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                )),
        )
        .into_any_element()
}

fn render_input_scrollbar_row(
    template: &Arc<dyn ScrollbarTemplate>,
    row_label: &'static str,
    orientation: ScrollbarOrientation,
    samples: &[InputInteractionSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(row_label))
        .child(
            div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                samples.iter().copied().map(|sample| {
                    render_input_scrollbar_sample(template, orientation, sample, label_color, window, cx)
                }),
            ),
        )
        .into_any_element()
}

fn render_input_scrollbar_sample(
    template: &Arc<dyn ScrollbarTemplate>,
    orientation: ScrollbarOrientation,
    sample: InputInteractionSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let orientation_id = match orientation {
        ScrollbarOrientation::Horizontal => "horizontal",
        ScrollbarOrientation::Vertical => "vertical",
    };
    let id = SharedString::from(format!("theme-studio-scrollbar-preview-{}-{}", orientation_id, sample.id));
    let range = ControlRange::from(0..220);
    let value = 40.0;
    let model = ScrollbarRenderModel {
        id: &id,
        orientation,
        range,
        step: 20.0,
        page_step: 80.0,
        value,
        percentage: range.percentage(value),
        thumb_fraction: 0.54,
        length: None,
        enabled: !sample.state.disabled,
        state: sample.state,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, input_scrollbar_handlers(), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn render_input_slider_preview(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let samples = input_interaction_samples();
    let template = look.slider_template();

    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(render_input_section_heading("Slider", chrome.muted_text))
        .child(
            div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                samples
                    .iter()
                    .copied()
                    .map(|sample| render_input_slider_sample(&template, sample, chrome.muted_text, window, cx)),
            ),
        )
        .into_any_element()
}

fn render_input_slider_sample(
    template: &Arc<dyn SliderTemplate>,
    sample: InputInteractionSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("theme-studio-slider-preview-{}", sample.id));
    let range = ControlRange::from(1..100);
    let value = 41.0;
    let position = range.percentage(value);
    let thumb_id = ThumbId::next();
    let thumbs = [SliderThumbValue { id: thumb_id, position, preview: None, role: SliderThumbRole::Value }];
    let track_segments = build_track_segments(TrackPresentation::Fill, position, &[], range);
    let model = SliderRenderModel {
        id: &id,
        strategy: SliderInputStrategy::Horizontal,
        orientation: SliderInputStrategy::Horizontal.orientation(),
        presentation: TrackPresentation::Fill,
        size: ControlSize::Md,
        thumb_size: None,
        range,
        step: 10.0,
        thumbs: &thumbs,
        track_segments,
        reversed: false,
        wrapping: false,
        enabled: !sample.state.disabled,
        corner_radius: None,
        thumb_policy: SliderThumbPolicy::default(),
        active_thumb_id: Some(thumb_id),
        state: sample.state,
        domain_track: None,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(div().w(px(320.0)).max_w(px(360.0)).child(template.render(
            &model,
            input_slider_handlers(),
            thumb_id,
            window,
            cx,
        )))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn render_input_textfield_preview(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let template = look.textfield_template();
    let theme = look.textfield_theme();
    let samples = input_textfield_samples();

    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(render_input_section_heading("TextField", chrome.muted_text))
        .child(div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
            samples.iter().copied().map(|sample| {
                render_input_textfield_sample(&template, theme.clone(), sample, chrome.muted_text, window, cx)
            }),
        ))
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

fn render_input_textarea_preview(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let theme = look.textarea_theme();
    let template: Arc<dyn TextAreaTemplate> = Arc::new(ThemedTextAreaTemplate::new(theme.clone()));
    let samples = input_textarea_samples();

    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(render_input_section_heading("TextArea", chrome.muted_text))
        .child(div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
            samples.iter().copied().map(|sample| {
                render_input_textarea_sample(&template, theme.clone(), sample, chrome.muted_text, window, cx)
            }),
        ))
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

fn input_interaction_samples() -> [InputInteractionSample; 5] {
    [
        InputInteractionSample { id: "default", label: "Standard", state: InteractionState::default() },
        InputInteractionSample {
            id: "hover",
            label: "Hover",
            state: InteractionState { hovered: true, ..InteractionState::default() },
        },
        InputInteractionSample {
            id: "focus",
            label: "Focus",
            state: InteractionState { focused: true, ..InteractionState::default() },
        },
        InputInteractionSample {
            id: "active",
            label: "Active",
            state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
        },
        InputInteractionSample {
            id: "disabled",
            label: "Disabled",
            state: InteractionState { disabled: true, ..InteractionState::default() },
        },
    ]
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

fn input_scrollbar_handlers() -> ScrollbarTemplateHandlers {
    ScrollbarTemplateHandlers {
        track_bounds: Box::new(input_noop_bounds) as ScrollbarBoundsHandler,
        thumb_bounds: Box::new(input_noop_bounds) as ScrollbarBoundsHandler,
        hover: Box::new(input_noop_hover) as ScrollbarHoverHandler,
        mouse_down: Box::new(input_noop_mouse_down) as ScrollbarMouseDownHandler,
        mouse_up: Box::new(input_noop_mouse_up) as ScrollbarMouseUpHandler,
        mouse_up_out: Box::new(input_noop_mouse_up) as ScrollbarMouseUpHandler,
        drag_move: Box::new(input_noop_scrollbar_drag_move) as ScrollbarDragMoveHandler,
        scroll_wheel: Box::new(input_noop_scroll_wheel) as ScrollbarScrollWheelHandler,
    }
}

fn input_slider_handlers() -> SliderTemplateHandlers {
    SliderTemplateHandlers {
        track_bounds: Box::new(input_noop_bounds) as SliderBoundsHandler,
        hover: Box::new(input_noop_hover) as SliderHoverHandler,
        mouse_down: Box::new(input_noop_mouse_down) as SliderMouseDownHandler,
        mouse_move: Box::new(input_noop_mouse_move) as SliderMouseMoveHandler,
        mouse_up: Box::new(input_noop_mouse_up) as SliderMouseUpHandler,
        mouse_up_out: Box::new(input_noop_mouse_up) as SliderMouseUpHandler,
        drag_move: Arc::new(input_noop_slider_drag_move),
        thumb_mouse_down: Arc::new(input_noop_thumb_mouse_down),
    }
}

pub(super) fn input_textfield_handlers() -> TextFieldTemplateHandlers {
    TextFieldTemplateHandlers {
        hover: Box::new(input_noop_hover) as TextFieldHoverHandler,
        mouse_down: Box::new(input_noop_mouse_down) as TextFieldMouseDownHandler,
        mouse_move: Box::new(input_noop_mouse_move) as TextFieldMouseMoveHandler,
        mouse_up: Box::new(input_noop_mouse_up) as TextFieldMouseUpHandler,
        mouse_up_out: Box::new(input_noop_mouse_up) as TextFieldMouseUpHandler,
        click: Box::new(input_noop_click) as TextFieldClickHandler,
        key_down: Box::new(input_noop_key_down) as TextFieldKeyDownHandler,
    }
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

fn input_textarea_handlers() -> TextAreaTemplateHandlers {
    TextAreaTemplateHandlers {
        hover: Box::new(input_noop_hover) as TextAreaHoverHandler,
        mouse_down: Box::new(input_noop_mouse_down) as TextAreaMouseDownHandler,
        mouse_move: Box::new(input_noop_mouse_move) as TextAreaMouseMoveHandler,
        mouse_up: Box::new(input_noop_mouse_up) as TextAreaMouseUpHandler,
        mouse_up_out: Box::new(input_noop_mouse_up) as TextAreaMouseUpHandler,
        click: Box::new(input_noop_click) as TextAreaClickHandler,
        key_down: Box::new(input_noop_key_down) as TextAreaKeyDownHandler,
        drag_move: Box::new(input_noop_textarea_drag_move),
    }
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

pub(super) fn input_noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

pub(super) fn input_noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

pub(super) fn input_noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}

pub(super) fn input_noop_mouse_move(_: &MouseMoveEvent, _: &mut Window, _: &mut App) {}

pub(super) fn input_noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

pub(super) fn input_noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}

fn input_noop_key_down(_: &KeyDownEvent, _: &mut Window, _: &mut App) {}

fn input_noop_scrollbar_drag_move(_: &DragMoveEvent<ScrollbarDrag>, _: &mut Window, _: &mut App) {}

fn input_noop_scroll_wheel(_: &ScrollWheelEvent, _: &mut Window, _: &mut App) {}

fn input_noop_slider_drag_move(_: &DragMoveEvent<SliderDrag>, _: &mut Window, _: &mut App) {}

fn input_noop_thumb_mouse_down(_: &ThumbId, _: &MouseDownEvent, _: &mut Window, _: &mut App) {}

fn input_noop_textarea_drag_move(_: &DragMoveEvent<TextAreaDrag>, _: &mut Window, _: &mut App) {}
