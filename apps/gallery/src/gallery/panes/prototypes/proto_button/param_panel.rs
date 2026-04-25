use gpui::{AnyElement, FontWeight, div, prelude::*, px};
use gpui_luma::controls::prototypes::proto_button::{
    ProtoButtonTemplateParamField, ProtoButtonTemplateParamType, ProtoButtonTemplateParamUsage,
    ProtoButtonTemplateUsage,
};

use crate::gallery::theme::{GalleryChrome, GalleryThemePack};

pub(in crate::gallery) fn render_proto_button_param_panel(
    usage: &ProtoButtonTemplateUsage,
    chrome: GalleryChrome,
    theme: &GalleryThemePack,
    selected_visual_state_label: &str,
) -> AnyElement {
    div()
        .id("proto-button-template-usage")
        .w(px(430.0))
        .max_h(px(560.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .overflow_y_scroll()
        .border_1()
        .border_color(chrome.border)
        .rounded(px(6.0))
        .bg(chrome.panel_background)
        .p(px(12.0))
        .child(
            div()
                .text_size(px(13.0))
                .line_height(px(18.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.title_text)
                .child(format!("Template Parameters · state: {}", selected_visual_state_label)),
        )
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(chrome.muted_text).child(usage.component))
        .children(usage.parameters.iter().map(|parameter| render_template_usage_param(parameter, theme)))
        .into_any_element()
}

fn render_template_usage_param(parameter: &ProtoButtonTemplateParamUsage, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .border_1()
        .border_color(chrome.border)
        .rounded(px(5.0))
        .p(px(8.0))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .child(
                    div()
                        .min_w(px(0.0))
                        .flex_1()
                        .truncate()
                        .text_size(px(12.0))
                        .line_height(px(17.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.title_text)
                        .child(parameter.name),
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .line_height(px(14.0))
                        .font_family("Monaco")
                        .text_color(chrome.muted_text)
                        .child(param_type_label(parameter.param_type)),
                ),
        )
        .child(
            div()
                .text_size(px(11.0))
                .line_height(px(15.0))
                .text_color(chrome.body_text)
                .child(parameter.description),
        )
        .child(
            div()
                .font_family("Monaco")
                .text_size(px(10.0))
                .line_height(px(14.0))
                .text_color(chrome.muted_text)
                .child(format!("states: {}", parameter.states.join(", "))),
        )
        .child(
            div()
                .font_family("Monaco")
                .text_size(px(10.0))
                .line_height(px(14.0))
                .text_color(chrome.muted_text)
                .child(format!("fields: {}", format_param_fields(parameter.param_fields))),
        )
        .child(
            div()
                .font_family("Monaco")
                .text_size(px(10.0))
                .line_height(px(14.0))
                .text_color(chrome.muted_text)
                .child(format!("default: {}", parameter.default_source)),
        )
        .into_any_element()
}

fn format_param_fields(fields: &[ProtoButtonTemplateParamField]) -> String {
    let field_names: Vec<String> = fields
        .iter()
        .map(|field| match field {
            ProtoButtonTemplateParamField::Variant => "variant".to_string(),
            ProtoButtonTemplateParamField::Size => "size".to_string(),
            ProtoButtonTemplateParamField::DisabledOpacity => "disabled_opacity".to_string(),
            ProtoButtonTemplateParamField::PointerCursorWhenEnabled => "pointer_cursor_when_enabled".to_string(),
            ProtoButtonTemplateParamField::BackgroundBase => "background.base".to_string(),
            ProtoButtonTemplateParamField::BackgroundHovered => "background.hovered".to_string(),
            ProtoButtonTemplateParamField::BackgroundPressed => "background.pressed".to_string(),
            ProtoButtonTemplateParamField::BackgroundFocused => "background.focused".to_string(),
            ProtoButtonTemplateParamField::BackgroundDisabled => "background.disabled".to_string(),
            ProtoButtonTemplateParamField::ForegroundBase => "foreground.base".to_string(),
            ProtoButtonTemplateParamField::ForegroundHovered => "foreground.hovered".to_string(),
            ProtoButtonTemplateParamField::ForegroundPressed => "foreground.pressed".to_string(),
            ProtoButtonTemplateParamField::ForegroundFocused => "foreground.focused".to_string(),
            ProtoButtonTemplateParamField::ForegroundDisabled => "foreground.disabled".to_string(),
            ProtoButtonTemplateParamField::BorderBase => "border.base".to_string(),
            ProtoButtonTemplateParamField::BorderHovered => "border.hovered".to_string(),
            ProtoButtonTemplateParamField::BorderPressed => "border.pressed".to_string(),
            ProtoButtonTemplateParamField::BorderFocused => "border.focused".to_string(),
            ProtoButtonTemplateParamField::BorderDisabled => "border.disabled".to_string(),
            ProtoButtonTemplateParamField::FocusRing => "focus_ring".to_string(),
            ProtoButtonTemplateParamField::Radius => "radius".to_string(),
            ProtoButtonTemplateParamField::PaddingX => "padding_x".to_string(),
            ProtoButtonTemplateParamField::PaddingY => "padding_y".to_string(),
            ProtoButtonTemplateParamField::Gap => "gap".to_string(),
            ProtoButtonTemplateParamField::Height => "height".to_string(),
            ProtoButtonTemplateParamField::TypographySize => "typography_size".to_string(),
            ProtoButtonTemplateParamField::TypographyLineHeight => "typography_line_height".to_string(),
            ProtoButtonTemplateParamField::TypographyWeight => "typography_weight".to_string(),
        })
        .collect();
    field_names.join(", ")
}

fn param_type_label(param_type: ProtoButtonTemplateParamType) -> &'static str {
    match param_type {
        ProtoButtonTemplateParamType::Color => "Color",
        ProtoButtonTemplateParamType::Pixels => "Pixels",
        ProtoButtonTemplateParamType::Number => "Number",
        ProtoButtonTemplateParamType::Bool => "Bool",
        ProtoButtonTemplateParamType::Enum => "Enum",
    }
}
