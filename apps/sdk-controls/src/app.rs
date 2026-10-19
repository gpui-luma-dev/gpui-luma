use std::sync::Arc;
use gpui::{Context, Entity, FocusHandle, IntoElement, Render, ScrollHandle, Window, div, prelude::*, px};
use gpui_luma::{
    controls::{
        button::{Button, ButtonEventExt},
        resizable_panels::{DefaultResizablePanelsTheme, ResizablePanelSpec, ResizablePanels, ResizablePanelsBuilder},
        sidebar::{
            self, DefaultSidebarTheme, SidebarControl, SidebarEvent, SidebarEventExt, ThemedSidebarPanelTemplate,
        },
        switch::{self, Switch, SwitchEvent, SwitchEventExt},
    },
    infra::presenter::HasPresenter,
    shell::TitleBar,
    theme::ThemeMode,
};
use luma_control_support::{
    event_log::EventLogView,
    format_switch_event,
    native::NativeTheme,
    shell::workspace_canvas,
    switch_sample::{InspectedSwitchTemplate, SwitchInspector},
};
use crate::catalog;
use gpui_luma::focus::LumaFocusScopeExt;

pub struct SdkControlsApp {
    theme: NativeTheme,
    sidebar: Entity<SidebarControl>,
    page: Entity<WorkPane>,
    panels: Entity<ResizablePanels>,
    mode_toggle: Entity<Button>,
    sidebar_toggle: Entity<Button>,
    focus: FocusHandle,
}
impl SdkControlsApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let theme = NativeTheme::light();
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        let page = cx.new(|cx| WorkPane::new(theme.clone(), cx));
        let mut groups = sidebar::sidebar_content();
        for (group, entries) in catalog::GROUPS {
            let mut menu = sidebar::sidebar_menu(format!("sdk-{group}"));
            for (id, label) in *entries {
                menu = menu.item(
                    sidebar::sidebar_menu_item(
                        *id,
                        if *id == "switch" {
                            label.to_string()
                        } else {
                            format!("{label} · deferred")
                        },
                    )
                    .active(*id == "switch"),
                );
            }
            groups = groups.group(sidebar::sidebar_group().label(*group).menu(menu));
        }
        let sidebar = sidebar::new("sdk-index")
            .selected_id("switch")
            .sidebar(sidebar::sidebar("sdk-catalog").content(groups))
            .panel_template(Arc::new(ThemedSidebarPanelTemplate::new(Arc::new(DefaultSidebarTheme::new(
                theme.tokens.clone(),
            )))))
            .scrollbar_template(theme.scrollbar_template())
            .spawn(cx);
        sidebar.clone().on_event(cx, |this, event, cx| {
            if let SidebarEvent::Select { id } = event {
                this.page.update(cx, |page, cx| {
                    page.selected = id.to_string();
                    cx.notify();
                });
            }
        });
        let mode_toggle = theme
            .button("sdk-mode")
            .label("Light / dark")
            .on_click(cx, |this, cx| this.toggle_mode(cx))
            .spawn(cx);
        let sidebar_toggle = theme
            .button("sdk-sidebar")
            .label("Controls")
            .on_click(cx, |this, cx| {
                this.panels.update(cx, |panels, cx| {
                    panels.toggle_panel_hidden(0, gpui_luma::controls::resizable_panels::PanelHideMode::Completely, cx)
                });
            })
            .spawn(cx);
        let nav = sidebar.clone();
        let content = page.clone();
        // shell-2026's resizable sidebar/workspace, without its unrelated destination rail.
        let panels = ResizablePanelsBuilder::horizontal("sdk-workspace")
            .show_border(false)
            .theme(Arc::new(DefaultResizablePanelsTheme::new(theme.tokens.clone())))
            .panel(ResizablePanelSpec::new_render(move || nav.clone()).size(px(260.0)).min(px(200.0)).max(px(480.0)))
            .panel(ResizablePanelSpec::new_render(move || content.clone()).weight(1.0).min(px(550.0)))
            .spawn(cx);
        Self { theme, sidebar, page, panels, mode_toggle, sidebar_toggle, focus }
    }
    fn toggle_mode(&mut self, cx: &mut Context<Self>) {
        let mode = match self.theme.mode {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
        self.theme = NativeTheme::new(mode);
        self.sidebar.update(cx, |sidebar, cx| {
            sidebar.set_panel_template(
                Arc::new(ThemedSidebarPanelTemplate::new(Arc::new(DefaultSidebarTheme::new(
                    self.theme.tokens.clone(),
                )))),
                cx,
            );
            sidebar.set_scrollbar_template(self.theme.scrollbar_template(), cx);
        });
        for button in [&self.mode_toggle, &self.sidebar_toggle] {
            button.update(cx, |button, cx| button.set_template(self.theme.button_template(), cx));
        }
        self.panels.update(cx, |panels, cx| {
            panels.set_theme(Arc::new(DefaultResizablePanelsTheme::new(self.theme.tokens.clone())), cx)
        });
        self.page.update(cx, |page, cx| page.set_theme(self.theme.clone(), cx));
        cx.notify();
    }
}
impl Render for SdkControlsApp {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let p = &self.theme.tokens.palette;
        div()
            .size_full()
            .flex()
            .flex_col()
            .luma_focus_scope(&self.focus)
            .font_family(self.theme.tokens.typography.font.sans.family.clone())
            .bg(p.shell.background)
            .text_color(p.app.foreground)
            .child(
                TitleBar::new()
                    .height(px(40.0))
                    .background_color(p.shell.background)
                    .text_color(p.shell.foreground)
                    .child(
                        div()
                            .pl(px(80.0))
                            .flex()
                            .items_center()
                            .gap(px(12.0))
                            .child(self.sidebar_toggle.clone())
                            .child("SDK Controls")
                            .child(self.mode_toggle.clone()),
                    ),
            )
            .child(
                workspace_canvas(p.app.background, p.border.default)
                    .ml(px(3.0))
                    .min_h_0()
                    .child(self.panels.clone()),
            )
    }
}

struct WorkPane {
    theme: NativeTheme,
    selected: String,
    switch: Switch,
    inspector: Entity<SwitchInspector>,
    events: Entity<EventLogView>,
    split: Entity<ResizablePanels>,
    sample: Entity<SwitchSample>,
}
struct SwitchSample {
    theme: NativeTheme,
    switch: Switch,
    events: Entity<EventLogView>,
    enabled: Switch,
    reset: Entity<Button>,
    inspector_toggle: Entity<Button>,
    scroll: ScrollHandle,
}
impl WorkPane {
    fn new(theme: NativeTheme, cx: &mut Context<Self>) -> Self {
        let inspector = cx.new(|_| SwitchInspector::new(theme.clone()));
        let switch = switch::new("sdk-switch")
            .label("Notifications")
            .template(Arc::new(InspectedSwitchTemplate::new(&theme, inspector.clone())))
            .spawn(cx);
        let events = theme
            .event_log("sdk-switch-events")
            .placeholder("Interact with the Switch to record semantic events.")
            .rows(6)
            .full_width(true)
            .spawn(cx);
        let log = events.clone();
        switch.clone().on_event(cx, move |_, event: &SwitchEvent, cx| {
            log.update(cx, |log, cx| log.append_line(&format_switch_event(event), cx));
        });
        let sample = cx.new(|cx| {
            let target = switch.clone();
            let enabled = switch::new("sdk-switch-enabled")
                .label("Enabled")
                .with_data(true)
                .template(Arc::new(gpui_luma::controls::switch::ThemedSwitchTemplate::new(theme.switch_theme())))
                .on_value_changed(cx, move |_: &mut SwitchSample, enabled, cx| {
                    target.update(cx, |target, cx| target.set_enabled(enabled, cx))
                })
                .spawn(cx);
            let reset = theme
                .button("sdk-switch-reset")
                .label("Reset value")
                .on_click(cx, |this: &mut SwitchSample, cx| {
                    this.switch.update(cx, |control, cx| control.set_data(false, cx));
                    this.events.update(cx, |log, cx| {
                        log.append_line("Reset value to false (programmatic; no Change event)", cx)
                    });
                })
                .spawn(cx);
            let inspector_toggle = theme.button("sdk-inspector-toggle").label("Show / hide inspector").spawn(cx);
            SwitchSample {
                inspector_toggle,
                theme: theme.clone(),
                switch: switch.clone(),
                events: events.clone(),
                enabled,
                reset,
                scroll: ScrollHandle::new(),
            }
        });
        let preview = sample.clone();
        let inspection = inspector.clone();
        let split = ResizablePanelsBuilder::horizontal("sdk-switch-split")
            .show_border(false)
            .theme(Arc::new(DefaultResizablePanelsTheme::new(theme.tokens.clone())))
            .panel(ResizablePanelSpec::new_render(move || preview.clone()).weight(1.0).min(px(240.0)))
            .panel(ResizablePanelSpec::new_render(move || inspection.clone()).size(px(360.0)).min(px(240.0)))
            .spawn(cx);
        sample.read(cx).inspector_toggle.clone().on_click(cx, |this, cx| {
            this.split.update(cx, |split, cx| {
                split.toggle_panel_hidden(1, gpui_luma::controls::resizable_panels::PanelHideMode::Completely, cx)
            });
        });
        Self { theme, selected: "switch".into(), switch, inspector, events, split, sample }
    }
    fn set_theme(&mut self, theme: NativeTheme, cx: &mut Context<Self>) {
        self.theme = theme.clone();
        self.inspector.update(cx, |inspector, cx| inspector.set_theme(theme.clone(), cx));
        self.switch.update(cx, |control, cx| {
            control.set_template(Arc::new(InspectedSwitchTemplate::new(&theme, self.inspector.clone())), cx)
        });
        self.events.update(cx, |events, cx| {
            events.set_theme(Arc::new(theme.clone()), cx);
            events.set_scrollbar_template(theme.scrollbar_template(), cx);
        });
        self.split.update(cx, |split, cx| {
            split.set_theme(Arc::new(DefaultResizablePanelsTheme::new(theme.tokens.clone())), cx)
        });
        self.sample.update(cx, |sample, cx| {
            sample.theme = theme.clone();
            sample.enabled.update(cx, |control, cx| {
                control.set_template(
                    Arc::new(gpui_luma::controls::switch::ThemedSwitchTemplate::new(theme.switch_theme())),
                    cx,
                )
            });
            sample.reset.update(cx, |button, cx| button.set_template(theme.button_template(), cx));
            sample.inspector_toggle.update(cx, |button, cx| button.set_template(theme.button_template(), cx));
            cx.notify();
        });
        cx.notify();
    }
}
impl Render for WorkPane {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut pane = div().id("sdk-work-pane").size_full().bg(self.theme.tokens.palette.app.background);
        if self.selected == "switch" {
            pane = pane.child(self.split.clone());
        } else {
            pane = pane.child(div().p(px(24.0)).child(format!("{} · deferred", catalog::label(&self.selected).unwrap_or("Control"))).child(div().child("This first slice implements Switch. Core scenarios remain in Luma Studio until their replacements work here.")));
        }
        pane
    }
}
impl Render for SwitchSample {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().id("sdk-switch-page").size_full().overflow_y_scroll().track_scroll(&self.scroll).p(px(20.0))
            .text_color(self.theme.tokens.palette.app.foreground).flex().flex_col().gap(px(20.0))
            .child(div().text_size(px(24.0)).child("Switch"))
            .child("Native SDK default · Notifications. Tab to focus; Space or Enter toggles. Toggle repeatedly to interrupt motion.")
            .child(div().debug_selector(|| "sdk-switch-bounds".into()).child(self.switch.clone()))
            .child(self.enabled.clone()).child(self.reset.clone()).child(self.inspector_toggle.clone())
            .child("Events · Change, focus, hover and enabled state")
            .child(self.events.clone())
    }
}
#[cfg(test)]
mod tests;
