use gpui::{AnyElement, Context, FocusHandle, IntoElement, MouseButton, Render, Window, div, prelude::*, px, rgb};
use gpui_luma::controls::split_view::render_pane;
use gpui_luma::focus::LumaFocusScopeExt;

use super::control::GalleryApp;

impl Render for GalleryApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane_focus = self.pane_focus.clone();
        let nav_view = self.nav_view.clone();
        let panes = self.panes.clone();
        let nav_selection = self.nav_selection.clone();

        self.split_view.update(cx, |split_view, _cx| {
            split_view.set_panes(
                render_pane(move || render_sidebar(nav_view.clone())),
                render_pane(move || render_content_pane(panes.render_selected(&nav_selection), pane_focus.clone())),
            );
        });

        div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .relative()
            .bg(rgb(0xf8fafc))
            .child(self.split_view.clone())
    }
}

fn render_sidebar(nav_view: gpui::Entity<gpui_luma::controls::nav_view::NavView>) -> AnyElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .gap_3()
        .p_3()
        .bg(rgb(0xffffff))
        .text_color(rgb(0x334155))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .pb_2()
                .child(div().text_size(px(14.0)).line_height(px(18.0)).text_color(rgb(0x0f172a)).child("GPUI-Luma"))
                .child(
                    div().text_size(px(12.0)).line_height(px(16.0)).text_color(rgb(0x64748b)).child("Control gallery"),
                ),
        )
        .child(div().flex_1().min_h(px(0.0)).child(nav_view))
        .into_any_element()
}

fn render_content_pane(content: AnyElement, pane_focus: FocusHandle) -> AnyElement {
    let focus = pane_focus.clone();

    div()
        .size_full()
        .track_focus(&pane_focus)
        .capture_any_mouse_down(move |event, window, cx| {
            if event.button == MouseButton::Left {
                window.focus(&focus, cx);
            }
        })
        .child(content)
        .into_any_element()
}
