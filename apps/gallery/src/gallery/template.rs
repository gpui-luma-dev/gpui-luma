use gpui::{AnyElement, Context, FocusHandle, IntoElement, MouseButton, Render, Window, div, prelude::*, px};
use gpui_luma::controls::split_view::render_pane;
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::shell::TitleBar;
use gpui_luma::theme::ThemeMode;
use lucide_icons::Icon as LucideIcon;

use super::control::GalleryApp;

impl Render for GalleryApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane_focus = self.pane_focus.clone();
        let navigation_sidebar = self.navigation_sidebar.clone();
        let panes = self.panes.clone();
        let nav_selection = self.nav_selection.clone();
        let chrome = self.theme.chrome();
        let active_mode = self.theme.mode();
        let toggle_icon = match active_mode {
            ThemeMode::Light => LucideIcon::Moon,
            ThemeMode::Dark => LucideIcon::Sun,
        };

        self.split_view.update(cx, |split_view, _cx| {
            split_view.set_panes(
                render_pane(move || navigation_sidebar.clone()),
                render_pane(move || {
                    render_content_pane(panes.render_selected(&nav_selection), pane_focus.clone(), &panes.theme)
                }),
            );
        });

        let title_bar = TitleBar::new().child(
            div()
                .id("gallery-titlebar")
                .h_full()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_2()
                .child(div().child("GPUI-Luma Gallery"))
                .child(
                    div()
                        .id("gallery-titlebar-theme-toggle")
                        .size(px(28.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(6.0))
                        .font_family("lucide")
                        .text_size(px(14.0))
                        .line_height(px(14.0))
                        .text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                        .cursor_pointer()
                        .hover(|style| style.bg(gpui::hsla(0.0, 0.0, 1.0, 0.10)))
                        .active(|style| style.bg(gpui::hsla(0.0, 0.0, 1.0, 0.18)))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.theme.toggle_mode();
                            tracing::info!("title bar theme toggled");
                            cx.notify();
                        }))
                        .child(char::from(toggle_icon).to_string()),
                ),
        );

        div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .flex()
            .flex_col()
            .bg(chrome.app_background)
            .child(title_bar)
            .child(div().flex_1().min_h_0().child(self.split_view.clone()))
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
