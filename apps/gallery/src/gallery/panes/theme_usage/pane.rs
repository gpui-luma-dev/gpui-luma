use std::collections::BTreeMap;

use gpui::{AnyElement, FontWeight, Hsla, IntoElement, div, prelude::*, px};
use gpui_luma::theme::{ThemePartUsage, ThemeTokens, ThemeUsage};

use crate::gallery::theme::GalleryThemePack;

pub(in crate::gallery) fn render(theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();
    let usage = theme.navigation_sidebar_theme().usage();
    let tokens = theme.tokens();

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
                        .child(
                            div()
                                .text_size(px(20.0))
                                .line_height(px(28.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(chrome.title_text)
                                .child("Theme Usage"),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .line_height(px(18.0))
                                .text_color(chrome.muted_text)
                                .child("SDK resolver metadata for semantic color token usage"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(28.0))
                        .items_start()
                        .child(render_by_token(usage, &tokens, theme))
                        .child(render_by_component(usage, theme)),
                ),
        )
        .into_any_element()
}

fn render_by_token(usage: &ThemeUsage, tokens: &ThemeTokens, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();
    let mut by_token: BTreeMap<&'static str, Vec<&ThemePartUsage>> = BTreeMap::new();

    for part in usage.parts {
        by_token.entry(part.token).or_default().push(part);
    }

    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(section_title("By Token", theme))
        .children(by_token.into_iter().map(|(token, parts)| {
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .border_1()
                .border_color(chrome.border)
                .rounded(px(6.0))
                .p(px(10.0))
                .child(render_token_header(token, tokens, theme))
                .children(parts.into_iter().map(|part| {
                    div()
                        .pl(px(44.0))
                        .text_size(px(12.0))
                        .line_height(px(17.0))
                        .text_color(chrome.body_text)
                        .child(format!("{} {}", usage.component, part.part))
                }))
        }))
        .into_any_element()
}

fn render_by_component(usage: &ThemeUsage, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(section_title("By Component", theme))
        .child(
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
                        .text_size(px(13.0))
                        .line_height(px(18.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(usage.component),
                )
                .children(usage.parts.iter().map(|part| render_component_part(part, theme))),
        )
        .into_any_element()
}

fn section_title(title: &'static str, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

    div()
        .text_size(px(13.0))
        .line_height(px(18.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(chrome.title_text)
        .child(title)
        .into_any_element()
}

fn render_token_header(token: &'static str, tokens: &ThemeTokens, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();
    let color = resolve_token_color(tokens, token);

    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(
            div()
                .size(px(34.0))
                .bg(color.unwrap_or(chrome.panel_background))
                .border_1()
                .border_color(chrome.border)
                .rounded(px(3.0)),
        )
        .child(
            div()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .gap(px(1.0))
                .child(
                    div()
                        .truncate()
                        .font_family("Monaco")
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(chrome.title_text)
                        .child(token),
                )
                .child(
                    div()
                        .truncate()
                        .font_family("Monaco")
                        .text_size(px(11.0))
                        .line_height(px(15.0))
                        .text_color(chrome.muted_text)
                        .child(color.map(format_hsla).unwrap_or_else(|| "unresolved".to_string())),
                ),
        )
        .into_any_element()
}

fn render_component_part(part: &ThemePartUsage, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

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
                        .min_w(px(160.0))
                        .text_size(px(12.0))
                        .line_height(px(17.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.title_text)
                        .child(part.part),
                )
                .child(
                    div()
                        .min_w(px(0.0))
                        .truncate()
                        .font_family("Monaco")
                        .text_size(px(12.0))
                        .line_height(px(17.0))
                        .text_color(chrome.body_text)
                        .child(part.token),
                ),
        )
        .child(
            div()
                .pl(px(168.0))
                .text_size(px(11.0))
                .line_height(px(15.0))
                .text_color(chrome.muted_text)
                .child(format!("{} -> {}", part.states.join(", "), part.appearance_fields.join(", "))),
        )
        .into_any_element()
}

fn resolve_token_color(tokens: &ThemeTokens, token: &str) -> Option<Hsla> {
    let palette = &tokens.palette;

    match token {
        "action.primary.hover_background" => Some(palette.action.primary.hover_background),
        "action.primary.pressed_background" => Some(palette.action.primary.pressed_background),
        "navigation.background" => Some(palette.navigation.background),
        "navigation.foreground" => Some(palette.navigation.foreground),
        "navigation.muted_foreground" => Some(palette.navigation.muted_foreground),
        "navigation.hover_background" => Some(palette.navigation.hover_background),
        "navigation.selected_background" => Some(palette.navigation.selected_background),
        "navigation.selected_foreground" => Some(palette.navigation.selected_foreground),
        "navigation.border" => Some(palette.navigation.border),
        "navigation.focus_ring" => Some(palette.navigation.focus_ring),
        "state.disabled.foreground" => Some(palette.state.disabled.foreground),
        "state.pressed.background" => Some(palette.state.pressed.background),
        _ => None,
    }
}

fn format_hsla(color: Hsla) -> String {
    format!("hsla({:.1} {:.1}% {:.1}% / {:.2})", color.h * 360.0, color.s * 100.0, color.l * 100.0, color.a)
}
