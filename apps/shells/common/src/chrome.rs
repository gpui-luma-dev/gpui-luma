use std::sync::Arc;

use gpui::{AnyElement, Context, FocusHandle, IntoElement, MouseButton, ParentElement, Render, div, prelude::*, px};
use gpui_luma::controls::command::button::{ButtonRenderModel, ControlPresenter};
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::shell::TitleBar;
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};
use lucide_icons::Icon as LucideIcon;

use crate::theme::toggle_shell_theme;

pub trait HasShellTheme {
    fn look(&self) -> &Arc<ShadcnLook>;
    fn theme_toggle_button(&self) -> IconButton;
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

pub fn spawn_theme_toggle_button<T: 'static>(
    id: &'static str,
    look: &Arc<ShadcnLook>,
    cx: &mut Context<T>,
) -> IconButton {
    look.content_only_icon_button(id, theme_toggle_icon(look.mode())).spawn(cx)
}

pub fn sync_theme_toggle_button<T: 'static>(button: &IconButton, look: &ShadcnLook, cx: &mut Context<T>) {
    button.update(cx, |button, cx| {
        button.set_presenter(theme_toggle_presenter(theme_toggle_icon(look.mode())), cx);
    });
}

pub fn handle_theme_toggle<T: Render + HasShellTheme>(app: &mut T, cx: &mut Context<T>) {
    toggle_shell_theme(app.look(), cx);
    sync_theme_toggle_button(&app.theme_toggle_button(), app.look(), cx);
    cx.notify();
}

pub fn render_title_bar<T: Render + HasShellTheme>(title: &str, app: &T, _cx: &mut Context<T>) -> impl IntoElement {
    let look = app.look();
    let chrome = look.chrome();
    let sans_family = look.mode_tokens().typography.font.sans.family.clone();

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
            .child(app.theme_toggle_button()),
    )
}

fn theme_toggle_icon(mode: ThemeMode) -> LucideIcon {
    match mode {
        ThemeMode::Light => LucideIcon::Moon,
        ThemeMode::Dark => LucideIcon::Sun,
    }
}

fn theme_toggle_presenter(icon: LucideIcon) -> ControlPresenter<ButtonRenderModel<()>> {
    Arc::new(move |_, _| {
        div()
            .font_family("lucide")
            .text_size(px(14.0))
            .child(char::from(icon).to_string())
            .into_any_element()
    })
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
