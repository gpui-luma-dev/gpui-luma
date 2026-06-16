use gpui::{AnyElement, FontWeight, div, prelude::*, px};
use gpui_luma::{hstack, vstack};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use super::shared::render_sparse_catalog_callout;

const TYPOGRAPHY_DESCRIPTION: &str = concat!(
    "Reference page for the gallery typography system. ",
    "Shows Luma semantic roles, Luma scale helpers, and GPUI built-in text scale helpers side by side."
);

pub(in crate::gallery) fn render(look: &ShadcnLook) -> AnyElement {
    with_look(&std::sync::Arc::new(look.clone()), || {
        let chrome = look.chrome();

        div()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(chrome.content_background)
            .p(px(28.0))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(20.0))
                            .line_height(px(28.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(chrome.title_text)
                            .child("Typography"),
                    )
                    .child(
                        div()
                            .max_w(px(760.0))
                            .text_size(px(13.0))
                            .line_height(px(18.0))
                            .text_color(chrome.muted_text)
                            .child(TYPOGRAPHY_DESCRIPTION),
                    ),
            )
            .child(
                div().w_full().min_h(px(0.0)).flex_1().child(
                    div()
                        .id("typography-content")
                        .size_full()
                        .flex()
                        .flex_col()
                        .gap(px(18.0))
                        .overflow_y_scroll()
                        .pt(px(18.0))
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .flex_wrap()
                                .items_start()
                                .gap(px(20.0))
                                .when_some(render_sparse_catalog_callout(look), |panel, callout| panel.child(callout))
                                .children([
                                    render_semantic_section(look),
                                    render_semantic_composition_section(look),
                                    render_scale_section(look),
                                    render_gpui_scale_section(
                                        chrome.muted_text,
                                        chrome.body_text,
                                        chrome.border,
                                        chrome.panel_background,
                                    ),
                                ]),
                        ),
                ),
            )
            .into_any_element()
    })
}

fn render_semantic_section(look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

    section_shell(
        "Luma semantic roles",
        "Theme-resolved document-style roles from `ShadcnLook::typography_role(...)` and the `LumaTypographyExt` semantic helpers.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        vstack! {
            gap=14;
            render_semantic_row("text_h1() / role h1", div().text_h1().text_color(chrome.title_text).child("Heading One")),
            render_semantic_row("text_h2() / role h2", div().text_h2().text_color(chrome.title_text).child("Heading Two")),
            render_semantic_row("text_h3() / role h3", div().text_h3().text_color(chrome.title_text).child("Heading Three")),
            render_semantic_row("text_h4() / role h4", div().text_h4().text_color(chrome.title_text).child("Heading Four")),
            render_semantic_row(
                "text_p() / role p",
                div()
                    .text_p()
                    .text_color(chrome.body_text)
                    .child("Body paragraph copy resolved from the active look typography tokens."),
            ),
        }
        .into_any_element(),
    )
}

fn render_semantic_row(label: &'static str, sample: impl IntoElement) -> AnyElement {
    hstack! {
        gap=16 align=start;
        div()
            .w(px(200.0))
            .min_w(px(200.0))
            .text_xs()
            .line_height(px(15.0))
            .text_color(gpui::hsla(0.0, 0.0, 0.55, 1.0))
            .child(label),
        div().flex_1().min_w(px(0.0)).child(sample),
    }
    .into_any_element()
}

fn render_semantic_composition_section(look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

    section_shell(
        "Semantic composition example",
        "A practical example of how the semantic heading and paragraph roles could be composed in a product surface.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap(px(14.0))
            .child(
                div()
                    .w(px(420.0))
                    .max_w_full()
                    .flex()
                    .flex_col()
                    .gap(px(14.0))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(chrome.border)
                    .bg(chrome.content_background)
                    .p(px(18.0))
                    .child(div().text_h1().text_color(chrome.title_text).child("Foundation Controls"))
                    .child(div().text_p().text_color(chrome.body_text).child(
                        "A hero section for a product shell using the larger semantic page title and paragraph copy.",
                    ))
                    .child(div().text_h2().text_color(chrome.title_text).child("Primary Section"))
                    .child(
                        div()
                            .text_p()
                            .text_color(chrome.body_text)
                            .child("Use h2 for the main page sections that structure the content and orient the user."),
                    )
                    .child(div().text_h3().text_color(chrome.title_text).child("Panel Heading"))
                    .child(
                        div()
                            .text_p()
                            .text_color(chrome.body_text)
                            .child("Use h3 for a panel or content block title inside the main section."),
                    )
                    .child(div().text_h4().text_color(chrome.title_text).child("Field Group"))
                    .child(
                        div()
                            .text_p()
                            .text_color(chrome.body_text)
                            .child("Use h4 for smaller grouped content or component subheaders."),
                    ),
            )
            .into_any_element(),
    )
}

fn render_scale_section(look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

    section_shell(
        "Luma scale helpers",
        "Theme-aware scale steps using SDK-owned names that avoid GPUI method collisions.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        vstack! {
            gap=12;
            render_scale_row("typography_xs()", div().typography_xs().text_color(chrome.body_text).child("Extra small helper text"), "xs"),
            render_scale_row("typography_sm()", div().typography_sm().text_color(chrome.body_text).child("Small helper text"), "sm"),
            render_scale_row("typography_md()", div().typography_md().text_color(chrome.body_text).child("Default body text"), "md/base"),
            render_scale_row("typography_lg()", div().typography_lg().text_color(chrome.body_text).child("Large list title text"), "lg"),
            render_scale_row("typography_xl()", div().typography_xl().text_color(chrome.body_text).child("Extra large title text"), "xl"),
            render_scale_row("typography_2xl()", div().typography_2xl().text_color(chrome.body_text).child("2XL section title text"), "2xl"),
        }
        .into_any_element(),
    )
}

fn render_gpui_scale_section(
    muted_text: gpui::Hsla,
    body_text: gpui::Hsla,
    border: gpui::Hsla,
    background: gpui::Hsla,
) -> AnyElement {
    section_shell(
        "GPUI built-in text scale helpers",
        "App-level fixed utilities. Useful for pragmatic local sizing, but not theme-resolved typography.",
        body_text,
        muted_text,
        border,
        background,
        vstack! {
            gap=12;
            render_scale_row("text_xs()", div().text_xs().text_color(body_text).child("GPUI extra small"), "fixed xs"),
            render_scale_row("text_sm()", div().text_sm().text_color(body_text).child("GPUI small"), "fixed sm"),
            render_scale_row("text_base()", div().text_base().text_color(body_text).child("GPUI base"), "fixed base"),
            render_scale_row("text_lg()", div().text_lg().text_color(body_text).child("GPUI large"), "fixed lg"),
            render_scale_row("text_xl()", div().text_xl().text_color(body_text).child("GPUI extra large"), "fixed xl"),
            render_scale_row("text_2xl()", div().text_2xl().text_color(body_text).child("GPUI 2XL"), "fixed 2xl"),
        }
        .into_any_element(),
    )
}

fn section_shell(
    title: &'static str,
    description: &'static str,
    title_color: gpui::Hsla,
    muted_text: gpui::Hsla,
    border: gpui::Hsla,
    background: gpui::Hsla,
    content: AnyElement,
) -> AnyElement {
    div()
        .w(px(560.0))
        .max_w_full()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .rounded(px(12.0))
        .border_1()
        .border_color(border)
        .bg(background)
        .p(px(18.0))
        .child(
            vstack! {
                gap=4;
                div().text_h4().text_color(title_color).child(title),
                div().typography_sm().text_color(muted_text).child(description),
            }
            .into_any_element(),
        )
        .child(content)
        .into_any_element()
}

fn render_scale_row(label: &'static str, sample: impl IntoElement, badge: &'static str) -> AnyElement {
    hstack! {
        gap=16 align=center;
        div()
            .w(px(200.0))
            .min_w(px(200.0))
            .text_xs()
            .line_height(px(15.0))
            .text_color(gpui::hsla(0.0, 0.0, 0.55, 1.0))
            .child(label),
        div().flex_1().min_w(px(0.0)).child(sample),
        div()
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(999.0))
            .border_1()
            .border_color(gpui::hsla(0.0, 0.0, 0.75, 0.35))
            .text_xs()
            .line_height(px(14.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(gpui::hsla(0.0, 0.0, 0.62, 1.0))
            .child(badge),
    }
    .into_any_element()
}
