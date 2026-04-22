use gpui::{AnyElement, Context, Entity, FontWeight, Hsla, IntoElement, div, prelude::*, px};
use gpui_luma::theme::{ThemePartUsage, all_theme_usages, resolve_palette_color};

use crate::gallery::{control::GalleryApp, theme::GalleryThemePack};

pub(super) fn gallery_pane(title: &'static str, content: AnyElement, theme: &GalleryThemePack) -> AnyElement {
    gallery_pane_with_description(title, None, content, theme)
}

pub(super) fn gallery_pane_with_description(
    title: &'static str,
    description: Option<&'static str>,
    content: AnyElement,
    theme: &GalleryThemePack,
) -> AnyElement {
    let chrome = theme.chrome();

    div()
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .overflow_hidden()
        .bg(chrome.content_background)
        .p(px(28.0))
        .child(render_pane_header(title, description, chrome.title_text, chrome.muted_text))
        .child(
            div().min_h(px(0.0)).flex_1().flex().items_center().justify_center().child(
                div().relative().flex().flex_col().items_center().justify_center().gap_4().occlude().child(content),
            ),
        )
        .into_any_element()
}

pub(super) fn gallery_pane_with_usage(
    title: &'static str,
    usage_component: &'static str,
    content: AnyElement,
    theme: &GalleryThemePack,
) -> AnyElement {
    gallery_pane_with_usage_description(title, None, usage_component, content, theme)
}

pub(super) fn gallery_pane_with_usage_description(
    title: &'static str,
    description: Option<&'static str>,
    usage_component: &'static str,
    content: AnyElement,
    theme: &GalleryThemePack,
) -> AnyElement {
    let chrome = theme.chrome();

    div()
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .overflow_hidden()
        .bg(chrome.content_background)
        .p(px(28.0))
        .child(render_pane_header(title, description, chrome.title_text, chrome.muted_text))
        .child(
            div()
                .min_h(px(0.0))
                .flex_1()
                .flex()
                .items_stretch()
                .justify_center()
                .gap(px(28.0))
                .child(
                    div()
                        .min_w(px(0.0))
                        .h_full()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_4()
                        .occlude()
                        .child(content),
                )
                .child(div().flex().items_center().child(render_usage_panel(usage_component, theme))),
        )
        .into_any_element()
}

fn render_pane_header(
    title: &'static str,
    description: Option<&'static str>,
    title_color: Hsla,
    description_color: Hsla,
) -> AnyElement {
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
                .text_color(title_color)
                .child(title),
        )
        .when_some(description, |header, description| {
            header.child(
                div()
                    .max_w(px(760.0))
                    .text_size(px(13.0))
                    .line_height(px(18.0))
                    .text_color(description_color)
                    .child(description),
            )
        })
        .into_any_element()
}

pub(super) fn notify_entity<T: 'static>(entity: &Entity<T>, cx: &mut Context<GalleryApp>) {
    entity.update(cx, |_, cx| cx.notify());
}

fn render_usage_panel(component: &'static str, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();
    let tokens = theme.tokens();
    let usage = all_theme_usages().iter().copied().find(|usage| usage.component == component);
    let parts = usage.map(|usage| usage.parts).unwrap_or(&[]);

    div()
        .id(format!("{component}-theme-usage"))
        .w(px(390.0))
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
                .child("Theme Parts"),
        )
        .children(parts.iter().map(|part| render_usage_part(part, &tokens, theme)))
        .when(usage.is_none(), |panel| {
            panel.child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(17.0))
                    .text_color(chrome.muted_text)
                    .child("No theme usage metadata registered."),
            )
        })
        .into_any_element()
}

fn render_usage_part(
    part: &ThemePartUsage,
    tokens: &gpui_luma::theme::ThemeTokens,
    theme: &GalleryThemePack,
) -> AnyElement {
    let chrome = theme.chrome();
    let color = resolve_palette_color(tokens, part.token);

    div()
        .flex()
        .flex_col()
        .gap(px(5.0))
        .border_1()
        .border_color(chrome.border)
        .rounded(px(5.0))
        .p(px(8.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .size(px(18.0))
                        .bg(color.unwrap_or(chrome.panel_background))
                        .border_1()
                        .border_color(chrome.border)
                        .rounded(px(3.0)),
                )
                .child(
                    div()
                        .min_w(px(0.0))
                        .flex_1()
                        .truncate()
                        .text_size(px(12.0))
                        .line_height(px(17.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.title_text)
                        .child(part.part),
                ),
        )
        .child(
            div()
                .truncate()
                .font_family("Monaco")
                .text_size(px(11.0))
                .line_height(px(15.0))
                .text_color(chrome.body_text)
                .child(part.token),
        )
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(chrome.muted_text).child(format!(
            "{} -> {}",
            part.states.join(", "),
            part.appearance_fields.join(", ")
        )))
        .when_some(color, |part, color| {
            part.child(
                div()
                    .truncate()
                    .font_family("Monaco")
                    .text_size(px(10.0))
                    .line_height(px(14.0))
                    .text_color(chrome.muted_text)
                    .child(format_compact_hsla(color)),
            )
        })
        .into_any_element()
}

pub(in crate::gallery::panes) fn format_compact_hsla(color: Hsla) -> String {
    format!(
        "hsla({} {}% {}% / {})",
        rounded_channel(color.h * 360.0),
        rounded_channel(color.s * 100.0),
        rounded_channel(color.l * 100.0),
        compact_alpha(color.a)
    )
}

fn rounded_channel(value: f32) -> i32 {
    value.round() as i32
}

fn compact_alpha(value: f32) -> String {
    let formatted = format!("{value:.3}");
    formatted.trim_end_matches('0').trim_end_matches('.').to_string()
}
