use std::sync::Arc;

use gpui::{Context, Entity, FocusHandle, IntoElement, Render, Subscription, Window, px};
use gpui_luma::controls::command::button::ButtonEvent;
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma::controls::navigation_sidebar::NavigationSidebar;
use gpui_luma::controls::split_view::{SplitView, SplitViewSeparatorVisibility, render_pane};
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_shell_common::{
    chrome::{HasShellTheme, render_app_root, render_title_bar, wrap_content_pane},
    content, nav_sample,
    split_sync::wire_split_nav_sync,
    theme::{ShellThemeChoice, sync_color_control_theme},
};
use lucide_icons::Icon as LucideIcon;

pub struct DetachedShellApp {
    focus_scope: FocusHandle,
    pane_focus: FocusHandle,
    look: Arc<ShadcnLook>,
    split_view: Entity<SplitView>,
    navigation_sidebar: Entity<NavigationSidebar>,
    toggle_button: IconButton,
    _subscriptions: Vec<Subscription>,
}

impl DetachedShellApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, theme_choice: ShellThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        let pane_focus = cx.focus_handle().tab_stop(true);
        window.focus(&focus_scope, cx);
        let look = theme_choice.shadcn_look();
        look.set_mode(ThemeMode::Dark);
        sync_color_control_theme(&look);

        let navigation_sidebar = nav_sample::spawn_properties_sidebar(look.clone(), "shell-nav", cx);
        let toggle_button = look.ghost_icon_button("shell-detached-toggle", LucideIcon::Menu).spawn(cx);
        let split_view = look
            .split_view("shell-detached")
            .sidebar_width(px(560.0))
            .sidebar_min_width(px(440.0))
            .sidebar_max_width(px(840.0))
            .separator_visibility(SplitViewSeparatorVisibility::Hover)
            .spawn(cx);

        let mut subscriptions = Vec::new();
        wire_split_nav_sync(&mut subscriptions, split_view.clone(), navigation_sidebar.clone(), cx);
        let toggle_for_sub = toggle_button.clone();
        subscriptions.push(cx.subscribe(&toggle_for_sub, {
            let split_view = split_view.clone();
            move |_, _, _: &ButtonEvent, cx| {
                split_view.update(cx, |split_view, cx| {
                    split_view.toggle_collapsed(cx);
                });
            }
        }));

        Self {
            focus_scope,
            pane_focus,
            look,
            split_view,
            navigation_sidebar,
            toggle_button,
            _subscriptions: subscriptions,
        }
    }
}

impl HasShellTheme for DetachedShellApp {
    fn look(&self) -> &Arc<ShadcnLook> {
        &self.look
    }
}

impl Render for DetachedShellApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane_focus = self.pane_focus.clone();
        let navigation_sidebar = self.navigation_sidebar.clone();
        let look = self.look.clone();
        let sans_family = look.mode_tokens().typography.font.sans.family.clone();
        let toggle_button = self.toggle_button.clone();

        self.split_view.update(cx, |split_view, cx| {
            split_view.set_panes(
                render_pane(move || content::detached_nav_pane_with_sidebar(navigation_sidebar.clone())),
                render_pane(move || {
                    wrap_content_pane(
                        content::detached_content_pane(toggle_button.clone()),
                        pane_focus.clone(),
                        &look,
                        sans_family.clone(),
                    )
                }),
                cx,
            );
        });

        render_app_root(
            &self.focus_scope,
            &self.look,
            render_title_bar("Shell: Detached", self, cx),
            self.split_view.clone(),
        )
    }
}
