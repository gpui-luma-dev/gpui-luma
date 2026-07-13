use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, FocusHandle, IntoElement, ParentElement, Render, Subscription, Window, div,
    prelude::*, px,
};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::navigation_sidebar::NavigationSidebar;
use gpui_luma::controls::split_view::{SplitView, SplitViewSeparatorVisibility, render_pane};
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_shell_common::{
    chrome::{HasShellTheme, render_app_root, render_title_bar, wrap_content_pane},
    content::{self, ICON_RAIL_COLLAPSED_WIDTH},
    nav_sample,
    split_sync::wire_split_nav_sync,
    theme::{ShellThemeChoice, sync_color_control_theme},
};

pub struct IconRailShellApp {
    focus_scope: FocusHandle,
    pane_focus: FocusHandle,
    look: Arc<ShadcnLook>,
    split_view: Entity<SplitView>,
    navigation_sidebar: Entity<NavigationSidebar>,
    toggle_button: Entity<Button<()>>,
    _subscriptions: Vec<Subscription>,
}

impl IconRailShellApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, theme_choice: ShellThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        let pane_focus = cx.focus_handle().tab_stop(true);
        window.focus(&focus_scope, cx);
        let look = theme_choice.shadcn_look();
        look.set_mode(ThemeMode::Dark);
        sync_color_control_theme(&look);

        let navigation_sidebar = nav_sample::spawn_properties_sidebar(look.clone(), "shell-nav", cx);
        let split_view = look
            .split_view("shell-icon-rail")
            .sidebar_width(px(560.0))
            .sidebar_min_width(px(440.0))
            .sidebar_max_width(px(760.0))
            .sidebar_collapsed_width(px(ICON_RAIL_COLLAPSED_WIDTH))
            .separator_visibility(SplitViewSeparatorVisibility::Hover)
            .spawn(cx);
        let toggle_button = look.secondary_button("shell-icon-rail-toggle").label("Toggle Collapse").spawn(cx);

        let mut subscriptions = Vec::new();
        wire_split_nav_sync(&mut subscriptions, split_view.clone(), navigation_sidebar.clone(), cx);
        subscriptions.push(cx.subscribe(&toggle_button, {
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

impl HasShellTheme for IconRailShellApp {
    fn look(&self) -> &Arc<ShadcnLook> {
        &self.look
    }
}

impl Render for IconRailShellApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane_focus = self.pane_focus.clone();
        let navigation_sidebar = self.navigation_sidebar.clone();
        let look = self.look.clone();
        let sans_family = look.mode_tokens().typography.font.sans.family.clone();
        let toggle_button = self.toggle_button.clone();

        self.split_view.update(cx, |split_view, cx| {
            split_view.set_panes(
                render_pane(move || navigation_sidebar.clone()),
                render_pane(move || {
                    wrap_content_pane(
                        icon_rail_content_pane(toggle_button.clone()),
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
            render_title_bar("Shell: Icon Rail", self, cx),
            self.split_view.clone(),
        )
    }
}

fn icon_rail_content_pane(toggle_button: Entity<Button<()>>) -> AnyElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .gap_3()
        .p_3()
        .child(toggle_button)
        .child(div().flex_1().min_h_0().child(content::shell_content_pane()))
        .into_any_element()
}
