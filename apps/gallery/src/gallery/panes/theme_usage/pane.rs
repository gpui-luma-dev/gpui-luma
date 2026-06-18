use std::collections::BTreeMap;

use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, px};
use gpui_luma::theme::{ThemePartUsage, ThemeUsage};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextRole, ShadcnTextSize, all_shadcn_theme_usages};

use crate::gallery::panes::shared::format_compact_hsla;
use crate::gallery::panes::shared::render_sparse_catalog_callout;

type UsageRef = (&'static str, &'static ThemePartUsage);

#[derive(Clone)]
struct CatalogToken {
    token: String,
    color: gpui::Hsla,
}

pub(in crate::gallery) fn render(look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    let heading_style = look.typography_role(ShadcnTextRole::H3);
    let body_style = look.typography_scale(ShadcnTextSize::Sm);
    let section_style = look.typography_scale(ShadcnTextSize::Sm);
    let detail_style = look.typography_scale(ShadcnTextSize::Sm);
    let caption_style = look.typography_scale(ShadcnTextSize::Xs);
    let catalog_tokens = catalog_tokens(look);
    let usages = all_shadcn_theme_usages();
    let by_token = usage_by_token(usages);
    let sdk_token_count = catalog_tokens.iter().filter(|token| by_token.contains_key(token.token.as_str())).count();
    let shared_value_count = shared_value_groups(&catalog_tokens).len();

    div()
        .size_full()
        .bg(chrome.content_background)
        .text_color(chrome.body_text)
        .p(px(28.0))
        .overflow_hidden()
        .child(
            div()
                .id("theme-usage-content")
                .size_full()
                .flex()
                .flex_col()
                .gap(px(18.0))
                .overflow_y_scroll()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(div().typography_style(heading_style).text_color(chrome.title_text).child("Theme Usage"))
                        .child(div().typography_style(body_style).text_color(chrome.muted_text).child(
                            if look.has_css_catalog() {
                                "Shadcn CSS token usage metadata for migrated controls"
                            } else {
                                "SDK resolver metadata — CSS catalog empty on native default theme"
                            },
                        )),
                )
                .when_some(render_sparse_catalog_callout(look), |panel, callout| panel.child(callout))
                .child(div().flex().gap(px(8.0)).children([
                    render_count_badge("Components", usages.len().to_string(), look, caption_style),
                    render_count_badge("Catalog tokens", catalog_tokens.len().to_string(), look, caption_style),
                    render_count_badge("Used by SDK", sdk_token_count.to_string(), look, caption_style),
                    render_count_badge("Shared values", shared_value_count.to_string(), look, caption_style),
                ]))
                .child(
                    div()
                        .flex()
                        .gap(px(20.0))
                        .items_start()
                        .child(render_by_token(&catalog_tokens, &by_token, look, detail_style))
                        .child(render_by_component(usages, look, detail_style, caption_style)),
                )
                .child(render_shared_values(
                    &catalog_tokens,
                    &by_token,
                    look,
                    caption_style,
                    detail_style,
                    section_style,
                )),
        )
        .into_any_element()
}

fn catalog_tokens(look: &ShadcnLook) -> Vec<CatalogToken> {
    let catalog = &look.mode_tokens().catalog;
    let mut tokens: Vec<CatalogToken> = catalog
        .tokens
        .keys()
        .filter_map(|token| catalog.color(token).ok().map(|color| CatalogToken { token: token.clone(), color }))
        .collect();
    tokens.sort_by(|left, right| left.token.cmp(&right.token));
    tokens
}

fn usage_by_token(usages: &'static [&'static ThemeUsage]) -> BTreeMap<&'static str, Vec<UsageRef>> {
    let mut by_token: BTreeMap<&'static str, Vec<UsageRef>> = BTreeMap::new();

    for usage in usages {
        for part in usage.parts {
            by_token.entry(part.token).or_default().push((usage.label, part));
        }
    }

    by_token
}

fn render_by_token(
    catalog_tokens: &[CatalogToken],
    by_token: &BTreeMap<&'static str, Vec<UsageRef>>,
    look: &ShadcnLook,
    detail_style: gpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    let chrome = look.chrome();

    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(section_title("By Token", look, detail_style))
        .children(catalog_tokens.iter().map(|token| {
            let consumers = by_token.get(token.token.as_str());

            div()
                .flex()
                .flex_col()
                .gap(px(7.0))
                .border_1()
                .border_color(chrome.border)
                .rounded(px(6.0))
                .p(px(10.0))
                .child(render_token_header(token, consumers.is_some(), look))
                .children(consumers.into_iter().flat_map(|parts| {
                    parts.iter().map(|(component, part)| {
                        div()
                            .pl(px(44.0))
                            .typography_style(detail_style)
                            .text_color(chrome.body_text)
                            .child(format!("{component} {}", part.part))
                    })
                }))
                .when(consumers.is_none(), |row| {
                    row.child(
                        div()
                            .pl(px(44.0))
                            .typography_style(detail_style)
                            .text_color(chrome.muted_text)
                            .child("No current SDK resolver usage"),
                    )
                })
        }))
        .into_any_element()
}

fn render_by_component(
    usages: &'static [&'static ThemeUsage],
    look: &ShadcnLook,
    detail_style: gpui_luma::theme::LumaTextStyle,
    caption_style: gpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    let chrome = look.chrome();

    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(section_title("By Component", look, detail_style))
        .children(usages.iter().map(|usage| {
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .border_1()
                .border_color(chrome.border)
                .rounded(px(6.0))
                .p(px(10.0))
                .child(
                    div()
                        .typography_style(detail_style)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(usage.label),
                )
                .children(usage.parts.iter().map(|part| render_component_part(part, look, detail_style, caption_style)))
        }))
        .into_any_element()
}

fn render_shared_values(
    catalog_tokens: &[CatalogToken],
    by_token: &BTreeMap<&'static str, Vec<UsageRef>>,
    look: &ShadcnLook,
    caption_style: gpui_luma::theme::LumaTextStyle,
    detail_style: gpui_luma::theme::LumaTextStyle,
    section_style: gpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    let chrome = look.chrome();
    let groups = shared_value_groups(catalog_tokens);

    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(section_title("Shared Values", look, section_style))
        .child(div().flex().flex_wrap().gap(px(12.0)).children(groups.into_iter().map(|(value, tokens)| {
            div()
                .w(px(360.0))
                .min_h(px(0.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .border_1()
                .border_color(chrome.border)
                .rounded(px(6.0))
                .p(px(10.0))
                .child(
                    div()
                        .font_family("Monaco")
                        .typography_style(caption_style)
                        .text_color(chrome.muted_text)
                        .child(value),
                )
                .children(tokens.into_iter().map(|token| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().size(px(16.0)).bg(token.color).border_1().border_color(chrome.border))
                        .child(
                            div()
                                .min_w(px(0.0))
                                .truncate()
                                .font_family("Monaco")
                                .typography_style(detail_style)
                                .text_color(chrome.title_text)
                                .child(token_label(token)),
                        )
                        .child(render_status_badge(token_status(by_token.contains_key(token.token.as_str())), look))
                }))
        })))
        .into_any_element()
}

fn shared_value_groups(catalog_tokens: &[CatalogToken]) -> Vec<(String, Vec<&CatalogToken>)> {
    let mut by_value: BTreeMap<String, Vec<&CatalogToken>> = BTreeMap::new();

    for token in catalog_tokens {
        by_value.entry(format_compact_hsla(token.color)).or_default().push(token);
    }

    by_value.into_iter().filter(|(_, tokens)| tokens.len() > 1).collect()
}

fn section_title(title: &'static str, look: &ShadcnLook, style: gpui_luma::theme::LumaTextStyle) -> AnyElement {
    let chrome = look.chrome();

    div()
        .typography_style(style)
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(chrome.title_text)
        .child(title)
        .into_any_element()
}

fn render_count_badge(
    label: &'static str,
    value: String,
    look: &ShadcnLook,
    style: gpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    let chrome = look.chrome();

    div()
        .flex()
        .items_center()
        .gap(px(6.0))
        .border_1()
        .border_color(chrome.border)
        .rounded(px(4.0))
        .px(px(8.0))
        .py(px(4.0))
        .typography_style(style)
        .child(div().font_weight(FontWeight::SEMIBOLD).text_color(chrome.title_text).child(value))
        .child(div().text_color(chrome.muted_text).child(label))
        .into_any_element()
}

fn render_token_header(token: &CatalogToken, used_by_sdk: bool, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    let detail_style = look.typography_scale(ShadcnTextSize::Sm);
    let caption_style = look.typography_scale(ShadcnTextSize::Xs);

    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(div().size(px(34.0)).bg(token.color).border_1().border_color(chrome.border).rounded(px(3.0)))
        .child(
            div()
                .min_w(px(0.0))
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(1.0))
                .child(
                    div()
                        .truncate()
                        .font_family("Monaco")
                        .typography_style(detail_style)
                        .text_color(chrome.title_text)
                        .child(token_label(token)),
                )
                .child(
                    div()
                        .truncate()
                        .font_family("Monaco")
                        .typography_style(caption_style)
                        .text_color(chrome.muted_text)
                        .child(format_compact_hsla(token.color)),
                ),
        )
        .child(render_status_badge(token_status(used_by_sdk), look))
        .into_any_element()
}

fn render_component_part(
    part: &ThemePartUsage,
    look: &ShadcnLook,
    detail_style: gpui_luma::theme::LumaTextStyle,
    caption_style: gpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    let chrome = look.chrome();

    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .min_w(px(172.0))
                        .typography_style(detail_style)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.title_text)
                        .child(part.part),
                )
                .child(
                    div()
                        .min_w(px(0.0))
                        .truncate()
                        .font_family("Monaco")
                        .typography_style(detail_style)
                        .text_color(chrome.body_text)
                        .child(format!("--{}", part.token)),
                ),
        )
        .child(div().pl(px(180.0)).typography_style(caption_style).text_color(chrome.muted_text).child(format!(
            "{} -> {}",
            part.states.join(", "),
            part.appearance_fields.join(", ")
        )))
        .into_any_element()
}

fn render_status_badge(status: &'static str, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    let caption_style = look.typography_scale(ShadcnTextSize::Xs);

    div()
        .flex_none()
        .border_1()
        .border_color(chrome.border)
        .rounded(px(3.0))
        .px(px(6.0))
        .py(px(2.0))
        .typography_style(caption_style)
        .text_color(chrome.muted_text)
        .child(status)
        .into_any_element()
}

fn token_status(used_by_sdk: bool) -> &'static str {
    if used_by_sdk { "Used by SDK" } else { "No current usage" }
}

fn token_label(token: &CatalogToken) -> String {
    format!("--{}", token.token)
}
