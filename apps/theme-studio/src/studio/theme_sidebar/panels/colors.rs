use gpui::{AnyElement, IntoElement, div, prelude::*, px};
use gpui_luma::vstack;

use super::super::ThemeSidebar;
use super::super::parsing::effective_token_color;
use crate::studio::token_color_row::token_color_row;

pub(in crate::studio::theme_sidebar) fn render_colors_panel(sidebar: &ThemeSidebar) -> AnyElement {
    let chrome = sidebar.vm.look.chrome();
    let has_catalog = sidebar.vm.look.has_css_catalog();

    let mut body = div().flex().flex_col().w_full().gap(px(10.0));

    if !has_catalog {
        body = body.child(
            div()
                .text_xs()
                .text_color(chrome.muted_text)
                .child("Native default theme: pick a tweakcn theme above for catalog-backed swatches."),
        );
    }

    body.child(div().w_full().child(sidebar.token_accordion.clone())).into_any_element()
}

pub(in crate::studio::theme_sidebar) fn category_token_content(
    sidebar: &ThemeSidebar,
    tokens: &[(&str, &str)],
) -> AnyElement {
    let theme = &sidebar.vm.look;
    let overrides = &sidebar.vm.global_overrides;
    let chrome = theme.chrome();
    let row_label_typography = theme.mode_tokens().typography.text.label;

    let mut rows = vstack! {
        gap=6;
    };

    for (token, label) in tokens {
        let Some(field) = sidebar.vm.token_fields.get(*token) else {
            continue;
        };
        let color = effective_token_color(theme, overrides, token);
        let label = label.to_string();
        let field = field.clone();

        rows = vstack! {
            gap=6;
            rows,
            token_color_row(label, color, field, &chrome, &row_label_typography),
        };
    }

    rows.into_any_element()
}
