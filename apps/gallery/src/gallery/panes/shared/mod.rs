mod template_pipeline;
mod theme_context;

pub(in crate::gallery) use theme_context::render_sparse_catalog_callout;

pub(in crate::gallery) use template_pipeline::{
    render_combobox_popup_preview_from_templates, render_search_selector_popup_preview_from_templates,
};

use gpui::{AnyElement, Context, Entity, FontWeight, Hsla, IntoElement, div, prelude::*, px};
use gpui_luma::theme::{ThemePartUsage};
use gpui_luma_look_shadcn::{ShadcnLook, all_shadcn_theme_usages};

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
    look: &ShadcnLook,
) -> AnyElement {
    gallery_pane_with_usage_description(title, None, usage_component, content, look)
}

pub(super) fn gallery_pane_with_usage_top_aligned(
    title: &'static str,
    usage_component: &'static str,
    content: AnyElement,
    look: &ShadcnLook,
) -> AnyElement {
    render_gallery_pane_with_usage_descriptions(title, None, &[usage_component], content, look, true, false)
}

pub(super) fn gallery_pane_with_usage_description_scrollable(
    title: &'static str,
    description: Option<&'static str>,
    usage_component: &'static str,
    content: AnyElement,
    look: &ShadcnLook,
) -> AnyElement {
    render_gallery_pane_with_usage_descriptions(
        title,
        description,
        &[usage_component],
        content,
        look,
        true,
        true,
    )
}

pub(super) fn gallery_pane_with_usage_description(
    title: &'static str,
    description: Option<&'static str>,
    usage_component: &'static str,
    content: AnyElement,
    look: &ShadcnLook,
) -> AnyElement {
    gallery_pane_with_usage_descriptions(title, description, &[usage_component], content, look)
}

pub(super) fn gallery_pane_with_usage_descriptions(
    title: &'static str,
    description: Option<&'static str>,
    usage_components: &[&'static str],
    content: AnyElement,
    look: &ShadcnLook,
) -> AnyElement {
    render_gallery_pane_with_usage_descriptions(
        title,
        description,
        usage_components,
        content,
        look,
        false,
        false,
    )
}

fn render_gallery_pane_with_usage_descriptions(
    title: &'static str,
    description: Option<&'static str>,
    usage_components: &[&'static str],
    content: AnyElement,
    look: &ShadcnLook,
    top_aligned: bool,
    scrollable: bool,
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
        .child(render_pane_header(title, description, chrome.title_text, chrome.muted_text))
        .child(
            div()
                .min_h(px(0.0))
                .flex_1()
                .flex()
                .items_stretch()
                .justify_center()
                .gap(px(28.0))
                .child(render_gallery_pane_content_column(content, top_aligned, scrollable))
                .child(div().h_full().flex().items_stretch().child(render_usage_panels(usage_components, look))),
        )
        .into_any_element()
}

fn render_gallery_pane_content_column(content: AnyElement, top_aligned: bool, scrollable: bool) -> AnyElement {
    let mut column = div()
        .min_w(px(0.0))
        .min_h(px(0.0))
        .h_full()
        .flex_1()
        .flex()
        .flex_col()
        .when(scrollable, |this| this.overflow_hidden().items_stretch())
        .when(!scrollable, |this| this.items_center())
        .when(!top_aligned && !scrollable, |column| column.justify_center())
        .when(top_aligned || scrollable, |column| column.justify_start())
        .gap_4()
        .occlude();

    if scrollable {
        column = column
            .child(div().id("gallery-pane-scroll").w_full().h_full().min_h(px(0.0)).overflow_y_scroll().child(content));
    } else {
        column = column.child(content);
    }

    column.into_any_element()
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

fn render_usage_panels(components: &[&'static str], look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

    let panel = div()
        .id("theme-usage")
        .w(px(390.0))
        .h_full()
        .min_h(px(0.0))
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
        );

    panel
        .children(components.iter().copied().map(|component| render_usage_component_section(component, look)))
        .into_any_element()
}

fn render_usage_component_section(component: &'static str, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    let usage = all_shadcn_theme_usages().iter().copied().find(|usage| usage.label == component);
    let parts = usage.map(|usage| usage.parts).unwrap_or(&[]);

    div()
        .id(format!("{component}-theme-usage"))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .text_size(px(12.0))
                .line_height(px(17.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.title_text)
                .child(component),
        )
        .children(parts.iter().map(|part| render_usage_part(part, look)))
        .when(usage.is_none(), |section| {
            section.child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(17.0))
                    .text_color(chrome.muted_text)
                    .child("No Shadcn theme usage metadata registered."),
            )
        })
        .into_any_element()
}

fn render_usage_part(part: &ThemePartUsage, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    let color = resolve_shadcn_token_color(look, part.token);

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
                .child(format!("--{token}", token = part.token)),
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

fn resolve_shadcn_token_color(look: &ShadcnLook, token: &str) -> Option<Hsla> {
    look.token_color(token).ok()
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
