use std::collections::HashMap;
use std::sync::Arc;

use gpui::{AnyElement, Context, FontWeight, IntoElement, Render, Window, div, prelude::*, px};
use gpui_luma::theme::{LumaChrome, ThemeMode};
use gpui_luma::{declare_form, hstack, vstack};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnModeTokens, ShadcnTextRole, ShadcnTextSize};

use crate::studio::export::token_css_name;
use crate::studio::overrides::StudioOverrides;
use crate::studio::panels::format_hex_color;

const PALETTE_SWATCH_SIZE: f32 = 80.0;
const PALETTE_ITEM_GAP: f32 = 12.0;
const PALETTE_CELL_INNER_GAP: f32 = 10.0;
/// Fits longest label ("Sidebar Primary Foreground") beside the swatch at 13px.
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
    color: gpui::Hsla,
}

struct PaletteSection {
    title: &'static str,
    rows: Vec<TokenSwatchRow>,
}

struct ModePalette {
    title_text: gpui::Hsla,
    muted_text: gpui::Hsla,
    sections: Vec<PaletteSection>,
}

declare_form! {
    pub struct PalettePanel {
        controls: {},
        args: {
            look: Arc<ShadcnLook>,
            overrides: StudioOverrides,
        },
        fields: {}
    }
}

impl PalettePanel {
    pub fn sync_snapshot(&mut self, look: Arc<ShadcnLook>, overrides: StudioOverrides, cx: &mut Context<Self>) {
        self.look = look;
        self.overrides = overrides;
        cx.notify();
    }
}

impl Render for PalettePanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let mode_tokens = match self.look.mode() {
                ThemeMode::Light => self.look.light_tokens(),
                ThemeMode::Dark => self.look.dark_tokens(),
            };
            let palette = mode_palette(&mode_tokens, &self.overrides.global_color_overrides);
            let heading_style = self.look.typography_role(ShadcnTextRole::H3);
            let body_style = self.look.typography_scale(ShadcnTextSize::Sm);
            let section_style = self.look.typography_scale(ShadcnTextSize::Lg);
            let row_label_style = self.look.typography_scale(ShadcnTextSize::Sm);
            let code_style = self.look.typography_scale(ShadcnTextSize::Sm);

            vstack! {
                gap=18;
                vstack! {
                    gap=4;
                    div().typography_style(heading_style).text_color(chrome.title_text).child("Palette"),
                    div()
                        .typography_style(body_style)
                        .text_color(chrome.muted_text)
                        .child(
                            if self.look.has_css_catalog() {
                                "Theme colors grouped by role"
                            } else {
                                "Native default theme: pick a tweakcn theme in the sidebar for catalog-backed swatches"
                            },
                        ),
                },
                render_palette_sections(palette, &chrome, section_style, row_label_style, code_style),
            }
            .id("theme-studio-palette")
            .size_full()
            .min_h_0()
            .overflow_y_scroll()
            .p(px(24.0))
        })
    }
}

fn mode_palette(mode_tokens: &ShadcnModeTokens, global_overrides: &HashMap<String, gpui::Hsla>) -> ModePalette {
    let palette = mode_tokens.palette;
    let fallback = gpui::hsla(0.0, 0.0, 0.5, 1.0);

    ModePalette {
        title_text: palette.app_foreground,
        muted_text: palette.app_muted_foreground,
        sections: PALETTE_CATEGORIES
            .iter()
            .map(|(title, tokens)| PaletteSection {
                title,
                rows: tokens
                    .iter()
                    .map(|(token, token_label)| TokenSwatchRow {
                        label: token_label,
                        color: token_color_for_mode(mode_tokens, global_overrides, token, fallback),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn token_color_for_mode(
    mode_tokens: &ShadcnModeTokens,
    global_overrides: &HashMap<String, gpui::Hsla>,
    token: &str,
    fallback: gpui::Hsla,
) -> gpui::Hsla {
    let css_name = token_css_name(token);
    global_overrides
        .get(&css_name)
        .copied()
        .or_else(|| mode_tokens.catalog.color(token).ok())
        .unwrap_or(fallback)
}

fn render_palette_sections(
    palette: ModePalette,
    chrome: &LumaChrome,
    section_style: gpui_luma::theme::LumaTextStyle,
    row_label_style: gpui_luma::theme::LumaTextStyle,
    code_style: gpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    let ModePalette { title_text, muted_text, sections } = palette;

    div()
        .flex()
        .flex_col()
        .gap(px(20.0))
        .children(sections.into_iter().map(|section| {
            render_section(section, title_text, muted_text, chrome, section_style, row_label_style, code_style)
        }))
        .into_any_element()
}

fn render_section(
    section: PaletteSection,
    title_text: gpui::Hsla,
    muted_text: gpui::Hsla,
    chrome: &LumaChrome,
    section_style: gpui_luma::theme::LumaTextStyle,
    row_label_style: gpui_luma::theme::LumaTextStyle,
    code_style: gpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    vstack! {
        gap=12;
        div().typography_style(section_style).text_color(title_text).child(section.title),
        div()
            .w_full()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(px(PALETTE_ITEM_GAP))
            .children(section.rows.into_iter().map(|row| render_token_cell(row, title_text, muted_text, chrome, row_label_style, code_style))),
    }
    .into_any_element()
}

fn render_token_cell(
    row: TokenSwatchRow,
    title_text: gpui::Hsla,
    muted_text: gpui::Hsla,
    chrome: &LumaChrome,
    row_label_style: gpui_luma::theme::LumaTextStyle,
    code_style: gpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    hstack! {
        gap=PALETTE_CELL_INNER_GAP align=start;
        div()
            .size(px(PALETTE_SWATCH_SIZE))
            .flex_shrink_0()
            .rounded(px(6.0))
            .bg(row.color)
            .border_1()
            .border_color(chrome.border),
        vstack! {
            gap=2;
            div()
                .typography_style(row_label_style)
                .font_weight(FontWeight::MEDIUM)
                .text_color(title_text)
                .child(row.label),
            div()
                .font_family("Monaco")
                .typography_style(code_style)
                .text_color(muted_text)
                .child(format_hex_color(row.color)),
        }
        .w(px(PALETTE_CELL_TEXT_WIDTH))
        .flex_shrink_0(),
    }
    .w(px(PALETTE_CELL_WIDTH))
    .py(px(4.0))
    .flex_shrink_0()
    .into_any_element()
}
