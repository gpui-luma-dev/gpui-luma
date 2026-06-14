use gpui::{AnyElement, Context, Entity, FontWeight, IntoElement, div, prelude::*, px};
use gpui_luma_look_shadcn::ShadcnLook;

pub(super) fn color_gallery_pane(
    title: &'static str,
    description: &'static str,
    content: impl IntoElement,
    look: &ShadcnLook,
) -> AnyElement {
    let chrome = look.chrome();

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
                        .child(
                            div()
                                .text_size(px(20.0))
                                .line_height(px(28.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(chrome.title_text)
                                .child(title),
                        )
                        .child(
                            div()
                                .max_w(px(760.0))
                                .text_size(px(13.0))
                                .line_height(px(18.0))
                                .text_color(chrome.muted_text)
                                .child(description),
                        ),
                )
                .child(content),
        )
        .into_any_element()
}

pub(super) fn demo_section(
    title: &'static str,
    description: &'static str,
    cards: Vec<AnyElement>,
    look: &ShadcnLook,
) -> AnyElement {
    let chrome = look.chrome();

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
                        .text_size(px(15.0))
                        .line_height(px(21.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(title),
                )
                .child(
                    div().text_size(px(12.0)).line_height(px(17.0)).text_color(chrome.muted_text).child(description),
                ),
        )
        .child(div().flex().flex_wrap().items_start().gap(px(16.0)).children(cards))
        .into_any_element()
}

pub(super) fn demo_card(
    title: &'static str,
    description: &'static str,
    width_px: f32,
    content: impl IntoElement,
    look: &ShadcnLook,
) -> AnyElement {
    let chrome = look.chrome();

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
                        .text_size(px(13.0))
                        .line_height(px(18.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(title),
                )
                .child(
                    div().text_size(px(11.0)).line_height(px(16.0)).text_color(chrome.muted_text).child(description),
                ),
        )
        .child(content)
        .into_any_element()
}

pub(super) fn control_label(label: &'static str, look: &ShadcnLook) -> AnyElement {
    div()
        .text_size(px(11.0))
        .line_height(px(16.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(look.chrome().muted_text)
        .child(label)
        .into_any_element()
}

pub(super) fn detail_row(label: &'static str, value: String, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .child(
            div()
                .text_size(px(11.0))
                .line_height(px(16.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(chrome.muted_text)
                .child(label),
        )
        .child(div().text_size(px(11.0)).line_height(px(16.0)).text_color(chrome.body_text).child(value))
        .into_any_element()
}

pub(super) fn notify_control<T: 'static, V: 'static>(entity: &Entity<T>, cx: &mut Context<V>) {
    entity.update(cx, |_, cx| cx.notify());
}
