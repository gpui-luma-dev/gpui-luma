use gpui::{AnyElement, IntoElement, div, prelude::*, px};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook};

use crate::gallery::theme::GalleryThemeChoice;

pub(in crate::gallery) fn render_sparse_catalog_callout(look: &ShadcnLook) -> Option<AnyElement> {
    if look.has_css_catalog() {
        return None;
    }

    let chrome = look.chrome();
    let cli = GalleryThemeChoice::cli_usage_line();
    let title = "Native default theme — sparse CSS catalog";
    let body = format!(
        "This view lists tweakcn `--*` custom properties from the active theme. The native default has \
         no CSS catalog, so token swatches and cross-reference counts are empty. SDK controls still \
         resolve colors from the embedded palette. Restart with a CSS stem for full catalog data: {cli}"
    );

    Some(render_callout(
        title,
        &body,
        chrome.border,
        chrome.panel_background,
        chrome.title_text,
        chrome.body_text,
    ))
}

fn render_callout(
    title: &str,
    body: &str,
    border: gpui::Hsla,
    background: gpui::Hsla,
    title_color: gpui::Hsla,
    body_color: gpui::Hsla,
) -> AnyElement {
    div()
        .w_full()
        .max_w(px(860.0))
        .flex()
        .flex_col()
        .gap(px(6.0))
        .border_1()
        .border_color(border)
        .rounded(px(10.0))
        .bg(background)
        .p(px(14.0))
        .child(
            div()
                .typography_sm()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(title.to_string()),
        )
        .child(div().typography_xs().text_color(body_color).child(body.to_string()))
        .into_any_element()
}
