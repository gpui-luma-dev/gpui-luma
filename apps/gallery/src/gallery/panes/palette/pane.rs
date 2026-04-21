use gpui::{AnyElement, FontWeight, Hsla, IntoElement, div, prelude::*, px};
use gpui_luma::theme::{LumaPalette, ThemeTokens};

use crate::gallery::theme::GalleryThemePack;

struct ColorItem {
    label: &'static str,
    color: Hsla,
    reserved: bool,
}

struct PaletteSection {
    title: &'static str,
    items: Vec<ColorItem>,
}

pub(in crate::gallery) fn render(theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();
    let tokens = theme.tokens();
    let sections = palette_sections(&tokens);

    div()
        .size_full()
        .bg(chrome.content_background)
        .text_color(chrome.body_text)
        .p(px(28.0))
        .overflow_hidden()
        .child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .gap(px(18.0))
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
                                .child(format!("{} theme v{}", theme.theme_name(), theme.theme_version())),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.muted_text)
                                .child("* reserved for future SDK components"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(20.0))
                        .children(sections.into_iter().map(|section| render_section(section, theme))),
                ),
        )
        .into_any_element()
}

fn palette_sections(tokens: &ThemeTokens) -> Vec<PaletteSection> {
    let palette = &tokens.palette;

    vec![
        PaletteSection {
            title: "App",
            items: vec![
                item("background", palette.app.background),
                item("foreground", palette.app.foreground),
                item("muted_foreground", palette.app.muted_foreground),
            ],
        },
        PaletteSection {
            title: "Surface",
            items: vec![
                item("panel.background", palette.surface.panel.background),
                item("panel.foreground", palette.surface.panel.foreground),
                item("panel.border", palette.surface.panel.border),
                item("floating.background", palette.surface.floating.background),
                item("floating.foreground", palette.surface.floating.foreground),
                item("floating.border", palette.surface.floating.border),
                item("subtle.background", palette.surface.subtle.background),
                item("subtle.foreground", palette.surface.subtle.foreground),
            ],
        },
        PaletteSection { title: "Action", items: action_items(palette) },
        PaletteSection {
            title: "State",
            items: vec![
                item("hover.background", palette.state.hover.background),
                item("hover.foreground", palette.state.hover.foreground),
                item("pressed.background", palette.state.pressed.background),
                item("selected.background", palette.state.selected.background),
                item("selected.foreground", palette.state.selected.foreground),
                item("disabled.background", palette.state.disabled.background),
                item("disabled.foreground", palette.state.disabled.foreground),
            ],
        },
        PaletteSection {
            title: "Form, Focus, Border",
            items: vec![
                item("input.background", palette.form.input.background),
                reserved_item("input.foreground", palette.form.input.foreground),
                item("input.border", palette.form.input.border),
                reserved_item("input.placeholder", palette.form.input.placeholder),
                item("focus.ring", palette.focus.ring),
                item("border.default", palette.border.default),
                reserved_item("border.strong", palette.border.strong),
            ],
        },
        PaletteSection {
            title: "Navigation",
            items: vec![
                item("background", palette.navigation.background),
                item("foreground", palette.navigation.foreground),
                item("muted_foreground", palette.navigation.muted_foreground),
                item("hover_background", palette.navigation.hover_background),
                item("selected_background", palette.navigation.selected_background),
                item("selected_foreground", palette.navigation.selected_foreground),
                item("border", palette.navigation.border),
                item("focus_ring", palette.navigation.focus_ring),
            ],
        },
        PaletteSection {
            title: "Data",
            items: vec![
                reserved_item("accent_1", palette.data.accent_1),
                reserved_item("accent_2", palette.data.accent_2),
                reserved_item("accent_3", palette.data.accent_3),
                reserved_item("accent_4", palette.data.accent_4),
                reserved_item("accent_5", palette.data.accent_5),
            ],
        },
    ]
}

fn action_items(palette: &LumaPalette) -> Vec<ColorItem> {
    vec![
        item("primary.background", palette.action.primary.background),
        item("primary.foreground", palette.action.primary.foreground),
        item("primary.hover_background", palette.action.primary.hover_background),
        item("primary.pressed_background", palette.action.primary.pressed_background),
        item("secondary.background", palette.action.secondary.background),
        item("secondary.foreground", palette.action.secondary.foreground),
        item("secondary.hover_background", palette.action.secondary.hover_background),
        item("secondary.pressed_background", palette.action.secondary.pressed_background),
        item("danger.background", palette.action.danger.background),
        item("danger.foreground", palette.action.danger.foreground),
        item("danger.hover_background", palette.action.danger.hover_background),
        item("danger.pressed_background", palette.action.danger.pressed_background),
    ]
}

fn item(label: &'static str, color: Hsla) -> ColorItem {
    ColorItem { label, color, reserved: false }
}

fn reserved_item(label: &'static str, color: Hsla) -> ColorItem {
    ColorItem { label, color, reserved: true }
}

fn render_section(section: PaletteSection, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

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
                .children(section.items.into_iter().map(|item| render_color_item(item, theme))),
        )
        .into_any_element()
}

fn render_color_item(item: ColorItem, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();
    let name = if item.reserved {
        format!("{} *", item.label)
    } else {
        item.label.to_string()
    };

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
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.title_text)
                        .child(name),
                )
                .child(
                    div()
                        .truncate()
                        .font_family("Monaco")
                        .text_size(px(11.0))
                        .line_height(px(15.0))
                        .text_color(chrome.muted_text)
                        .child(format_hsla(item.color)),
                ),
        )
        .into_any_element()
}

fn format_hsla(color: Hsla) -> String {
    format!("hsla({:.1} {:.1}% {:.1}% / {:.2})", color.h * 360.0, color.s * 100.0, color.l * 100.0, color.a)
}
