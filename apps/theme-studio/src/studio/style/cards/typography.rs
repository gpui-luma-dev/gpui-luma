use super::super::*;

pub(in crate::studio::style::style_guide) fn render_typography_section(look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

    section_shell_with_width(
        860.0,
        "Typography",
        "Combines the semantic role references, composition example, and both theme-aware and fixed text scale samples into a single style-guide card.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .flex()
            .flex_col()
            .gap(px(24.0))
            .children([
                render_typography_subsection(
                    "Luma semantic roles",
                    "Theme-resolved document-style roles from `ShadcnLook::typography_role(...)` and the `LumaTypographyExt` semantic helpers.",
                    chrome.muted_text,
                    render_semantic_content(look),
                ),
                render_typography_subsection(
                    "Semantic composition example",
                    "A practical example of how the semantic heading and paragraph roles could be composed in a product surface.",
                    chrome.muted_text,
                    render_semantic_composition_content(look),
                ),
                render_typography_subsection(
                    "Luma scale helpers",
                    "Theme-aware scale steps using SDK-owned names that avoid GPUI method collisions.",
                    chrome.muted_text,
                    render_scale_content(look),
                ),
                render_typography_subsection(
                    "GPUI built-in text scale helpers",
                    "App-level fixed utilities. Useful for pragmatic local sizing, but not theme-resolved typography.",
                    chrome.muted_text,
                    render_gpui_scale_content(chrome.body_text),
                ),
            ])
            .into_any_element(),
    )
}

fn render_typography_subsection(
    title: &'static str,
    description: &'static str,
    muted_text: gpui::Hsla,
    content: AnyElement,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(13.0))
                        .line_height(px(18.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(muted_text)
                        .child(title),
                )
                .child(div().typography_sm().text_color(muted_text).child(description)),
        )
        .child(content)
        .into_any_element()
}

fn render_semantic_content(look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

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
    .into_any_element()
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

fn render_semantic_composition_content(look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

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
        .into_any_element()
}

fn render_scale_content(look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

    vstack! {
        gap=12;
        render_scale_row("typography_xs()", div().typography_xs().text_color(chrome.body_text).child("Extra small helper text"), "xs"),
        render_scale_row("typography_sm()", div().typography_sm().text_color(chrome.body_text).child("Small helper text"), "sm"),
        render_scale_row("typography_md()", div().typography_md().text_color(chrome.body_text).child("Default body text"), "md/base"),
        render_scale_row("typography_lg()", div().typography_lg().text_color(chrome.body_text).child("Large list title text"), "lg"),
        render_scale_row("typography_xl()", div().typography_xl().text_color(chrome.body_text).child("Extra large title text"), "xl"),
        render_scale_row("typography_2xl()", div().typography_2xl().text_color(chrome.body_text).child("2XL section title text"), "2xl"),
    }
    .into_any_element()
}

fn render_gpui_scale_content(body_text: gpui::Hsla) -> AnyElement {
    vstack! {
        gap=12;
        render_scale_row("text_xs()", div().text_xs().text_color(body_text).child("GPUI extra small"), "fixed xs"),
        render_scale_row("text_sm()", div().text_sm().text_color(body_text).child("GPUI small"), "fixed sm"),
        render_scale_row("text_base()", div().text_base().text_color(body_text).child("GPUI base"), "fixed base"),
        render_scale_row("text_lg()", div().text_lg().text_color(body_text).child("GPUI large"), "fixed lg"),
        render_scale_row("text_xl()", div().text_xl().text_color(body_text).child("GPUI extra large"), "fixed xl"),
        render_scale_row("text_2xl()", div().text_2xl().text_color(body_text).child("GPUI 2XL"), "fixed 2xl"),
    }
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
