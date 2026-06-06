use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, px};
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnModeTokens};

use crate::gallery::panes::shared::format_compact_hsla;
use crate::gallery::panes::shared::render_sparse_catalog_callout;

struct CatalogColorItem {
    token: String,
    color: gpui::Hsla,
}

struct CatalogSection {
    title: &'static str,
    items: Vec<CatalogColorItem>,
}

struct ModePalette {
    label: &'static str,
    background: gpui::Hsla,
    border: gpui::Hsla,
    title_text: gpui::Hsla,
    muted_text: gpui::Hsla,
    sections: Vec<CatalogSection>,
}

pub(in crate::gallery) fn render(look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    let palettes = [mode_palette(look.light_tokens(), "Light"), mode_palette(look.dark_tokens(), "Dark")];

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
                        .child(div().text_size(px(13.0)).line_height(px(18.0)).text_color(chrome.muted_text).child(
                            if look.has_css_catalog() {
                                "CSS custom properties from the active tweakcn theme"
                            } else {
                                "No tweakcn CSS loaded — native default uses the embedded SDK palette only"
                            },
                        )),
                )
                .when_some(render_sparse_catalog_callout(look), |panel, callout| panel.child(callout))
                .child(div().flex().flex_col().gap(px(24.0)).children(palettes.into_iter().map(render_mode_palette))),
        )
        .into_any_element()
}

fn mode_palette(tokens: &ShadcnModeTokens, label: &'static str) -> ModePalette {
    let palette = tokens.palette;

    ModePalette {
        label,
        background: palette.app_background,
        border: palette.border_default,
        title_text: palette.app_foreground,
        muted_text: palette.app_muted_foreground,
        sections: catalog_sections(tokens),
    }
}

fn catalog_sections(tokens: &ShadcnModeTokens) -> Vec<CatalogSection> {
    let catalog = &tokens.catalog;
    let mut core = Vec::new();
    let mut sidebar = Vec::new();
    let mut chart = Vec::new();
    let mut other = Vec::new();

    for token in catalog.tokens.keys() {
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

fn render_mode_palette(palette: ModePalette) -> AnyElement {
    let ModePalette { label, background, border, title_text, muted_text, sections, .. } = palette;

    div()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(
            div()
                .text_size(px(14.0))
                .line_height(px(20.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_text)
                .child(label),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .border_1()
                .border_color(border)
                .rounded(px(10.0))
                .bg(background)
                .p(px(18.0))
                .children(sections.into_iter().map(|section| render_section(section, border, title_text, muted_text))),
        )
        .into_any_element()
}

fn render_section(
    section: CatalogSection,
    border: gpui::Hsla,
    title_text: gpui::Hsla,
    muted_text: gpui::Hsla,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .text_size(px(13.0))
                .line_height(px(18.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_text)
                .child(section.title),
        )
        .child(
            div().flex().flex_wrap().gap(px(12.0)).children(
                section.items.into_iter().map(|item| render_color_item(item, border, title_text, muted_text)),
            ),
        )
        .into_any_element()
}

fn render_color_item(
    item: CatalogColorItem,
    border: gpui::Hsla,
    title_text: gpui::Hsla,
    muted_text: gpui::Hsla,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .w(px(278.0))
        .min_h(px(42.0))
        .child(div().size(px(34.0)).bg(item.color).border_1().border_color(border).rounded(px(3.0)))
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
                        .text_color(title_text)
                        .child(format!("--{}", item.token)),
                )
                .child(
                    div()
                        .truncate()
                        .font_family("Monaco")
                        .text_size(px(11.0))
                        .line_height(px(15.0))
                        .text_color(muted_text)
                        .child(format_compact_hsla(item.color)),
                ),
        )
        .into_any_element()
}
