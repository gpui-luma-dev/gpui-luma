use std::sync::Arc;

use gpui::{AnyElement, Entity, IntoElement, ParentElement, Pixels, SharedString, Styled, div, px, rgb};
use gpui_luma::controls::command::button::ButtonEvent;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::split_view::{SplitView, SplitViewSeparatorVisibility};
use gpui_luma::controls::switch::Switch;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

pub(super) const DEMO_WIDTH: f32 = 820.0;
pub(super) const DEMO_HEIGHT: f32 = 460.0;
pub(super) const ICON_RAIL_COLLAPSED_WIDTH: f32 = 128.0;

pub(super) fn demo_frame(title: impl Into<SharedString>, body: impl IntoElement) -> AnyElement {
    let title = title.into();
    div()
        .w_full()
        .min_w_0()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_sm().child(title))
        .child(div().w_full().child(body))
        .into_any_element()
}

pub(super) fn mockup_shell(body: impl IntoElement, canvas_width: Pixels, canvas_height: Pixels) -> AnyElement {
    let radius = px(10.0);
    let border = rgb(0x101010);

    div()
        .w(canvas_width)
        .flex()
        .flex_col()
        .child(
            div()
                .w_full()
                .h(px(27.0))
                .px_3()
                .flex()
                .items_center()
                .gap_2()
                .bg(rgb(0x000000))
                .rounded_tl(radius)
                .rounded_tr(radius)
                .border_1()
                .border_color(border)
                .child(div().size(px(12.0)).rounded_full().bg(rgb(0xFFFFFF)))
                .child(div().size(px(12.0)).rounded_full().bg(rgb(0xFFFFFF)))
                .child(div().size(px(12.0)).rounded_full().bg(rgb(0xFFFFFF))),
        )
        .child(
            div()
                .w_full()
                .h(canvas_height)
                .bg(rgb(0x000000))
                .border_1()
                .border_t_0()
                .border_color(border)
                .rounded_bl(radius)
                .rounded_br(radius)
                .overflow_hidden()
                .child(body),
        )
        .into_any_element()
}

pub(super) fn content_pane_with_panel(panel: impl IntoElement) -> AnyElement {
    div().size_full().p_3().child(panel).into_any_element()
}

pub(super) fn shell_content_pane() -> AnyElement {
    div()
        .size_full()
        .bg(rgb(0x000000))
        .p(px(10.0))
        .child(
            div()
                .size_full()
                .flex()
                .child(div().h_full().flex_1().rounded(px(16.0)).bg(rgb(0x242835)))
                .child(div().w(px(20.0))),
        )
        .into_any_element()
}

pub(super) fn nav_pane_mock() -> AnyElement {
    div().size_full().bg(rgb(0x242835)).into_any_element()
}

pub(super) fn inset_content_pane() -> AnyElement {
    div().size_full().rounded(px(16.0)).bg(rgb(0x000000)).into_any_element()
}

pub(super) fn separator_visibility_label(visibility: SplitViewSeparatorVisibility) -> &'static str {
    match visibility {
        SplitViewSeparatorVisibility::Always => "always",
        SplitViewSeparatorVisibility::Hover => "hover",
    }
}

pub(super) fn apply_separator_visibility_toggle(
    split_view: &Entity<SplitView>,
    switch: &Switch,
    event: &ButtonEvent,
    cx: &mut gpui::Context<GalleryApp>,
) {
    if !matches!(event, ButtonEvent::Click) {
        return;
    }

    split_view.update(cx, |split_view, cx| {
        let next = match split_view.separator_visibility() {
            SplitViewSeparatorVisibility::Always => SplitViewSeparatorVisibility::Hover,
            SplitViewSeparatorVisibility::Hover => SplitViewSeparatorVisibility::Always,
        };
        split_view.set_separator_visibility(next, cx);
    });
    switch.update(cx, |button, cx| {
        let checked = matches!(split_view.read(cx).separator_visibility(), SplitViewSeparatorVisibility::Always);
        if *button.data() != checked {
            button.set_data(checked, cx);
        }
    });
}

pub(super) fn separator_switch(
    look: &Arc<ShadcnLook>,
    id: impl Into<SharedString>,
    initial_visibility: SplitViewSeparatorVisibility,
    cx: &mut gpui::Context<GalleryApp>,
) -> Switch {
    let id = id.into();
    look.secondary_switch(id)
        .with_data(matches!(initial_visibility, SplitViewSeparatorVisibility::Always))
        .content(|_, _| div().child("Separator Always Visible").into_any_element())
        .spawn(cx)
}
