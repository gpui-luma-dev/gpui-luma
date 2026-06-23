mod template_pipeline;
mod theme_context;

pub(in crate::gallery) mod inspector;

pub(in crate::gallery) use theme_context::render_sparse_catalog_callout;

pub(in crate::gallery) use template_pipeline::{
    render_combobox_popup_preview_from_templates, render_search_selector_popup_preview_from_templates,
};

use gpui::{AnyElement, Context, Entity, Hsla, IntoElement, div, prelude::*, px};
use gpui_luma::DockPanel;
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextRole, ShadcnTextSize};

use crate::gallery::control::GalleryApp;

pub(super) fn gallery_pane(title: &'static str, content: AnyElement, look: &ShadcnLook) -> AnyElement {
    gallery_pane_with_description(title, None, content, look)
}

pub(super) fn gallery_pane_with_description(
    title: &'static str,
    description: Option<&'static str>,
    content: AnyElement,
    look: &ShadcnLook,
) -> AnyElement {
    let chrome = look.chrome();

    div()
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .overflow_hidden()
        .bg(chrome.content_background)
        .p(px(28.0))
        .child(render_pane_header(title, description, look, chrome.title_text, chrome.muted_text))
        .child(render_centered_pane_body(content))
        .into_any_element()
}

pub(super) fn gallery_pane_with_inspector(
    title: &'static str,
    content: AnyElement,
    inspector: impl IntoElement,
    look: &ShadcnLook,
) -> AnyElement {
    gallery_pane_with_inspector_description(title, None, content, inspector, look)
}

pub(super) fn gallery_pane_scrollable_with_inspector(
    title: &'static str,
    content: AnyElement,
    inspector: impl IntoElement,
    look: &ShadcnLook,
) -> AnyElement {
    gallery_pane_scrollable_with_inspector_description(title, None, content, inspector, look)
}

pub(super) fn gallery_pane_scrollable_with_inspector_description(
    title: &'static str,
    description: Option<&'static str>,
    content: AnyElement,
    inspector: impl IntoElement,
    look: &ShadcnLook,
) -> AnyElement {
    let chrome = look.chrome();

    div()
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .overflow_hidden()
        .bg(chrome.content_background)
        .p(px(28.0))
        .child(render_pane_header(title, description, look, chrome.title_text, chrome.muted_text))
        .child(
            div().min_h(px(0.0)).flex_1().child(
                DockPanel::new()
                    .right(div().h_full().min_h(px(0.0)).flex().child(div().w(px(28.0)).h_full()).child(
                        div().w(px(620.0)).min_w(px(620.0)).min_h(px(0.0)).h_full().flex().flex_col().child(inspector),
                    ))
                    .fill(
                        div()
                            .min_h(px(0.0))
                            .flex()
                            .items_stretch()
                            .justify_center()
                            .child(render_scrollable_pane_body(title, content)),
                    ),
            ),
        )
        .into_any_element()
}

pub(super) fn gallery_pane_with_inspector_description(
    title: &'static str,
    description: Option<&'static str>,
    content: AnyElement,
    inspector: impl IntoElement,
    look: &ShadcnLook,
) -> AnyElement {
    let chrome = look.chrome();

    div()
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .overflow_hidden()
        .bg(chrome.content_background)
        .p(px(28.0))
        .child(render_pane_header(title, description, look, chrome.title_text, chrome.muted_text))
        .child(
            div().min_h(px(0.0)).flex_1().child(
                DockPanel::new()
                    .right(div().h_full().min_h(px(0.0)).flex().child(div().w(px(28.0)).h_full()).child(
                        div().w(px(620.0)).min_w(px(620.0)).min_h(px(0.0)).h_full().flex().flex_col().child(inspector),
                    ))
                    .fill(
                        div()
                            .min_h(px(0.0))
                            .flex()
                            .items_stretch()
                            .justify_center()
                            .child(render_centered_pane_body(content)),
                    ),
            ),
        )
        .into_any_element()
}

fn render_scrollable_pane_body(title: &'static str, content: AnyElement) -> AnyElement {
    div()
        .w_full()
        .min_h(px(0.0))
        .flex_1()
        .child(
            div()
                .id(format!("{title}-scroll"))
                .size_full()
                .overflow_y_scroll()
                .pt(px(18.0))
                .pb(px(24.0))
                .child(content),
        )
        .into_any_element()
}

fn render_centered_pane_body(content: AnyElement) -> AnyElement {
    div()
        .min_w(px(0.0))
        .min_h(px(0.0))
        .flex_1()
        .flex()
        .items_center()
        .justify_center()
        .child(div().relative().flex().flex_col().items_center().justify_center().gap_4().occlude().child(content))
        .into_any_element()
}

fn render_pane_header(
    title: &'static str,
    description: Option<&'static str>,
    look: &ShadcnLook,
    title_color: Hsla,
    description_color: Hsla,
) -> AnyElement {
    let title_style = look.typography_role(ShadcnTextRole::H3);
    let description_style = look.typography_scale(ShadcnTextSize::Sm);

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(div().typography_style(title_style).text_color(title_color).child(title))
        .when_some(description, |header, description| {
            header.child(
                div()
                    .max_w(px(760.0))
                    .typography_style(description_style)
                    .text_color(description_color)
                    .child(description),
            )
        })
        .into_any_element()
}

pub(super) fn notify_entity<T: 'static>(entity: &Entity<T>, cx: &mut Context<GalleryApp>) {
    entity.update(cx, |_, cx| cx.notify());
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

pub(in crate::gallery::panes) fn format_hex_color(color: Hsla) -> String {
    let (r, g, b) = hsla_to_rgb8(color);
    format!("#{r:02x}{g:02x}{b:02x}")
}

pub(in crate::gallery::panes) fn format_inspector_rgba(color: Hsla) -> String {
    let (r, g, b) = hsla_to_rgb8(color);
    format!("{r}, {g}, {b}, {}", compact_alpha(color.a))
}

pub(in crate::gallery::panes) fn format_inspector_hsl(color: Hsla) -> String {
    format!(
        "{} {}% {}%",
        rounded_channel(color.h * 360.0),
        rounded_channel(color.s * 100.0),
        rounded_channel(color.l * 100.0)
    )
}

fn hsla_to_rgb8(color: Hsla) -> (u8, u8, u8) {
    let h = color.h.fract() * 6.0;
    let s = color.s.clamp(0.0, 1.0);
    let l = color.l.clamp(0.0, 1.0);

    if s <= f32::EPSILON {
        let gray = (l * 255.0).round() as u8;
        return (gray, gray, gray);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let r = hue_to_channel(p, q, h + 2.0);
    let g = hue_to_channel(p, q, h);
    let b = hue_to_channel(p, q, h - 2.0);
    ((r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8)
}

fn hue_to_channel(p: f32, q: f32, t: f32) -> f32 {
    let mut t = t;
    if t < 0.0 {
        t += 6.0;
    }
    if t >= 6.0 {
        t -= 6.0;
    }
    if t < 1.0 {
        p + (q - p) * t
    } else if t < 3.0 {
        q
    } else if t < 4.0 {
        p + (q - p) * (4.0 - t)
    } else {
        p
    }
}

fn rounded_channel(value: f32) -> i32 {
    value.round() as i32
}

fn compact_alpha(value: f32) -> String {
    let formatted = format!("{value:.3}");
    formatted.trim_end_matches('0').trim_end_matches('.').to_string()
}
