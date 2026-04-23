use gpui::{AnyElement, Context, FocusHandle, IntoElement, MouseButton, Render, Window, div, prelude::*};
use gpui_luma::controls::split_view::render_pane;
use gpui_luma::focus::LumaFocusScopeExt;

use super::control::GalleryApp;

impl Render for GalleryApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane_focus = self.pane_focus.clone();
        let navigation_sidebar = self.navigation_sidebar.clone();
        let panes = self.panes.clone();
        let nav_selection = self.nav_selection.clone();
        let chrome = self.theme.chrome();

        self.split_view.update(cx, |split_view, _cx| {
            split_view.set_panes(
                render_pane(move || navigation_sidebar.clone()),
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
