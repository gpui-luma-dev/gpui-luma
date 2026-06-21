use gpui::{AnyElement, Context, Entity, FontWeight, IntoElement, div, prelude::*, px};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextRole, ShadcnTextSize};

pub(in crate::gallery::panes) fn color_gallery_pane(
    title: &'static str,
    description: &'static str,
    content: impl IntoElement,
    look: &ShadcnLook,
) -> AnyElement {
    let chrome = look.chrome();
    let title_style = look.typography_role(ShadcnTextRole::H3);
    let description_style = look.typography_scale(ShadcnTextSize::Sm);

    div()
        .size_full()
        .overflow_hidden()
        .bg(chrome.content_background)
        .p(px(28.0))
        .child(
            div()
                .id(format!("{}-color-pane", title))
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
                        .child(div().typography_style(title_style).text_color(chrome.title_text).child(title))
                        .child(
                            div()
                                .max_w(px(760.0))
                                .typography_style(description_style)
                                .text_color(chrome.muted_text)
                                .child(description),
                        ),
                )
                .child(content),
        )
        .into_any_element()
}

pub(in crate::gallery::panes) fn demo_section(
    title: &'static str,
    description: &'static str,
    cards: Vec<AnyElement>,
    look: &ShadcnLook,
) -> AnyElement {
    let chrome = look.chrome();
    let title_style = look.typography_role(ShadcnTextRole::H4);
    let description_style = look.typography_scale(ShadcnTextSize::Sm);

    div()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .typography_style(title_style)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(title),
                )
                .child(div().typography_style(description_style).text_color(chrome.muted_text).child(description)),
        )
        .child(div().flex().flex_wrap().items_start().gap(px(16.0)).children(cards))
        .into_any_element()
}

pub(in crate::gallery::panes) fn demo_card(
    title: &'static str,
    description: &'static str,
    width_px: f32,
    content: impl IntoElement,
    look: &ShadcnLook,
) -> AnyElement {
    let chrome = look.chrome();
    let title_style = look.typography_scale(ShadcnTextSize::Sm);
    let description_style = look.typography_scale(ShadcnTextSize::Xs);

    div()
        .w(px(width_px))
        .max_w_full()
        .min_h(px(180.0))
        .flex()
        .flex_col()
        .gap(px(14.0))
        .rounded(px(16.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.panel_background)
        .p(px(18.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(3.0))
                .child(
                    div()
                        .typography_style(title_style)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(title),
                )
                .child(div().typography_style(description_style).text_color(chrome.muted_text).child(description)),
        )
        .child(content)
        .into_any_element()
}

pub(in crate::gallery::panes) fn control_label(label: &'static str, look: &ShadcnLook) -> AnyElement {
    div()
        .typography_style(look.typography_scale(ShadcnTextSize::Xs))
        .font_weight(FontWeight::MEDIUM)
        .text_color(look.chrome().muted_text)
        .child(label)
        .into_any_element()
}

pub(in crate::gallery::panes) fn detail_row(label: &'static str, value: String, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    let label_style = look.typography_scale(ShadcnTextSize::Xs);

    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .child(
            div()
                .typography_style(label_style)
                .font_weight(FontWeight::MEDIUM)
                .text_color(chrome.muted_text)
                .child(label),
        )
        .child(div().typography_style(label_style).text_color(chrome.body_text).child(value))
        .into_any_element()
}

pub(in crate::gallery::panes) fn notify_control<T: 'static, V: 'static>(entity: &Entity<T>, cx: &mut Context<V>) {
    entity.update(cx, |_, cx| cx.notify());
}
