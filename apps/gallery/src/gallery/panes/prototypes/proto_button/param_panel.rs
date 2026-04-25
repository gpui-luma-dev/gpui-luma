use gpui::{AnyElement, FontWeight, div, prelude::*, px};
use gpui_luma::controls::prototypes::proto_button::{
    ProtoButtonTemplateParamType, ProtoButtonTemplateParamUsage, ProtoButtonTemplateUsage,
};

use crate::gallery::theme::{GalleryChrome, GalleryThemePack};

pub(in crate::gallery) fn render_proto_button_param_panel(
    usage: &ProtoButtonTemplateUsage,
    chrome: GalleryChrome,
    theme: &GalleryThemePack,
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
                .child("Template Parameters"),
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
                .child(format!("fields: {}", parameter.param_fields.join(", "))),
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

fn param_type_label(param_type: ProtoButtonTemplateParamType) -> &'static str {
    match param_type {
        ProtoButtonTemplateParamType::Color => "Color",
        ProtoButtonTemplateParamType::Pixels => "Pixels",
        ProtoButtonTemplateParamType::Number => "Number",
        ProtoButtonTemplateParamType::Bool => "Bool",
        ProtoButtonTemplateParamType::Enum => "Enum",
    }
}
