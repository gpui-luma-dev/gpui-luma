use gpui::{AnyElement, Context, IntoElement, MouseButton, Render, Window, div, prelude::*, px, rgb};
use gpui_luma::focus::LumaFocusScopeExt;

use super::control::GalleryApp;
impl GalleryApp {
    fn render_sidebar(&self) -> AnyElement {
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
                        div()
                            .text_size(px(12.0))
                            .line_height(px(16.0))
                            .text_color(rgb(0x64748b))
                            .child("Control gallery"),
                    ),
            )
            .child(div().flex_1().min_h(px(0.0)).child(self.nav_view.clone()))
            .into_any_element()
    }
}

impl Render for GalleryApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_scope = self.focus_scope.clone();
        let sidebar = self.render_sidebar();
        let work_content = self.panes.render_selected(&self.nav_selection);

        self.split_view.update(cx, |split_view, _cx| {
            split_view.set_panes_once(sidebar, work_content);
        });

        div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .relative()
            .child(div().absolute().size_full().bg(rgb(0xf8fafc)).on_mouse_down(
                MouseButton::Left,
                move |_event, window, cx| {
                    window.focus(&focus_scope, cx);
                    cx.stop_propagation();
                },
            ))
            .child(self.split_view.clone())
    }
}
