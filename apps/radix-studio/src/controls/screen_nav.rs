//! Top-level screen navigation with the SDK animated tabs control.

use std::sync::Arc;

use gpui::{Context, Entity, EventEmitter, IntoElement, Render, Subscription, Window, div, prelude::*};
use gpui_luma::controls::button::{Button, ButtonEvent};
use gpui_luma::controls::tabs::{Tabs};
use gpui_luma::infra::presenter::HasPresenter;
use gpui_luma::prelude::TooltipEntityExt;
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_radix::Look;
use gpui_luma_look_radix as radix;

use crate::assets::{icon_named, react_icon};
use crate::tabs::RadixStudioTab;

/// Radix icons are drawn on a native 15x15 grid.
const THEME_ICON_SIZE: f32 = 15.0;

pub(crate) const SCREEN_TABS: [(RadixStudioTab, &str, &str); 5] = [
    (RadixStudioTab::CustomPalette, "page-custom-palette", "Custom Palette"),
    (RadixStudioTab::Colors, "page-colors", "Colors"),
    (RadixStudioTab::Icons, "page-icons", "Icons"),
    (RadixStudioTab::StyleGuide, "page-style-guide", "Style Guide"),
    (RadixStudioTab::Developer, "page-developer", "Developer"),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScreenNavEvent {
    Change { tab: RadixStudioTab },
    ModeChange { mode: ThemeMode },
    ResetTheme,
}

pub struct ScreenNav {
    look: Arc<Look>,
    active: RadixStudioTab,
    tabs: Entity<Tabs>,
    theme_toggle: Entity<Button>,
    theme_reset: Entity<Button>,
    github: Entity<Button>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ScreenNavEvent> for ScreenNav {}

impl ScreenNav {
    /// Page navigation uses the app look; theme actions follow the editable palette.
    pub fn new(look: &Arc<Look>, action_look: &Arc<Look>, tabs: Entity<Tabs>, cx: &mut Context<Self>) -> Self {
        let icon_look = Arc::clone(action_look);
        let theme_toggle = radix::Button::new("screen-nav-theme")
            .look(action_look)
            .ghost_quiet()
            .content(move |model, _| {
                let name = match icon_look.mode() {
                    ThemeMode::Dark => "moon",
                    ThemeMode::Light => "sun",
                };
                icon_named(name)
                    .map(|icon| react_icon(icon, model.look.foreground, THEME_ICON_SIZE))
                    .unwrap_or_else(|| div().into_any_element())
            })
            .spawn(cx)
            .help("Switch between light and dark mode", cx);

        let theme_reset = radix::Button::new("screen-nav-reset-theme")
            .look(action_look)
            .ghost_quiet()
            .content(|model, _| {
                icon_named("reset")
                    .map(|icon| react_icon(icon, model.look.foreground, THEME_ICON_SIZE))
                    .unwrap_or_else(|| div().into_any_element())
            })
            .spawn(cx)
            .help("Reset the custom palette", cx);

        let github = radix::Button::new("screen-nav-github")
            .look(action_look)
            .ghost_quiet()
            .content(|model, _| {
                icon_named("github-logo")
                    .map(|icon| react_icon(icon, model.look.foreground, THEME_ICON_SIZE))
                    .unwrap_or_else(|| div().into_any_element())
            })
            .spawn(cx)
            .help("View gpui-luma on GitHub", cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&github, |_, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                cx.open_url("https://github.com/gpui-luma-dev/gpui-luma");
            }
        }));
        subscriptions.push(cx.subscribe(&theme_reset, |_, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                cx.emit(ScreenNavEvent::ResetTheme);
            }
        }));
        subscriptions.push(cx.observe(&tabs, |this, tabs, cx| {
            let active_id = tabs.read(cx).active_id().cloned();
            if let Some((tab, _, _)) = SCREEN_TABS.iter().find(|(_, id, _)| Some(*id) == active_id.as_deref()) {
                this.select(*tab, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&theme_toggle, |this, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                let mode = match this.look.mode() {
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::Dark,
                };
                cx.emit(ScreenNavEvent::ModeChange { mode });
            }
        }));

        Self {
            look: Arc::clone(look),
            active: RadixStudioTab::CustomPalette,
            tabs,
            theme_toggle,
            theme_reset,
            github,
            _subscriptions: subscriptions,
        }
    }

    pub fn active(&self) -> RadixStudioTab {
        self.active
    }

    pub fn select(&mut self, tab: RadixStudioTab, cx: &mut Context<Self>) {
        if self.active == tab {
            return;
        }
        self.active = tab;
        if let Some((_, id, _)) = SCREEN_TABS.iter().find(|(screen, _, _)| *screen == tab) {
            self.tabs.update(cx, |tabs, cx| {
                // User selection already started the indicator transition.
                if tabs.active_id().map(|active| active.as_ref()) != Some(*id) {
                    tabs.set_active(*id, cx);
                }
            });
        }
        cx.emit(ScreenNavEvent::Change { tab });
        cx.notify();
    }

    /// Refresh child controls after mutating their shared Look.
    pub fn theme_changed(&mut self, cx: &mut Context<Self>) {
        self.tabs.update(cx, |tabs, cx| tabs.set_template(radix::tabs_template(&self.look), cx));
        self.theme_toggle.update(cx, |_, cx| cx.notify());
        self.theme_reset.update(cx, |_, cx| cx.notify());
        self.github.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

impl Render for ScreenNav {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .child(div().flex_1())
            .child(self.tabs.read(cx).tab_list())
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .child(self.github.clone())
                    .child(self.theme_reset.clone())
                    .child(self.theme_toggle.clone()),
            )
    }
}
