use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, px};
use gpui_luma::theme::RadixTheme;

use crate::gallery::panes::shared::format_compact_hsla;

struct CatalogColorItem {
    token: String,
    color: gpui::Hsla,
}

struct CatalogSection {
    title: &'static str,
    items: Vec<CatalogColorItem>,
}

pub(in crate::gallery) fn render(radix_theme: &RadixTheme) -> AnyElement {
    let chrome = radix_theme.chrome();
    let sections = catalog_sections(radix_theme);

    div()
        .size_full()
        .bg(chrome.content_background)
        .text_color(chrome.body_text)
        .p(px(28.0))
        .overflow_hidden()
        .child(
            div()
                .id("palette-content")
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
                                .child("Palette"),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .line_height(px(18.0))
                                .text_color(chrome.muted_text)
                                .child("CSS custom properties from the active tweakcn theme"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(20.0))
                        .children(sections.into_iter().map(|section| render_section(section, radix_theme))),
                ),
        )
        .into_any_element()
}

fn catalog_sections(radix_theme: &RadixTheme) -> Vec<CatalogSection> {
    let catalog = radix_theme.mode_tokens().catalog.clone();
    let mut core = Vec::new();
    let mut sidebar = Vec::new();
    let mut chart = Vec::new();
    let mut other = Vec::new();

    for (token, _) in catalog.tokens.iter() {
        let Ok(color) = catalog.color(token) else {
            continue;
        };
        let item = CatalogColorItem { token: token.clone(), color };
        if token.starts_with("sidebar") {
            sidebar.push(item);
        } else if token.starts_with("chart-") {
            chart.push(item);
        } else if matches!(
            token.as_str(),
            "background"
                | "foreground"
                | "card"
                | "card-foreground"
                | "popover"
                | "popover-foreground"
                | "primary"
                | "primary-foreground"
                | "secondary"
                | "secondary-foreground"
                | "muted"
                | "muted-foreground"
                | "accent"
                | "accent-foreground"
                | "destructive"
                | "destructive-foreground"
                | "border"
                | "input"
                | "ring"
        ) {
            core.push(item);
        } else {
            other.push(item);
        }
    }

    let mut sections = Vec::new();
    if !core.is_empty() {
        sections.push(CatalogSection { title: "Core", items: core });
    }
    if !sidebar.is_empty() {
        sections.push(CatalogSection { title: "Sidebar", items: sidebar });
    }
    if !chart.is_empty() {
        sections.push(CatalogSection { title: "Chart", items: chart });
    }
    if !other.is_empty() {
        sections.push(CatalogSection { title: "Other", items: other });
    }
    sections
}

fn render_section(section: CatalogSection, radix_theme: &RadixTheme) -> AnyElement {
    let chrome = radix_theme.chrome();

    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .text_size(px(13.0))
                .line_height(px(18.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.title_text)
                .child(section.title),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(12.0))
                .children(section.items.into_iter().map(|item| render_color_item(item, radix_theme))),
        )
        .into_any_element()
}

fn render_color_item(item: CatalogColorItem, radix_theme: &RadixTheme) -> AnyElement {
    let chrome = radix_theme.chrome();

    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .w(px(278.0))
        .min_h(px(42.0))
        .child(div().size(px(34.0)).bg(item.color).border_1().border_color(chrome.border).rounded(px(3.0)))
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
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.title_text)
                        .child(format!("--{}", item.token)),
                )
                .child(
                    div()
                        .truncate()
                        .font_family("Monaco")
                        .text_size(px(11.0))
                        .line_height(px(15.0))
                        .text_color(chrome.muted_text)
                        .child(format_compact_hsla(item.color)),
                ),
        )
        .into_any_element()
}
