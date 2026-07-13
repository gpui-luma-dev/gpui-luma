use std::sync::Arc;

use gpui::{AnyElement, Context, FocusHandle, IntoElement, MouseButton, ParentElement, Render, div, prelude::*, px};
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::shell::TitleBar;
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::theme::toggle_shell_theme;

pub trait HasShellTheme {
    fn look(&self) -> &Arc<ShadcnLook>;
}

pub fn wrap_content_pane(
    content: AnyElement,
    pane_focus: FocusHandle,
    look: &ShadcnLook,
    sans_family: String,
) -> AnyElement {
    let focus = pane_focus.clone();
    let chrome = look.chrome();

    div()
        .size_full()
        .font_family(sans_family)
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

pub fn render_title_bar<T: Render + HasShellTheme>(title: &str, app: &T, cx: &mut Context<T>) -> impl IntoElement {
    let look = app.look();
    let chrome = look.chrome();
    let sans_family = look.mode_tokens().typography.font.sans.family.clone();
    let toggle_icon = match look.mode() {
        ThemeMode::Light => LucideIcon::Moon,
        ThemeMode::Dark => LucideIcon::Sun,
    };

    TitleBar::new().background_color(chrome.panel_background).border_color(chrome.border).child(
        div()
            .id("shell-titlebar")
            .h_full()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_2()
            .text_color(chrome.title_text)
            .font_family(sans_family)
            .child(div().child(title.to_string()))
            .child(
                div()
                    .id("shell-titlebar-theme-toggle")
                    .size(px(28.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(6.0))
                    .font_family("lucide")
                    .text_size(px(14.0))
                    .line_height(px(14.0))
                    .text_color(chrome.title_text)
                    .cursor_pointer()
                    .hover(|style| style.bg(gpui::hsla(0.0, 0.0, 1.0, 0.10)))
                    .active(|style| style.bg(gpui::hsla(0.0, 0.0, 1.0, 0.18)))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .on_click(cx.listener(|this: &mut T, _, _, cx| {
                        toggle_shell_theme(this.look(), cx);
                        cx.notify();
                    }))
                    .child(char::from(toggle_icon).to_string()),
            ),
    )
}

pub fn render_app_root(
    focus_scope: &gpui::FocusHandle,
    look: &ShadcnLook,
    title_bar: impl IntoElement,
    body: impl IntoElement,
) -> impl IntoElement {
    let chrome = look.chrome();
    let sans_family = look.mode_tokens().typography.font.sans.family.clone();

    div()
        .luma_focus_scope(focus_scope)
        .size_full()
        .flex()
        .flex_col()
        .font_family(sans_family)
        .bg(chrome.app_background)
        .child(title_bar)
        .child(div().flex_1().min_h_0().child(body))
}
