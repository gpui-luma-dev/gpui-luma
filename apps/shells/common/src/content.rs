use gpui::{AnyElement, Entity, IntoElement, ParentElement, Styled, div, px, rgb};
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma::controls::navigation_sidebar::NavigationSidebar;

pub const ICON_RAIL_COLLAPSED_WIDTH: f32 = 128.0;

pub fn shell_content_pane() -> AnyElement {
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

pub fn inset_content_pane() -> AnyElement {
    div().size_full().rounded(px(16.0)).bg(rgb(0x000000)).into_any_element()
}

pub fn detached_nav_pane() -> AnyElement {
    div()
        .size_full()
        .p(px(10.0))
        .child(div().size_full().rounded(px(16.0)).bg(rgb(0x242835)))
        .into_any_element()
}

pub fn detached_nav_pane_with_sidebar(navigation_sidebar: Entity<NavigationSidebar>) -> AnyElement {
    div()
        .size_full()
        .p(px(10.0))
        .child(div().size_full().rounded(px(16.0)).bg(rgb(0x242835)).overflow_hidden().child(navigation_sidebar))
        .into_any_element()
}

pub fn detached_content_pane(toggle_button: IconButton) -> AnyElement {
    div()
        .size_full()
        .p_3()
        .child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(div().w_full().flex().items_center().child(toggle_button))
                .child(div().flex_1().mt_2().bg(rgb(0x800080)).rounded_md()),
        )
        .into_any_element()
}
