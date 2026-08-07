use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, FocusHandle, IntoElement, ParentElement, Render, Styled, Subscription, Window, div,
    px, rgb,
};
use gpui_luma::controls::command::button::ButtonEvent;
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma::controls::sidebar::SidebarControl;
use gpui_luma::controls::split_view::{SplitView, SplitViewSeparatorVisibility, render_pane};
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_shell_common::{
    chrome::{
        HasShellTheme, handle_theme_toggle, render_app_root, render_title_bar, spawn_theme_toggle_button,
        wrap_content_pane,
    },
    content, nav_sample,
    split_sync::wire_split_nav_sync,
    theme::{ShellThemeChoice, sync_color_control_theme},
};

pub struct InsetShellApp {
    focus_scope: FocusHandle,
    pane_focus: FocusHandle,
    look: Arc<ShadcnLook>,
    split_view: Entity<SplitView>,
    sidebar: Entity<SidebarControl>,
    theme_toggle_button: IconButton,
    _subscriptions: Vec<Subscription>,
}

impl InsetShellApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, theme_choice: ShellThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        let pane_focus = cx.focus_handle().tab_stop(true);
        window.focus(&focus_scope, cx);
        let look = theme_choice.shadcn_look();
        look.set_mode(ThemeMode::Dark);
        sync_color_control_theme(&look);

        let sidebar = nav_sample::spawn_properties_sidebar(look.clone(), "shell-nav", cx);
        let split_view = look
            .split_view("shell-inset")
            .sidebar_width(px(400.0))
            .sidebar_min_width(px(320.0))
            .sidebar_collapsed_width(px(0.0))
            .separator_visibility(SplitViewSeparatorVisibility::Hover)
            .spawn(cx);
        let theme_toggle_button = spawn_theme_toggle_button("shell-titlebar-theme-toggle-inset", &look, cx);

        let mut subscriptions = Vec::new();
        wire_split_nav_sync(&mut subscriptions, split_view.clone(), sidebar.clone(), cx);
        subscriptions.push(cx.subscribe(&theme_toggle_button, |this, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                handle_theme_toggle(this, cx);
            }
        }));

        Self {
            focus_scope,
            pane_focus,
            look,
            split_view,
            sidebar,
            theme_toggle_button,
            _subscriptions: subscriptions,
        }
    }
}

impl HasShellTheme for InsetShellApp {
    fn look(&self) -> &Arc<ShadcnLook> {
        &self.look
    }

    fn theme_toggle_button(&self) -> IconButton {
        self.theme_toggle_button.clone()
    }
}

impl Render for InsetShellApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane_focus = self.pane_focus.clone();
        let sidebar = self.sidebar.clone();
        let look = self.look.clone();
        let sans_family = look.mode_tokens().typography.font.sans.family.clone();
        let split_view = self.split_view.clone();

        self.split_view.update(cx, |split_view, cx| {
            split_view.set_panes(
                render_pane(move || sidebar.clone()),
                render_pane(move || {
                    wrap_content_pane(content::inset_content_pane(), pane_focus.clone(), &look, sans_family.clone())
                }),
                cx,
            );
        });

        let body = inset_shell_body(split_view);

        render_app_root(&self.focus_scope, &self.look, render_title_bar("Shell: Inset", self, cx), body)
    }
}

fn inset_shell_body(split_view: Entity<SplitView>) -> AnyElement {
    let shell_pad = px(10.0);
    let inset_pad = px(10.0);

    div()
        .size_full()
        .p(shell_pad)
        .child(div().size_full().rounded(px(16.0)).bg(rgb(0x242835)).p(inset_pad).child(split_view))
        .into_any_element()
}
