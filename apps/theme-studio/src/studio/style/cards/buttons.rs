use super::super::*;

const BUTTON_TABLE_STATE_COLUMN_WIDTH: f32 = 152.0;
const TOGGLE_TEXT_TABLE_STATE_COLUMN_WIDTH: f32 = 112.0;
const BUTTON_TABLE_ROW_HEIGHT: f32 = 140.0;
const BUTTON_TABLE_RADIUS_COLUMN_WIDTH: f32 = 136.0;
const BUTTON_TABLE_SIZE_RADIUS_ROW_HEIGHT: f32 = 55.0;
const BUTTON_TABLE_SIZE_VARIANT_COLUMN_WIDTH: f32 = 120.0;
const BUTTON_TABLE_SIZE_HEADER_HEIGHT: f32 = 40.0;

const BUTTON_TABLE_VARIANT_GAP: f32 = 10.0;

const SIZE_PREVIEW_STYLE: ShadcnButtonStyle = ShadcnButtonStyle::Primary;

const BUTTON_SIZES: [(ButtonSize, &'static str); 3] =
    [(ButtonSize::Sm, "Small"), (ButtonSize::Md, "Medium"), (ButtonSize::Lg, "Large")];

struct ButtonStyleVariantDef {
    label: &'static str,
    description: &'static str,
    style: ShadcnButtonStyle,
}

const BUTTON_STYLE_VARIANTS: [ButtonStyleVariantDef; 4] = [
    ButtonStyleVariantDef { label: "Primary", description: "High emphasis actions", style: ShadcnButtonStyle::Primary },
    ButtonStyleVariantDef {
        label: "Secondary",
        description: "Lower emphasis actions",
        style: ShadcnButtonStyle::Secondary,
    },
    ButtonStyleVariantDef {
        label: "Outline",
        description: "Subtle secondary controls",
        style: ShadcnButtonStyle::Outline,
    },
    ButtonStyleVariantDef { label: "Ghost", description: "Quiet utility actions", style: ShadcnButtonStyle::Ghost },
];

const CHOICE_STYLE_VARIANTS: [ButtonStyleVariantDef; 2] = [
    ButtonStyleVariantDef { label: "Primary", description: "High emphasis actions", style: ShadcnButtonStyle::Primary },
    ButtonStyleVariantDef {
        label: "Secondary",
        description: "Lower emphasis actions",
        style: ShadcnButtonStyle::Secondary,
    },
];

const BUTTON_TEMPLATE_VARIANTS: [ButtonTemplateVariant; 3] = [
    ButtonTemplateVariant::TextButton,
    ButtonTemplateVariant::TextButtonLeadingIcon,
    ButtonTemplateVariant::TextButtonTrailingIcon,
];

pub(in crate::studio::style::style_guide) fn render_button_template_matrix_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<TabsNavigation>,
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
    let variants = BUTTON_TEMPLATE_VARIANTS;
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));

    section_shell_with_width(
        960.0,
        "Buttons",
        "State and variant matrix.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        gpui::hsla(0.0, 0.0, 0.0, 0.0),
        render_button_preview_tabbed_content(
            look.as_ref(),
            &template,
            &variants,
            &samples,
            preview_tabs,
            active_tab,
            chrome.border,
            window,
            cx,
        ),
    )
}

fn render_button_preview_tabbed_content(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    variants: &[ButtonTemplateVariant],
    samples: &[ButtonStateSample],
    preview_tabs: Entity<TabsNavigation>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = if active_tab.as_ref() == "sizes" {
        render_button_size_matrix(look, template, window, cx)
    } else {
        render_button_template_matrix(look, template, variants, samples, window, cx)
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .child(div().w_full().flex().justify_start().child(preview_tabs))
        .child(div().w_full().h(px(1.0)).bg(border))
        .child(div().w_full().flex().justify_center().mt(px(16.0)).child(body))
        .into_any_element()
}

fn render_button_template_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    variants: &[ButtonTemplateVariant],
    samples: &[ButtonStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    super::super::variant_state_table::VariantStateTable::new(
        super::super::variant_state_table::VariantStateTableStyle::from_chrome(&chrome)
            .state_column_width(BUTTON_TABLE_STATE_COLUMN_WIDTH)
            .row_height(BUTTON_TABLE_ROW_HEIGHT),
    )
    .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
    .rows(BUTTON_STYLE_VARIANTS.iter().map(|row| {
        super::super::variant_state_table::VariantStateTableRow {
            label: SharedString::from(row.label),
            description: SharedString::from(row.description),
            cells: samples
                .iter()
                .map(|sample| {
                    render_button_state_variants_cell(template, look, row.style, variants, sample, window, cx)
                })
                .collect(),
        }
    }))
    .build()
}

fn render_button_state_variants_cell(
    template: &Arc<dyn ButtonTemplate<()>>,
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
    variants: &[ButtonTemplateVariant],
    sample: &ButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .w_full()
        .h_full()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(BUTTON_TABLE_VARIANT_GAP))
        .px(px(8.0))
        .py(px(8.0))
        .children(
            variants
                .iter()
                .map(|variant| render_button_state_sample(template, look, style, *variant, sample, window, cx)),
        )
        .into_any_element()
}

pub(in crate::studio::style::style_guide) fn render_icon_button_template_matrix_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<TabsNavigation>,
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
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));

    section_shell_with_width(
        960.0,
        "Icon Button",
        "Icon-only button states across style variants.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        gpui::hsla(0.0, 0.0, 0.0, 0.0),
        render_icon_button_preview_tabbed_content(
            look.as_ref(),
            &template,
            &samples,
            preview_tabs,
            active_tab,
            chrome.border,
            window,
            cx,
        ),
    )
}

fn render_icon_button_preview_tabbed_content(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    samples: &[ButtonStateSample],
    preview_tabs: Entity<TabsNavigation>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = if active_tab.as_ref() == "sizes" {
        render_icon_button_size_matrix(look, template, window, cx)
    } else {
        render_icon_button_matrix(look, template, samples, window, cx)
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .child(div().w_full().flex().justify_start().child(preview_tabs))
        .child(div().w_full().h(px(1.0)).bg(border))
        .child(div().w_full().flex().justify_center().mt(px(16.0)).child(body))
        .into_any_element()
}

struct IconButtonVariantDef {
    label: &'static str,
    description: &'static str,
    style: ShadcnButtonStyle,
}

const ICON_BUTTON_VARIANTS: [IconButtonVariantDef; 4] = [
    IconButtonVariantDef { label: "Primary", description: "High emphasis actions", style: ShadcnButtonStyle::Primary },
    IconButtonVariantDef {
        label: "Secondary",
        description: "Lower emphasis actions",
        style: ShadcnButtonStyle::Secondary,
    },
    IconButtonVariantDef {
        label: "Outline",
        description: "Subtle secondary controls",
        style: ShadcnButtonStyle::Outline,
    },
    IconButtonVariantDef { label: "Ghost", description: "Quiet utility actions", style: ShadcnButtonStyle::Ghost },
];

fn render_icon_button_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    samples: &[ButtonStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let variant = ButtonTemplateVariant::IconButton;
    let chrome = look.chrome();

    super::super::variant_state_table::VariantStateTable::new(
        super::super::variant_state_table::VariantStateTableStyle::from_chrome(&chrome),
    )
    .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
    .rows(ICON_BUTTON_VARIANTS.iter().map(|row| {
        super::super::variant_state_table::VariantStateTableRow {
            label: SharedString::from(row.label),
            description: SharedString::from(row.description),
            cells: samples
                .iter()
                .map(|sample| render_button_state_sample(template, look, row.style, variant, sample, window, cx))
                .collect(),
        }
    }))
    .build()
}

fn render_icon_button_size_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_button_size_radius_matrix(look, template, SIZE_PREVIEW_STYLE, true, window, cx)
}

fn render_button_size_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_button_size_radius_matrix(look, template, SIZE_PREVIEW_STYLE, false, window, cx)
}

fn render_button_size_radius_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    style: ShadcnButtonStyle,
    icon_only: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    super::super::variant_state_table::VariantStateTable::new(
        super::super::variant_state_table::VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(BUTTON_TABLE_SIZE_VARIANT_COLUMN_WIDTH)
            .state_column_width(BUTTON_TABLE_RADIUS_COLUMN_WIDTH)
            .header_height(BUTTON_TABLE_SIZE_HEADER_HEIGHT)
            .row_height(BUTTON_TABLE_SIZE_RADIUS_ROW_HEIGHT)
            .variant_column_align_center(true),
    )
    .row_group_label("SIZE")
    .column_headers(
        ButtonRadiusPreset::ALL
            .iter()
            .map(|preset| render_button_radius_header_cell(preset.label(), chrome.muted_text)),
    )
    .rows(BUTTON_SIZES.iter().map(|(size, label)| {
        super::super::variant_state_table::VariantStateTableRow {
            label: SharedString::from(*label),
            description: SharedString::from(""),
            cells: ButtonRadiusPreset::ALL
                .iter()
                .map(|radius| {
                    render_button_size_radius_cell(template, look, style, *size, *radius, icon_only, window, cx)
                })
                .collect(),
        }
    }))
    .build()
}

fn render_button_radius_header_cell(label: &'static str, muted_text: gpui::Hsla) -> AnyElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .line_height(px(15.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(muted_text)
        .child(label)
        .into_any_element()
}

fn render_button_size_radius_cell(
    template: &Arc<dyn ButtonTemplate<()>>,
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
    size: ButtonSize,
    radius: ButtonRadiusPreset,
    icon_only: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!(
        "theme-studio-button-size-radius-preview-{}-{}-{}-{}",
        shadcn_style_id(style),
        button_size_id(size),
        radius_label_id(radius),
        if icon_only { "icon" } else { "text" }
    ));
    let content: gpui_luma::controls::command::button::ControlPresenter<ButtonRenderModel<()>> = if icon_only {
        Arc::new(move |model, _| {
            let icon_size = button_preview_look(model).map(|look| look.icon_size).unwrap_or(16.0);
            render_lucide_icon(LucideIcon::Heart, icon_size)
        })
    } else {
        let label = SharedString::from("Next");
        Arc::new(move |model, _| {
            let look = button_preview_look(model);
            let gap = look.as_ref().map(|look| look.gap).unwrap_or(6.0);
            let icon_size = look.as_ref().map(|look| look.icon_size).unwrap_or(16.0);
            div()
                .flex()
                .items_center()
                .gap(px(gap))
                .child(label.clone())
                .child(render_lucide_icon(LucideIcon::ChevronRight, icon_size))
                .into_any_element()
        })
    };
    let model = ButtonRenderModel {
        id,
        data: (),
        content,
        role: if icon_only {
            ButtonFamilyRole::Icon
        } else {
            ButtonFamilyRole::Text
        },
        size,
        state: InteractionState::default(),
        round: false,
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: Some(button_look_for_semantic(Arc::new(look.clone()), style, size, radius)),
    };

    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .child(template.render(&model, window, cx))
        .into_any_element()
}

fn button_preview_look(model: &ButtonRenderModel<()>) -> Option<gpui_luma::controls::button_family::ButtonFamilyLook> {
    model.look.as_ref().map(|resolve| resolve(model))
}

fn radius_label_id(radius: ButtonRadiusPreset) -> &'static str {
    match radius {
        ButtonRadiusPreset::None => "none",
        ButtonRadiusPreset::Small => "small",
        ButtonRadiusPreset::Medium => "medium",
        ButtonRadiusPreset::Large => "large",
        ButtonRadiusPreset::Full => "full",
    }
}

fn render_icon_button_state_header_cell(sample: &ButtonStateSample, muted_text: gpui::Hsla) -> AnyElement {
    render_interaction_state_header_cell(sample.id, sample.header, muted_text)
}

fn render_interaction_state_header_cell(id: &'static str, header: &'static str, muted_text: gpui::Hsla) -> AnyElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .child(div().text_color(muted_text).child(render_lucide_icon(icon_button_state_header_icon(id), 16.0)))
        .child(
            div()
                .text_xs()
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted_text)
                .child(icon_button_state_display_label(header)),
        )
        .into_any_element()
}

fn icon_button_state_header_icon(state_id: &'static str) -> LucideIcon {
    match state_id {
        "default" => LucideIcon::House,
        "hover" => LucideIcon::MousePointer2,
        "focused" => LucideIcon::SquareDashed,
        "pressed" => LucideIcon::ArrowDown,
        "disabled" => LucideIcon::CircleMinus,
        "disabled-pressed" => LucideIcon::CircleMinus,
        _ => LucideIcon::House,
    }
}

fn icon_button_state_display_label(header: &'static str) -> &'static str {
    match header {
        "default" => "Default",
        "hover" => "Hover",
        "focused" => "Focused",
        "pressed" => "Pressed",
        "disabled" => "Disabled",
        "disabled-pressed" => "Disabled · On",
        _ => header,
    }
}

pub(in crate::studio::style::style_guide) fn render_checkbox_template_matrix_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<TabsNavigation>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_choice_control_template_matrix_section(
        look,
        ChoiceTemplateControl::Checkbox,
        "Checkbox",
        "Style variants by row. Pressed and Disabled · On columns show checked.",
        preview_tabs,
        window,
        cx,
    )
}

pub(in crate::studio::style::style_guide) fn render_radio_template_matrix_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<TabsNavigation>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_choice_control_template_matrix_section(
        look,
        ChoiceTemplateControl::Radio,
        "Radio",
        "Style variants by row. Pressed and Disabled · On columns show selected.",
        preview_tabs,
        window,
        cx,
    )
}

pub(in crate::studio::style::style_guide) fn render_switch_template_matrix_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<TabsNavigation>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_choice_control_template_matrix_section(
        look,
        ChoiceTemplateControl::Switch,
        "Switch",
        "Style variants by row. Pressed and Disabled · On columns show on.",
        preview_tabs,
        window,
        cx,
    )
}

fn render_choice_control_template_matrix_section(
    look: Arc<ShadcnLook>,
    control: ChoiceTemplateControl,
    title: &'static str,
    description: &'static str,
    preview_tabs: Entity<TabsNavigation>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));

    section_shell_with_width(
        960.0,
        title,
        description,
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        gpui::hsla(0.0, 0.0, 0.0, 0.0),
        render_choice_preview_tabbed_content(look, control, preview_tabs, active_tab, chrome.border, window, cx),
    )
}

fn render_choice_preview_tabbed_content(
    look: Arc<ShadcnLook>,
    control: ChoiceTemplateControl,
    preview_tabs: Entity<TabsNavigation>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = if active_tab.as_ref() == "sizes" {
        render_choice_size_matrix(look.clone(), control, window, cx)
    } else {
        render_choice_variant_state_matrix(look.clone(), control, &toggle_interaction_state_samples(), window, cx)
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .child(div().w_full().flex().justify_start().child(preview_tabs))
        .child(div().w_full().h(px(1.0)).bg(border))
        .child(div().w_full().flex().justify_center().mt(px(16.0)).child(body))
        .into_any_element()
}

fn render_choice_size_matrix(
    look: Arc<ShadcnLook>,
    control: ChoiceTemplateControl,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    super::super::variant_state_table::VariantStateTable::new(
        super::super::variant_state_table::VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(BUTTON_TABLE_SIZE_VARIANT_COLUMN_WIDTH)
            .state_column_width(BUTTON_TABLE_RADIUS_COLUMN_WIDTH)
            .header_height(BUTTON_TABLE_SIZE_HEADER_HEIGHT)
            .row_height(BUTTON_TABLE_SIZE_RADIUS_ROW_HEIGHT)
            .variant_column_align_center(true),
    )
    .row_group_label("SIZE")
    .column_headers(
        CHOICE_STYLE_VARIANTS
            .iter()
            .map(|variant| render_button_radius_header_cell(variant.label, chrome.muted_text)),
    )
    .rows(BUTTON_SIZES.iter().map(|(size, label)| {
        super::super::variant_state_table::VariantStateTableRow {
            label: SharedString::from(*label),
            description: SharedString::from(""),
            cells: CHOICE_STYLE_VARIANTS
                .iter()
                .map(|variant| render_choice_size_cell(&look, control, variant.style, *size, window, cx))
                .collect(),
        }
    }))
    .build()
}

fn render_choice_size_cell(
    look: &Arc<ShadcnLook>,
    control: ChoiceTemplateControl,
    style: ShadcnButtonStyle,
    size: ButtonSize,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let active = true;
    let id = SharedString::from(format!(
        "theme-studio-choice-size-preview-{}-{}-{}",
        control.id(),
        shadcn_style_id(style),
        button_size_id(size),
    ));
    let model = ButtonRenderModel {
        id,
        data: active,
        content: control.content(active),
        role: control.role(),
        size,
        state: InteractionState::default(),
        round: false,
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: None,
    };

    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .child(choice_template_for_control(look, control, style).render(&model, window, cx))
        .into_any_element()
}

fn render_choice_variant_state_matrix(
    look: Arc<ShadcnLook>,
    control: ChoiceTemplateControl,
    samples: &[ButtonStateSample; 6],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    super::super::variant_state_table::VariantStateTable::new(
        super::super::variant_state_table::VariantStateTableStyle::from_chrome(&chrome),
    )
    .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
    .rows(CHOICE_STYLE_VARIANTS.iter().map(|row| {
        super::super::variant_state_table::VariantStateTableRow {
            label: SharedString::from(row.label),
            description: SharedString::from(row.description),
            cells: samples
                .iter()
                .map(|sample| render_choice_variant_state_cell(&look, control, row.style, sample, window, cx))
                .collect(),
        }
    }))
    .build()
}

fn choice_active_for_sample(sample: &ButtonStateSample) -> bool {
    matches!(sample.id, "pressed" | "disabled-pressed")
}

fn render_choice_variant_state_cell(
    look: &Arc<ShadcnLook>,
    control: ChoiceTemplateControl,
    style: ShadcnButtonStyle,
    sample: &ButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let active = choice_active_for_sample(sample);
    let id = SharedString::from(format!(
        "theme-studio-choice-preview-{}-{}-{}",
        control.id(),
        shadcn_style_id(style),
        sample.id
    ));
    let model = ButtonRenderModel {
        id,
        data: active,
        content: control.content(active),
        role: control.role(),
        size: ButtonSize::Md,
        state: sample.state,
        round: false,
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: None,
    };

    div()
        .w_full()
        .flex()
        .justify_center()
        .items_center()
        .child(choice_template_for_control(look, control, style).render(&model, window, cx))
        .into_any_element()
}

fn choice_template_for_control(
    look: &Arc<ShadcnLook>,
    control: ChoiceTemplateControl,
    style: ShadcnButtonStyle,
) -> Arc<dyn ButtonTemplate<bool>> {
    match control {
        ChoiceTemplateControl::Radio => look.radio_button_template(style),
        ChoiceTemplateControl::Checkbox => look.checkbox_template(style),
        ChoiceTemplateControl::Switch => look.switch_template(style),
    }
}

pub(in crate::studio::style::style_guide) fn render_toggle_template_matrix_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<TabsNavigation>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let template = look.toggle_template(ShadcnButtonStyle::Secondary);
    let samples = toggle_interaction_state_samples();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));

    section_shell_with_width(
        960.0,
        "Toggles",
        "Selected toggles per style variant. Pressed and Disabled · On columns show toggled on.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        gpui::hsla(0.0, 0.0, 0.0, 0.0),
        render_toggle_preview_tabbed_content(
            look.as_ref(),
            &template,
            &samples,
            preview_tabs,
            active_tab,
            chrome.border,
            window,
            cx,
        ),
    )
}

fn toggle_interaction_state_samples() -> [ButtonStateSample; 6] {
    [
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
        ButtonStateSample {
            id: "disabled-pressed",
            header: "disabled-pressed",
            state: InteractionState { disabled: true, pressed: true, ..InteractionState::default() },
        },
    ]
}

fn render_toggle_preview_tabbed_content(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<bool>>,
    samples: &[ButtonStateSample],
    preview_tabs: Entity<TabsNavigation>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = if active_tab.as_ref() == "sizes" {
        render_toggle_sizes_preview(look, template, window, cx)
    } else {
        render_toggle_template_preview(look, template, samples, window, cx)
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .child(div().w_full().flex().justify_start().child(preview_tabs))
        .child(div().w_full().h(px(1.0)).bg(border))
        .child(div().w_full().flex().justify_center().mt(px(16.0)).child(body))
        .into_any_element()
}

fn render_toggle_template_preview(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<bool>>,
    samples: &[ButtonStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(20.0))
        .children([
            render_toggle_text_template_matrix(look, template, samples, window, cx),
            render_toggle_icon_template_matrix(look, template, samples, window, cx),
        ])
        .into_any_element()
}

fn render_toggle_text_template_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<bool>>,
    samples: &[ButtonStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    super::super::variant_state_table::VariantStateTable::new(
        super::super::variant_state_table::VariantStateTableStyle::from_chrome(&chrome)
            .state_column_width(TOGGLE_TEXT_TABLE_STATE_COLUMN_WIDTH),
    )
    .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
    .rows(BUTTON_STYLE_VARIANTS.iter().map(|row| {
        super::super::variant_state_table::VariantStateTableRow {
            label: SharedString::from(row.label),
            description: SharedString::from(row.description),
            cells: samples
                .iter()
                .map(|sample| render_toggle_state_sample(template, look, row.style, false, sample, window, cx))
                .collect(),
        }
    }))
    .build()
}

fn render_toggle_icon_template_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<bool>>,
    samples: &[ButtonStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    super::super::variant_state_table::VariantStateTable::new(
        super::super::variant_state_table::VariantStateTableStyle::from_chrome(&chrome),
    )
    .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
    .rows(ICON_BUTTON_VARIANTS.iter().map(|row| {
        super::super::variant_state_table::VariantStateTableRow {
            label: SharedString::from(row.label),
            description: SharedString::from(row.description),
            cells: samples
                .iter()
                .map(|sample| render_toggle_state_sample(template, look, row.style, true, sample, window, cx))
                .collect(),
        }
    }))
    .build()
}

fn render_toggle_sizes_preview(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<bool>>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(20.0))
        .children([
            render_toggle_text_size_matrix(look, template, window, cx),
            render_toggle_icon_size_matrix(look, template, window, cx),
        ])
        .into_any_element()
}

fn render_toggle_text_size_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<bool>>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_toggle_size_radius_matrix(look, template, SIZE_PREVIEW_STYLE, false, window, cx)
}

fn render_toggle_icon_size_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<bool>>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_toggle_size_radius_matrix(look, template, SIZE_PREVIEW_STYLE, true, window, cx)
}

fn render_toggle_size_radius_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<bool>>,
    style: ShadcnButtonStyle,
    icon_only: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    super::super::variant_state_table::VariantStateTable::new(
        super::super::variant_state_table::VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(BUTTON_TABLE_SIZE_VARIANT_COLUMN_WIDTH)
            .state_column_width(BUTTON_TABLE_RADIUS_COLUMN_WIDTH)
            .header_height(BUTTON_TABLE_SIZE_HEADER_HEIGHT)
            .row_height(BUTTON_TABLE_SIZE_RADIUS_ROW_HEIGHT)
            .variant_column_align_center(true),
    )
    .row_group_label("SIZE")
    .column_headers(
        ButtonRadiusPreset::ALL
            .iter()
            .map(|preset| render_button_radius_header_cell(preset.label(), chrome.muted_text)),
    )
    .rows(BUTTON_SIZES.iter().map(|(size, label)| {
        super::super::variant_state_table::VariantStateTableRow {
            label: SharedString::from(*label),
            description: SharedString::from(""),
            cells: ButtonRadiusPreset::ALL
                .iter()
                .map(|radius| {
                    render_toggle_size_radius_cell(template, look, style, *size, *radius, icon_only, window, cx)
                })
                .collect(),
        }
    }))
    .build()
}

fn render_toggle_size_radius_cell(
    template: &Arc<dyn ButtonTemplate<bool>>,
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
    size: ButtonSize,
    radius: ButtonRadiusPreset,
    icon_only: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selected = true;
    let id = SharedString::from(format!(
        "theme-studio-toggle-size-radius-preview-{}-{}-{}-{}",
        shadcn_style_id(style),
        button_size_id(size),
        radius_label_id(radius),
        if icon_only { "icon" } else { "text" }
    ));
    let content: gpui_luma::controls::command::button::ControlPresenter<ButtonRenderModel<bool>> = if icon_only {
        Arc::new(move |model, _| {
            let icon_size = button_preview_look_bool(model).map(|look| look.icon_size).unwrap_or(16.0);
            render_lucide_icon(LucideIcon::Heart, icon_size)
        })
    } else {
        let label = SharedString::from("Toggle");
        Arc::new(move |_, _| div().child(label.clone()).into_any_element())
    };
    let look_source = if icon_only {
        toggle_icon_look_for_semantic(Arc::new(look.clone()), style, size, radius, selected)
    } else {
        toggle_look_for_semantic(Arc::new(look.clone()), style, size, radius, selected)
    };
    let model = ButtonRenderModel {
        id,
        data: selected,
        content,
        role: ButtonFamilyRole::Toggle { selected },
        size,
        state: InteractionState::default(),
        round: false,
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: Some(look_source),
    };

    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .child(template.render(&model, window, cx))
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

fn toggle_selected_for_sample(sample: &ButtonStateSample) -> bool {
    matches!(sample.id, "pressed" | "disabled-pressed")
}

fn render_toggle_state_sample(
    template: &Arc<dyn ButtonTemplate<bool>>,
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
    icon_only: bool,
    sample: &ButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selected = toggle_selected_for_sample(sample);
    let id = SharedString::from(format!(
        "theme-studio-toggle-preview-{}-{}-{}",
        shadcn_style_id(style),
        if icon_only { "icon" } else { "text" },
        sample.id
    ));
    let content: gpui_luma::controls::command::button::ControlPresenter<ButtonRenderModel<bool>> = if icon_only {
        Arc::new(move |model, _| round_icon_glyph(model, selected))
    } else {
        let label = SharedString::from("Toggle");
        Arc::new(move |_, _| div().child(label.clone()).into_any_element())
    };
    let model = ButtonRenderModel {
        id,
        data: selected,
        content,
        role: ButtonFamilyRole::Toggle { selected },
        size: ButtonSize::Md,
        state: sample.state,
        round: icon_only,
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: Some(if icon_only {
            toggle_icon_look_for_style(Arc::new(look.clone()), style)
        } else {
            toggle_button_look_for_style(Arc::new(look.clone()), style)
        }),
    };

    div()
        .w_full()
        .flex()
        .justify_center()
        .items_center()
        .child(template.render(&model, window, cx))
        .into_any_element()
}

fn toggle_icon_look_for_style(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> gpui_luma::controls::command::button::ButtonLookSource<bool> {
    Arc::new(move |model| {
        let tokens = theme.mode_tokens();
        gpui_luma_look_shadcn::paint::toggle_icon_look_semantic(
            tokens.as_ref(),
            theme.mode(),
            style,
            model.data,
            model.size,
            Some(ButtonRadiusPreset::Full),
            model.state,
        )
    })
}

fn toggle_icon_look_for_semantic(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ButtonSize,
    radius: ButtonRadiusPreset,
    selected: bool,
) -> gpui_luma::controls::command::button::ButtonLookSource<bool> {
    Arc::new(move |model| {
        let tokens = theme.mode_tokens();
        gpui_luma_look_shadcn::paint::toggle_icon_look_semantic(
            tokens.as_ref(),
            theme.mode(),
            style,
            selected,
            size,
            Some(radius),
            model.state,
        )
    })
}

fn toggle_look_for_semantic(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ButtonSize,
    radius: ButtonRadiusPreset,
    selected: bool,
) -> gpui_luma::controls::command::button::ButtonLookSource<bool> {
    Arc::new(move |model| {
        let tokens = theme.mode_tokens();
        gpui_luma_look_shadcn::paint::toggle_look_semantic(
            tokens.as_ref(),
            theme.mode(),
            style,
            selected,
            size,
            Some(radius),
            model.state,
        )
    })
}

fn button_preview_look_bool(
    model: &ButtonRenderModel<bool>,
) -> Option<gpui_luma::controls::button_family::ButtonFamilyLook> {
    model.look.as_ref().map(|resolve| resolve(model))
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

fn button_look_for_semantic(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ButtonSize,
    radius: ButtonRadiusPreset,
) -> gpui_luma::controls::command::button::ButtonLookSource<()> {
    Arc::new(move |model| {
        let tokens = theme.mode_tokens();
        gpui_luma_look_shadcn::paint::button_look_semantic(
            tokens.as_ref(),
            theme.mode(),
            style,
            model.role,
            size,
            Some(radius),
            model.state,
        )
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

fn button_size_id(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::Sm => "sm",
        ButtonSize::Md => "md",
        ButtonSize::Lg => "lg",
    }
}

pub(in crate::studio::style::style_guide) fn render_lucide_icon(icon: LucideIcon, size: f32) -> AnyElement {
    div()
        .font_family("lucide")
        .text_size(px(size))
        .line_height(px(size))
        .child(char::from(icon).to_string())
        .into_any_element()
}

pub(in crate::studio::style::style_guide) fn round_icon_glyph(
    model: &ButtonRenderModel<bool>,
    selected: bool,
) -> AnyElement {
    let icon = if selected { LucideIcon::Check } else { LucideIcon::Plus };
    let icon_size = model.look.as_ref().map(|resolve| resolve(model).icon_size).unwrap_or(16.0);
    render_lucide_icon(icon, icon_size)
}
