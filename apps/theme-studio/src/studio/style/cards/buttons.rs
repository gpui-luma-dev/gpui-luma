use super::super::*;

const TEMPLATE_GRID_COLUMN_WIDTH: f32 = 152.0;
const TEMPLATE_GRID_GAP: f32 = 10.0;

pub(in crate::studio::style::style_guide) fn render_button_template_matrix_section(
    look: Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let template: Arc<dyn ButtonTemplate<()>> = default_button_template();
    let samples = [
        ButtonStateSample { id: "default", header: "default", state: InteractionState::default() },
        ButtonStateSample {
            id: "hover",
            header: "hover",
            state: InteractionState { hovered: true, ..InteractionState::default() },
        },
        ButtonStateSample {
            id: "focused",
            header: "focused",
            state: InteractionState { focused: true, ..InteractionState::default() },
        },
        ButtonStateSample {
            id: "pressed",
            header: "pressed",
            state: InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
        },
        ButtonStateSample {
            id: "disabled",
            header: "disabled",
            state: InteractionState { disabled: true, ..InteractionState::default() },
        },
    ];
    let variants = [
        ButtonTemplateVariant::TextButton,
        ButtonTemplateVariant::TextButtonLeadingIcon,
        ButtonTemplateVariant::TextButtonTrailingIcon,
        ButtonTemplateVariant::IconButton,
    ];

    section_shell_with_width(
        960.0,
        "Buttons",
        "State and variant matrix.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        gpui::hsla(0.0, 0.0, 0.0, 0.0),
        div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .children([
                render_button_size_preview_row(look.as_ref(), &template, chrome.muted_text, window, cx),
                render_button_template_section(
                    look.as_ref(),
                    &template,
                    "Primary",
                    ShadcnButtonStyle::Primary,
                    &variants,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_button_template_section(
                    look.as_ref(),
                    &template,
                    "Secondary",
                    ShadcnButtonStyle::Secondary,
                    &variants,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_button_template_section(
                    look.as_ref(),
                    &template,
                    "Outline",
                    ShadcnButtonStyle::Outline,
                    &variants,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_button_template_section(
                    look.as_ref(),
                    &template,
                    "Ghost",
                    ShadcnButtonStyle::Ghost,
                    &variants,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ),
            ])
            .into_any_element(),
    )
}

fn render_button_size_preview_row(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let sizes = [(ButtonSize::Sm, "Small"), (ButtonSize::Md, "Medium"), (ButtonSize::Lg, "Large")];

    div()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(10.0))
        .child(
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child("Standard sizes"),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_start()
                .gap(px(12.0))
                .children(sizes.into_iter().map(|(size, label)| {
                    render_button_size_sample(template, look, ShadcnButtonStyle::Primary, size, label, window, cx)
                })),
        )
        .into_any_element()
}

pub(in crate::studio::style::style_guide) fn render_choice_template_matrix_section(
    look: Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let states = [
        ChoiceTemplateStateSample { id: "default", label: "Default", state: InteractionState::default() },
        ChoiceTemplateStateSample {
            id: "hover",
            label: "Hover",
            state: InteractionState { hovered: true, ..InteractionState::default() },
        },
        ChoiceTemplateStateSample {
            id: "focused",
            label: "Focused",
            state: InteractionState { focused: true, ..InteractionState::default() },
        },
        ChoiceTemplateStateSample {
            id: "pressed",
            label: "Pressed",
            state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
        },
        ChoiceTemplateStateSample {
            id: "disabled",
            label: "Disabled",
            state: InteractionState { disabled: true, ..InteractionState::default() },
        },
    ];
    let controls = [
        ChoiceTemplateControl::Radio,
        ChoiceTemplateControl::Checkbox,
        ChoiceTemplateControl::Switch,
        ChoiceTemplateControl::Toggle,
        ChoiceTemplateControl::ToggleIcon,
    ];

    section_shell_with_width(
        960.0,
        "Choice Controls",
        "Selection states across control families.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(14.0))
            .child(render_choice_matrix(&look, "Selected", true, &states, &controls, chrome.muted_text, window, cx))
            .child(render_choice_matrix(&look, "Unselected", false, &states, &controls, chrome.muted_text, window, cx))
            .into_any_element(),
    )
}

pub(in crate::studio::style::style_guide) fn render_toggle_template_matrix_section(
    look: Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let template = look.toggle_template(ShadcnButtonStyle::Secondary);
    let samples = [
        ToggleStateSample { id: "default", header: "default", state: InteractionState::default() },
        ToggleStateSample {
            id: "hover",
            header: "hover",
            state: InteractionState { hovered: true, ..InteractionState::default() },
        },
        ToggleStateSample {
            id: "focused",
            header: "focused",
            state: InteractionState { focused: true, ..InteractionState::default() },
        },
        ToggleStateSample {
            id: "pressed",
            header: "pressed",
            state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
        },
        ToggleStateSample {
            id: "disabled",
            header: "disabled",
            state: InteractionState { disabled: true, ..InteractionState::default() },
        },
    ];
    let variants = [
        ToggleTemplateVariant::TextUnselected,
        ToggleTemplateVariant::TextSelected,
        ToggleTemplateVariant::RoundIconUnselected,
        ToggleTemplateVariant::RoundIconSelected,
    ];

    section_shell_with_width(
        960.0,
        "Toggles",
        "Selected and unselected interaction states.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .children([
                render_toggle_template_section(
                    &look,
                    &template,
                    "Primary",
                    ShadcnButtonStyle::Primary,
                    &variants,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_toggle_template_section(
                    &look,
                    &template,
                    "Secondary",
                    ShadcnButtonStyle::Secondary,
                    &variants,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ),
            ])
            .into_any_element(),
    )
}

fn render_button_template_section(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    section_label: &'static str,
    style: ShadcnButtonStyle,
    variants: &[ButtonTemplateVariant],
    samples: &[ButtonStateSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_start()
        .gap(px(10.0))
        .child(render_vertical_section_rail(section_label, label_color))
        .child(render_button_matrix_grid(template, look, style, variants, samples, label_color, window, cx))
        .into_any_element()
}

fn render_choice_header_row(controls: &[ChoiceTemplateControl], label_color: gpui::Hsla) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(div().w(px(28.0)))
        .children(controls.iter().map(|control| {
            div()
                .w(px(152.0))
                .flex()
                .justify_center()
                .text_xs()
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(control.header())
        }))
        .into_any_element()
}

fn render_button_matrix_grid(
    template: &Arc<dyn ButtonTemplate<()>>,
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
    variants: &[ButtonTemplateVariant],
    samples: &[ButtonStateSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let mut grid = GridLayout::new()
        .rows(variants.len() + 1)
        .columns((0..samples.len()).map(|_| GridTrack::Px(TEMPLATE_GRID_COLUMN_WIDTH)))
        .gap_x(TEMPLATE_GRID_GAP)
        .gap_y(8.0);

    for (col, sample) in samples.iter().enumerate() {
        grid = grid.child(render_button_header_cell(sample.header, label_color), 0, col);
    }

    for (row, variant) in variants.iter().enumerate() {
        for (col, sample) in samples.iter().enumerate() {
            grid = grid.child(
                render_button_state_sample(template, look, style, *variant, sample, window, cx),
                row + 1,
                col,
            );
        }
    }

    grid.into_any_element()
}

fn render_button_header_cell(label: &'static str, label_color: gpui::Hsla) -> AnyElement {
    div()
        .w_full()
        .flex()
        .justify_center()
        .text_xs()
        .line_height(px(15.0))
        .text_color(label_color)
        .child(label)
        .into_any_element()
}

fn render_button_state_sample(
    template: &Arc<dyn ButtonTemplate<()>>,
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
    variant: ButtonTemplateVariant,
    sample: &ButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!(
        "theme-studio-button-preview-{}-{}-{}",
        shadcn_style_id(style),
        variant.id(),
        sample.id
    ));
    let look = button_look_for_style(Arc::new(look.clone()), style);
    let model = ButtonRenderModel {
        id,
        data: (),
        content: variant.content(),
        role: if matches!(variant, ButtonTemplateVariant::IconButton) {
            ButtonFamilyRole::Icon
        } else {
            ButtonFamilyRole::Text
        },
        size: ButtonSize::Md,
        state: sample.state,
        round: variant.round(),
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: Some(look),
    };

    div()
        .w_full()
        .flex()
        .justify_center()
        .items_center()
        .child(template.render(&model, window, cx))
        .into_any_element()
}

fn render_button_size_sample(
    template: &Arc<dyn ButtonTemplate<()>>,
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
    size: ButtonSize,
    label: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("theme-studio-button-size-preview-{}-{}", shadcn_style_id(style), label));
    let model = ButtonRenderModel {
        id,
        data: (),
        content: Arc::new(|_, _| div().child("Button").into_any_element()),
        role: ButtonFamilyRole::Text,
        size,
        state: InteractionState::default(),
        round: false,
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: Some(button_look_for_style(Arc::new(look.clone()), style)),
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(look.chrome().muted_text).child(label))
        .into_any_element()
}

fn render_choice_matrix(
    look: &Arc<ShadcnLook>,
    section_label: &'static str,
    selected: bool,
    states: &[ChoiceTemplateStateSample],
    controls: &[ChoiceTemplateControl],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_stretch()
        .gap(px(8.0))
        .child(render_vertical_section_rail(section_label, label_color))
        .child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(8.0))
                .child(render_choice_header_row(controls, label_color))
                .children(
                    states
                        .iter()
                        .map(|state| render_choice_state_row(look, state, selected, controls, label_color, window, cx)),
                ),
        )
        .into_any_element()
}

fn render_choice_state_row(
    look: &Arc<ShadcnLook>,
    state_sample: &ChoiceTemplateStateSample,
    selected: bool,
    controls: &[ChoiceTemplateControl],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_stretch()
        .gap(px(10.0))
        .child(render_vertical_state_rail(state_sample.label, state_sample.id, label_color))
        .children(
            controls
                .iter()
                .map(|control| render_choice_control_cell(look, *control, selected, state_sample, window, cx)),
        )
        .into_any_element()
}

fn render_choice_control_cell(
    look: &Arc<ShadcnLook>,
    control: ChoiceTemplateControl,
    selected: bool,
    state_sample: &ChoiceTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("theme-studio-choice-template-preview-{}-{}", state_sample.id, control.id()));
    let model = ButtonRenderModel {
        id,
        data: selected,
        content: control.content(selected),
        role: ButtonFamilyRole::Text,
        size: ButtonSize::Md,
        state: state_sample.state,
        round: control.round(),
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: None,
    };

    div()
        .w(px(152.0))
        .flex()
        .items_center()
        .justify_center()
        .child(choice_template_for_control(look, control).render(&model, window, cx))
        .into_any_element()
}

fn choice_template_for_control(
    look: &Arc<ShadcnLook>,
    control: ChoiceTemplateControl,
) -> Arc<dyn ButtonTemplate<bool>> {
    match control {
        ChoiceTemplateControl::Radio => look.radio_button_template(ShadcnButtonStyle::Primary),
        ChoiceTemplateControl::Checkbox => look.checkbox_template(ShadcnButtonStyle::Primary),
        ChoiceTemplateControl::Switch => look.switch_template(ShadcnButtonStyle::Primary),
        ChoiceTemplateControl::Toggle | ChoiceTemplateControl::ToggleIcon => {
            look.toggle_template(ShadcnButtonStyle::Secondary)
        }
    }
}

fn render_toggle_template_section(
    look: &Arc<ShadcnLook>,
    template: &Arc<dyn ButtonTemplate<bool>>,
    section_label: &'static str,
    style: ShadcnButtonStyle,
    variants: &[ToggleTemplateVariant],
    samples: &[ToggleStateSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_start()
        .gap(px(8.0))
        .child(render_vertical_section_rail(section_label, label_color))
        .child(render_toggle_matrix_grid(template, look, style, variants, samples, label_color, window, cx))
        .into_any_element()
}

fn render_toggle_matrix_grid(
    template: &Arc<dyn ButtonTemplate<bool>>,
    look: &Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
    variants: &[ToggleTemplateVariant],
    samples: &[ToggleStateSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let mut grid = GridLayout::new()
        .rows(variants.len() + 1)
        .columns((0..samples.len()).map(|_| GridTrack::Px(TEMPLATE_GRID_COLUMN_WIDTH)))
        .gap_x(TEMPLATE_GRID_GAP)
        .gap_y(8.0);

    for (col, sample) in samples.iter().enumerate() {
        grid = grid.child(render_toggle_header_cell(sample.header, label_color), 0, col);
    }

    for (row, variant) in variants.iter().enumerate() {
        for (col, sample) in samples.iter().enumerate() {
            grid = grid.child(
                render_toggle_state_sample(template, look, style, *variant, sample, window, cx),
                row + 1,
                col,
            );
        }
    }

    grid.into_any_element()
}

fn render_toggle_header_cell(label: &'static str, label_color: gpui::Hsla) -> AnyElement {
    div()
        .w_full()
        .flex()
        .justify_center()
        .text_size(px(11.0))
        .line_height(px(15.0))
        .text_color(label_color)
        .child(label)
        .into_any_element()
}

fn render_toggle_state_sample(
    template: &Arc<dyn ButtonTemplate<bool>>,
    look: &Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
    variant: ToggleTemplateVariant,
    sample: &ToggleStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selected = variant.selected();
    let id = SharedString::from(format!(
        "theme-studio-toggle-preview-{}-{}-{}",
        shadcn_style_id(style),
        variant.id(),
        sample.id
    ));
    let model = ButtonRenderModel {
        id,
        data: selected,
        content: variant.content(),
        role: ButtonFamilyRole::Toggle { selected },
        size: ButtonSize::Md,
        state: sample.state,
        round: variant.round(),
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: Some(toggle_button_look_for_style(look.clone(), style)),
    };

    div()
        .w_full()
        .flex()
        .justify_center()
        .items_center()
        .child(template.render(&model, window, cx))
        .into_any_element()
}

fn toggle_button_look_for_style(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> gpui_luma::controls::command::button::ButtonLookSource<bool> {
    Arc::new(move |model| {
        let role = ButtonFamilyRole::Toggle { selected: model.data };
        match style {
            ShadcnButtonStyle::Primary => theme.as_ref().resolve_primary_button(role, model.size, model.state),
            ShadcnButtonStyle::Secondary => theme.as_ref().resolve_secondary_button(role, model.size, model.state),
            ShadcnButtonStyle::Outline => theme.as_ref().resolve_outline_button(role, model.size, model.state),
            ShadcnButtonStyle::Ghost => theme.as_ref().resolve_ghost_button(role, model.size, model.state),
        }
    })
}

fn button_look_for_style(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> gpui_luma::controls::command::button::ButtonLookSource<()> {
    Arc::new(move |model| match style {
        ShadcnButtonStyle::Primary => theme.as_ref().resolve_primary_button(model.role, model.size, model.state),
        ShadcnButtonStyle::Secondary => theme.as_ref().resolve_secondary_button(model.role, model.size, model.state),
        ShadcnButtonStyle::Outline => theme.as_ref().resolve_outline_button(model.role, model.size, model.state),
        ShadcnButtonStyle::Ghost => theme.as_ref().resolve_ghost_button(model.role, model.size, model.state),
    })
}

fn shadcn_style_id(style: ShadcnButtonStyle) -> &'static str {
    match style {
        ShadcnButtonStyle::Primary => "primary",
        ShadcnButtonStyle::Secondary => "secondary",
        ShadcnButtonStyle::Outline => "outline",
        ShadcnButtonStyle::Ghost => "ghost",
    }
}

pub(in crate::studio::style::style_guide) fn render_lucide_icon(icon: LucideIcon) -> AnyElement {
    div()
        .font_family("lucide")
        .text_size(px(16.0))
        .line_height(px(16.0))
        .child(char::from(icon).to_string())
        .into_any_element()
}

pub(in crate::studio::style::style_guide) fn round_icon_glyph(selected: bool) -> AnyElement {
    let icon = if selected { LucideIcon::Check } else { LucideIcon::Plus };
    render_lucide_icon(icon)
}
