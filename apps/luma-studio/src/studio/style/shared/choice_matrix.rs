use std::sync::Arc;

use gpui::{AnyElement, App, Entity, IntoElement, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::button_family::ButtonSize;
use gpui_luma::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use gpui_luma::controls::tabs_navigation::TabsNavigation;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn::{ButtonRadiusPreset, ShadcnButtonStyle, ShadcnLook};

use crate::studio::style::shared::button_matrix::{
    BUTTON_SIZES, BUTTON_TABLE_RADIUS_COLUMN_WIDTH, BUTTON_TABLE_SIZE_HEADER_HEIGHT,
    BUTTON_TABLE_SIZE_RADIUS_ROW_HEIGHT, BUTTON_TABLE_SIZE_VARIANT_COLUMN_WIDTH, CHOICE_CONTROL_STYLE_VARIANTS,
    SIZE_PREVIEW_STYLE, button_size_id, radius_label_id, render_button_radius_header_cell,
    render_icon_button_state_header_cell, shadcn_style_id, toggle_interaction_state_samples,
};
use crate::studio::style::shared::samples::ButtonStateSample;
use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

#[derive(Clone, Copy)]
pub(crate) enum ChoiceTemplateControl {
    Radio,
    Checkbox,
    Switch,
}

impl ChoiceTemplateControl {
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Radio => "radio",
            Self::Checkbox => "checkbox",
            Self::Switch => "switch",
        }
    }

    pub(crate) fn content(
        self,
        _active: bool,
    ) -> gpui_luma::controls::command::button::ControlPresenter<ButtonRenderModel<bool>> {
        Arc::new(move |_, _| div().into_any_element())
    }

    pub(crate) fn role(self) -> gpui_luma::controls::button_family::ButtonFamilyRole {
        match self {
            Self::Checkbox | Self::Radio => gpui_luma::controls::button_family::ButtonFamilyRole::Icon,
            Self::Switch => gpui_luma::controls::button_family::ButtonFamilyRole::Text,
        }
    }
}

pub(crate) fn render_choice_control_template_matrix_section(
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
    if matches!(control, ChoiceTemplateControl::Switch) {
        return render_switch_size_radius_matrix(look, window, cx);
    }

    let chrome = look.chrome();

    VariantStateTable::new(
        VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(BUTTON_TABLE_SIZE_VARIANT_COLUMN_WIDTH)
            .state_column_width(BUTTON_TABLE_RADIUS_COLUMN_WIDTH)
            .header_height(BUTTON_TABLE_SIZE_HEADER_HEIGHT)
            .row_height(BUTTON_TABLE_SIZE_RADIUS_ROW_HEIGHT),
    )
    .row_group_label("SIZE")
    .column_headers(
        CHOICE_CONTROL_STYLE_VARIANTS
            .iter()
            .map(|variant| render_button_radius_header_cell(variant.label, chrome.muted_text)),
    )
    .rows(BUTTON_SIZES.iter().map(|(size, label)| {
        VariantStateTableRow {
            label: SharedString::from(*label),
            description: SharedString::from(""),
            cells: CHOICE_CONTROL_STYLE_VARIANTS
                .iter()
                .map(|variant| render_choice_size_cell(&look, control, variant.style, *size, None, window, cx))
                .collect(),
        }
    }))
    .build()
}

fn render_switch_size_radius_matrix(look: Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let style = SIZE_PREVIEW_STYLE;

    VariantStateTable::new(
        VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(BUTTON_TABLE_SIZE_VARIANT_COLUMN_WIDTH)
            .state_column_width(BUTTON_TABLE_RADIUS_COLUMN_WIDTH)
            .header_height(BUTTON_TABLE_SIZE_HEADER_HEIGHT)
            .row_height(BUTTON_TABLE_SIZE_RADIUS_ROW_HEIGHT),
    )
    .row_group_label("SIZE")
    .column_headers(
        ButtonRadiusPreset::ALL
            .iter()
            .map(|preset| render_button_radius_header_cell(preset.label(), chrome.muted_text)),
    )
    .rows(BUTTON_SIZES.iter().map(|(size, label)| {
        VariantStateTableRow {
            label: SharedString::from(*label),
            description: SharedString::from(""),
            cells: ButtonRadiusPreset::ALL
                .iter()
                .map(|radius| {
                    render_choice_size_cell(
                        &look,
                        ChoiceTemplateControl::Switch,
                        style,
                        *size,
                        Some(*radius),
                        window,
                        cx,
                    )
                })
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
    radius: Option<ButtonRadiusPreset>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let active = true;
    let id = SharedString::from(format!(
        "luma-studio-choice-size-preview-{}-{}-{}-{}",
        control.id(),
        shadcn_style_id(style),
        button_size_id(size),
        radius.map(radius_label_id).unwrap_or("default"),
    ));
    let radius_override = radius.map(|preset| {
        let tokens = look.mode_tokens();
        let scale = gpui_luma_look_shadcn::paint::switch_scale(tokens.as_ref(), look.mode(), style, size, 1.0);
        gpui_luma_look_shadcn::paint::resolve_switch_radius_preset(preset, &tokens.metrics, scale.track_height)
    });
    let model = ButtonRenderModel {
        id,
        data: active,
        content: control.content(active),
        role: control.role(),
        size,
        state: InteractionState::default(),
        round: false,
        radius_override: std::cell::Cell::new(radius_override),
        elevation: style != ShadcnButtonStyle::ContentOnly,
        compact: false,
        look: None,
        ..Default::default()
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

pub(crate) fn render_choice_control_template_preview_body(
    look: Arc<ShadcnLook>,
    control: ChoiceTemplateControl,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_choice_variant_state_matrix(look, control, &toggle_interaction_state_samples(), window, cx)
}

pub(crate) fn render_choice_control_sizes_body(
    look: Arc<ShadcnLook>,
    control: ChoiceTemplateControl,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_choice_size_matrix(look, control, window, cx)
}

fn render_choice_variant_state_matrix(
    look: Arc<ShadcnLook>,
    control: ChoiceTemplateControl,
    samples: &[ButtonStateSample; 6],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    VariantStateTable::new(VariantStateTableStyle::from_chrome(&chrome))
        .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
        .rows(CHOICE_CONTROL_STYLE_VARIANTS.iter().map(|row| {
            VariantStateTableRow {
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
        "luma-studio-choice-preview-{}-{}-{}",
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
        elevation: style != ShadcnButtonStyle::ContentOnly,
        compact: false,
        look: None,
        ..Default::default()
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
