use gpui::{AnyElement, Context, FocusHandle, IntoElement, MouseButton, Render, Window, div, prelude::*, px};
use gpui_luma::controls::split_view::render_pane;
use gpui_luma::focus::LumaFocusScopeExt;

use super::control::GalleryApp;

impl Render for GalleryApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane_focus = self.pane_focus.clone();
        let navigation_sidebar = self.navigation_sidebar.clone();
        let theme_toggle = self.theme_toggle.clone();
        let theme = self.theme.clone();
        let panes = self.panes.clone();
        let nav_selection = self.nav_selection.clone();
        let chrome = self.theme.chrome();

        self.split_view.update(cx, |split_view, _cx| {
            split_view.set_panes(
                render_pane(move || render_sidebar(navigation_sidebar.clone(), theme_toggle.clone(), &theme)),
                render_pane(move || {
                    render_content_pane(panes.render_selected(&nav_selection), pane_focus.clone(), &panes.theme)
                }),
            );
        });

        div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .relative()
            .bg(chrome.app_background)
            .child(self.split_view.clone())
    }
}

fn render_sidebar(
    navigation_sidebar: gpui::Entity<gpui_luma::controls::navigation_sidebar::NavigationSidebar>,
    theme_toggle: gpui::Entity<gpui_luma::controls::toggle_button::ToggleButton>,
    theme: &crate::gallery::theme::GalleryThemePack,
) -> AnyElement {
    let chrome = theme.chrome();

    div()
        .size_full()
        .flex()
        .flex_col()
        .gap_3()
        .p_3()
        .bg(chrome.sidebar_background)
        .text_color(chrome.body_text)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .pb_2()
                .child(div().text_size(px(14.0)).line_height(px(18.0)).text_color(chrome.title_text).child("GPUI-Luma"))
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(chrome.muted_text)
                        .child("Control gallery"),
                ),
        )
        .child(div().flex_1().min_h(px(0.0)).child(navigation_sidebar))
        .child(theme_toggle)
        .into_any_element()
}

fn render_content_pane(
    content: AnyElement,
    pane_focus: FocusHandle,
    theme: &crate::gallery::theme::GalleryThemePack,
) -> AnyElement {
    let focus = pane_focus.clone();
    let chrome = theme.chrome();

    div()
        .size_full()
        .bg(chrome.content_background)
        .track_focus(&pane_focus)
        .capture_any_mouse_down(move |event, window, cx| {
            if event.button == MouseButton::Left {
                window.focus(&focus, cx);
            }
        })
        .child(content)
        .into_any_element()
}
