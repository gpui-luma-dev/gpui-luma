//! Top-level screen navigation toggle bar (Custom Palette / Colors / Icons).

use std::sync::Arc;

use gpui::{Context, Entity, EventEmitter, IntoElement, Render, Subscription, Window, div, prelude::*};
use gpui_luma::controls::button::{Button, ButtonEvent};
use gpui_luma::controls::toggle::{Toggle, ToggleEvent};
use gpui_luma::hstack;
use gpui_luma::infra::presenter::HasPresenter;
use gpui_luma::prelude::TooltipEntityExt;
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_radix::Look;
use gpui_luma_look_radix as radix;

use crate::assets::{icon_named, react_icon};
use crate::tabs::RadixStudioTab;

/// Radix icons are drawn on a native 15x15 grid.
const THEME_ICON_SIZE: f32 = 15.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScreenNavEvent {
    Change { tab: RadixStudioTab },
    ModeChange { mode: ThemeMode },
    ResetTheme,
}

pub struct ScreenNav {
    look: Arc<Look>,
    active: RadixStudioTab,
    custom_palette: Toggle,
    colors: Toggle,
    icons: Toggle,
    style_guide: Toggle,
    developer: Toggle,
    theme_toggle: Entity<Button>,
    theme_reset: Entity<Button>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ScreenNavEvent> for ScreenNav {}

impl ScreenNav {
    /// Page navigation uses the app look; theme actions follow the editable palette.
    pub fn new(look: &Arc<Look>, action_look: &Arc<Look>, cx: &mut Context<Self>) -> Self {
        let custom_palette = radix::Toggle::new("page-custom-palette")
            .look(look)
            .page()
            .with_data(true)
            .label("Custom Palette")
            .spawn(cx);
        let colors = radix::Toggle::new("page-colors").look(look).page().with_data(false).label("Colors").spawn(cx);
        let icons = radix::Toggle::new("page-icons").look(look).page().with_data(false).label("Icons").spawn(cx);
        let style_guide = radix::Toggle::new("page-style-guide")
            .look(look)
            .page()
            .with_data(false)
            .label("Style Guide")
            .spawn(cx);
        let developer =
            radix::Toggle::new("page-developer").look(look).page().with_data(false).label("Developer").spawn(cx);
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

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&theme_reset, |_, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                cx.emit(ScreenNavEvent::ResetTheme);
            }
        }));
        subscriptions.push(cx.subscribe(&custom_palette, |this, _, event: &ToggleEvent, cx| {
            if let ToggleEvent::Change { selected: true } = event {
                this.select(RadixStudioTab::CustomPalette, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&colors, |this, _, event: &ToggleEvent, cx| {
            if let ToggleEvent::Change { selected: true } = event {
                this.select(RadixStudioTab::Colors, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&icons, |this, _, event: &ToggleEvent, cx| {
            if let ToggleEvent::Change { selected: true } = event {
                this.select(RadixStudioTab::Icons, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&style_guide, |this, _, event: &ToggleEvent, cx| {
            if let ToggleEvent::Change { selected: true } = event {
                this.select(RadixStudioTab::StyleGuide, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&developer, |this, _, event: &ToggleEvent, cx| {
            if let ToggleEvent::Change { selected: true } = event {
                this.select(RadixStudioTab::Developer, cx);
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
            custom_palette,
            colors,
            icons,
            style_guide,
            developer,
            theme_toggle,
            theme_reset,
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
        self.custom_palette
            .update(cx, |toggle, cx| toggle.set_data(tab == RadixStudioTab::CustomPalette, cx));
        self.colors.update(cx, |toggle, cx| toggle.set_data(tab == RadixStudioTab::Colors, cx));
        self.icons.update(cx, |toggle, cx| toggle.set_data(tab == RadixStudioTab::Icons, cx));
        self.style_guide.update(cx, |toggle, cx| toggle.set_data(tab == RadixStudioTab::StyleGuide, cx));
        self.developer.update(cx, |toggle, cx| toggle.set_data(tab == RadixStudioTab::Developer, cx));
        cx.emit(ScreenNavEvent::Change { tab });
        cx.notify();
    }

    /// Shared Look mutation does not invalidate independently rendered button entities.
    pub fn theme_changed(&mut self, cx: &mut Context<Self>) {
        self.theme_toggle.update(cx, |_, cx| cx.notify());
        self.theme_reset.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

impl Render for ScreenNav {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .child(div().flex_1())
            .child(hstack! {
                gap=8 align=center justify=center;
                self.custom_palette.clone(),
                self.colors.clone(),
                self.icons.clone(),
                self.style_guide.clone(),
                self.developer.clone(),
            })
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .child(self.theme_reset.clone())
                    .child(self.theme_toggle.clone()),
            )
    }
}
