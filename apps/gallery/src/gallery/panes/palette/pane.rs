use gpui::{AnyElement, FontWeight, Hsla, IntoElement, div, prelude::*, px};
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnModeTokens};

use crate::gallery::panes::shared::{format_hex_color, render_sparse_catalog_callout};

const PALETTE_SWATCH_SIZE: f32 = 80.0;
const PALETTE_ITEM_GAP: f32 = 12.0;
const PALETTE_CELL_INNER_GAP: f32 = 10.0;
const PALETTE_CELL_TEXT_WIDTH: f32 = 200.0;
const PALETTE_CELL_WIDTH: f32 = PALETTE_SWATCH_SIZE + PALETTE_CELL_INNER_GAP + PALETTE_CELL_TEXT_WIDTH;

const PALETTE_CATEGORIES: &[(&str, &[(&str, &str)])] = &[
    (
        "Primary Theme Colors",
        &[
            ("background", "Background"),
            ("foreground", "Foreground"),
            ("primary", "Primary"),
            ("primary-foreground", "Primary Foreground"),
        ],
    ),
    (
        "Secondary & Accent Colors",
        &[
            ("secondary", "Secondary"),
            ("secondary-foreground", "Secondary Foreground"),
            ("accent", "Accent"),
            ("accent-foreground", "Accent Foreground"),
        ],
    ),
    (
        "UI Component Colors",
        &[
            ("card", "Card"),
            ("card-foreground", "Card Foreground"),
            ("popover", "Popover"),
            ("popover-foreground", "Popover Foreground"),
            ("muted", "Muted"),
            ("muted-foreground", "Muted Foreground"),
        ],
    ),
    ("Utility & Form Colors", &[("border", "Border"), ("input", "Input"), ("ring", "Ring")]),
    (
        "Status & Feedback Colors",
        &[("destructive", "Destructive"), ("destructive-foreground", "Destructive Foreground")],
    ),
    (
        "Chart & Visualization Colors",
        &[
            ("chart-1", "Chart 1"),
            ("chart-2", "Chart 2"),
            ("chart-3", "Chart 3"),
            ("chart-4", "Chart 4"),
            ("chart-5", "Chart 5"),
        ],
    ),
    (
        "Sidebar & Navigation Colors",
        &[
            ("sidebar", "Sidebar Background"),
            ("sidebar-foreground", "Sidebar Foreground"),
            ("sidebar-primary", "Sidebar Primary"),
            ("sidebar-primary-foreground", "Sidebar Primary Foreground"),
            ("sidebar-accent", "Sidebar Accent"),
            ("sidebar-accent-foreground", "Sidebar Accent Foreground"),
            ("sidebar-border", "Sidebar Border"),
            ("sidebar-ring", "Sidebar Ring"),
        ],
    ),
];

struct TokenSwatchRow {
    label: &'static str,
    color: Hsla,
}

struct PaletteSection {
    title: &'static str,
    rows: Vec<TokenSwatchRow>,
}

struct ModePalette {
    label: &'static str,
    background: Hsla,
    border: Hsla,
    title_text: Hsla,
    muted_text: Hsla,
    sections: Vec<PaletteSection>,
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
                                "Theme colors grouped by role"
                            } else {
                                "Native default theme: pick a tweakcn theme in Settings for catalog-backed swatches"
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
    let fallback = gpui::hsla(0.0, 0.0, 0.5, 1.0);

    ModePalette {
        label,
        background: palette.app_background,
        border: palette.border_default,
        title_text: palette.app_foreground,
        muted_text: palette.app_muted_foreground,
        sections: PALETTE_CATEGORIES
            .iter()
            .map(|(title, token_rows)| PaletteSection {
                title,
                rows: token_rows
                    .iter()
                    .map(|(token, token_label)| TokenSwatchRow {
                        label: token_label,
                        color: tokens.catalog.color(token).unwrap_or(fallback),
                    })
                    .collect(),
            })
            .collect(),
    }
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
                .gap(px(20.0))
                .border_1()
                .border_color(border)
                .rounded(px(10.0))
                .bg(background)
                .p(px(18.0))
                .children(sections.into_iter().map(|section| render_section(section, title_text, muted_text, border))),
        )
        .into_any_element()
}

fn render_section(section: PaletteSection, title_text: Hsla, muted_text: Hsla, border: Hsla) -> AnyElement {
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
                .child(section.title),
        )
        .child(
            div()
                .w_full()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(px(PALETTE_ITEM_GAP))
                .children(section.rows.into_iter().map(|row| render_token_cell(row, title_text, muted_text, border))),
        )
        .into_any_element()
}

fn render_token_cell(row: TokenSwatchRow, title_text: Hsla, muted_text: Hsla, border: Hsla) -> AnyElement {
    div()
        .w(px(PALETTE_CELL_WIDTH))
        .py(px(4.0))
        .flex_shrink_0()
        .flex()
        .items_start()
        .gap(px(PALETTE_CELL_INNER_GAP))
        .child(
            div()
                .size(px(PALETTE_SWATCH_SIZE))
                .flex_shrink_0()
                .rounded(px(6.0))
                .bg(row.color)
                .border_1()
                .border_color(border),
        )
        .child(
            div()
                .w(px(PALETTE_CELL_TEXT_WIDTH))
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_size(px(13.0))
                        .line_height(px(18.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(title_text)
                        .child(row.label),
                )
                .child(
                    div()
                        .font_family("Monaco")
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(muted_text)
                        .child(format_hex_color(row.color)),
                ),
        )
        .into_any_element()
}
