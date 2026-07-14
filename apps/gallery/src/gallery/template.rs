use std::sync::Arc;

use gpui::{AnyElement, Context, FocusHandle, IntoElement, MouseButton, Render, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{ButtonRenderModel, ControlPresenter};
use gpui_luma::controls::split_view::render_pane;
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::shell::TitleBar;
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use super::control::GalleryApp;

impl Render for GalleryApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane_focus = self.pane_focus.clone();
        let navigation_sidebar = self.navigation_sidebar.clone();
        let panes = self.panes.clone();
        let nav_selection = self.nav_selection.clone();
        let needs_inspector_refresh = self.last_inspector_refresh.as_deref() != Some(nav_selection.as_str());
        let chrome = self.look.chrome();
        let sans_family = self.look.mode_tokens().typography.font.sans.family.clone();
        let content_sans_family = sans_family.clone();

        if needs_inspector_refresh {
            let selection = self.nav_selection.clone();
            self.last_inspector_refresh = Some(selection.clone());
            cx.on_next_frame(window, move |this, window, cx| {
                this.panes.notify_selected_controls(&selection, window, cx);
            });
        }

        self.split_view.update(cx, |split_view, cx| {
            split_view.set_panes(
                render_pane(move || navigation_sidebar.clone()),
                render_pane(move || {
                    render_content_pane(
                        panes.render_selected(&nav_selection),
                        pane_focus.clone(),
                        &panes.look,
                        content_sans_family.clone(),
                    )
                }),
                cx,
            );
        });

        let title_bar = TitleBar::new().background_color(chrome.panel_background).border_color(chrome.border).child(
            div()
                .id("gallery-titlebar")
                .h_full()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_2()
                .text_color(chrome.title_text)
                .font_family(sans_family.clone())
                .child(div().child("GPUI-Luma Gallery"))
                .child(self.theme_toggle_button.clone()),
        );

        div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .flex()
            .flex_col()
            .font_family(sans_family)
            .bg(chrome.app_background)
            .child(title_bar)
            .child(div().flex_1().min_h_0().child(self.split_view.clone()))
    }
}

pub(super) fn theme_toggle_icon(mode: ThemeMode) -> LucideIcon {
    match mode {
        ThemeMode::Light => LucideIcon::Moon,
        ThemeMode::Dark => LucideIcon::Sun,
    }
}

pub(super) fn theme_toggle_presenter(icon: LucideIcon) -> ControlPresenter<ButtonRenderModel<()>> {
    Arc::new(move |_, _| {
        div()
            .font_family("lucide")
            .text_size(px(14.0))
            .child(char::from(icon).to_string())
            .into_any_element()
    })
}

fn render_content_pane(
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
