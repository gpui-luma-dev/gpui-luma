use std::sync::Arc;

use gpui::{AnyElement, Bounds, Context, Entity, Focusable, Pixels, Render, Subscription, Window, div, prelude::*, px};
use luma::controls::popover_button::{PopoverButton, PopoverButtonEvent, PopoverDismissPolicy, PopoverPlacement};
use luma::controls::tabs::{Tabs, TabsEvent, TabsItem, TabsWidthMode};
use luma::theme::ControlSize;
use luma_look_shadcn::paint::floating_menu_look;
use luma_look_shadcn as shadcn;
use luma_look_shadcn::ShadcnLook;

use super::cards::render_demo_board;
use super::controls;
use super::dashboard;
use super::palette;
use super::navigation::luma_studio_tabs_template;
use super::style_guide;
use super::tab::ContentTab;
use super::theme_usage;
use super::super::app::LumaStudioApp;
use super::super::controls::control_catalog_picker::render_control_catalog_picker;
use super::super::controls::ControlsPanel;
use super::super::demo_controls::DemoControls;
use super::super::overrides::StudioOverrides;
use super::super::panels::{PalettePanel, ThemeUsagePanel};
use super::super::style::StyleGuidePanel;

/// Cached board state — `ContentPaneHost::render` must not read `LumaStudioApp` (re-entrancy panic).
#[derive(Clone)]
pub struct BoardSnapshot {
    pub demos: DemoControls,
    pub look: Arc<ShadcnLook>,
    pub overrides: StudioOverrides,
}

pub struct ContentPaneHost {
    tabs: Entity<Tabs>,
    style_guide_panel: Entity<StyleGuidePanel>,
    controls_panel: Entity<ControlsPanel>,
    palette_panel: Entity<PalettePanel>,
    theme_usage_panel: Entity<ThemeUsagePanel>,
    board: BoardSnapshot,
    active_tab: ContentTab,
    catalog_picker: Entity<PopoverButton>,
    _subscriptions: Vec<Subscription>,
}

impl ContentPaneHost {
    pub fn new(app: Entity<LumaStudioApp>, board: BoardSnapshot, cx: &mut Context<Self>) -> Self {
        let host = cx.entity();
        let controls_panel = cx.new(|cx| ControlsPanel::new(cx, board.look.clone()));
        let picker_host = host.clone();
        let catalog_picker = PopoverButton::new("luma-studio-controls-picker")
            .trigger(|_, _| div().w(px(0.0)).h(px(0.0)).into_any_element())
            .content(move |_, _, cx| picker_host.update(cx, |host, cx| host.render_catalog_picker(cx)))
            .placement(PopoverPlacement::BelowCenter)
            .dismiss_policy(PopoverDismissPolicy::CloseOnClickAwayOrFocusLoss)
            .offset_y(px(-36.0))
            .window_margin(px(8.0))
            .measure_trigger(false)
            .spawn(cx);
        let tabs = shadcn::Tabs::new("luma-studio-content-tabs")
            .look(board.look.as_ref())
            .size(shadcn::ShadcnSize::Lg)
            .width_mode(TabsWidthMode::Uniform)
            .template(luma_studio_tabs_template(board.look.clone(), ControlSize::Lg))
            .items([
                TabsItem::new("cards").label("Cards"),
                TabsItem::new("dashboard").label("Dashboard"),
                TabsItem::new("typography").label("Style Guide"),
                TabsItem::new("controls").label("Controls").dropdown_trigger(),
                TabsItem::new("palette").label("Color Palette"),
                TabsItem::new("theme-usage").label("Theme Usage"),
            ])
            .active("cards")
            .spawn(cx);

        let tabs_for_sub = tabs.clone();
        let picker_for_sub = catalog_picker.clone();
        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&tabs_for_sub, |host, _, event: &TabsEvent, cx| match event {
            TabsEvent::Change { tab_id, .. } => {
                let Some(tab) = ContentTab::from_id(tab_id.as_ref()) else {
                    return;
                };
                host.set_active_tab(tab, cx);
            }
            TabsEvent::DropdownRequested { tab_id, bounds, .. } => {
                if tab_id.as_ref() != "controls" {
                    return;
                }
                host.toggle_catalog_picker(*bounds, cx);
            }
            TabsEvent::ItemBoundsChanged { tab_id, bounds } if tab_id.as_ref() == "controls" => {
                host.set_catalog_picker_anchor(*bounds, cx);
            }
            _ => {}
        }));
        subscriptions.push(cx.subscribe(&picker_for_sub, |host, _, event: &PopoverButtonEvent, cx| {
            let PopoverButtonEvent::OpenChanged { open } = event;
            host.tabs.update(cx, |tabs, cx| {
                tabs.set_item_disclosure_open("controls", *open, cx);
            });
        }));

        let style_guide_panel = cx.new(|cx| StyleGuidePanel::new(cx, board.look.clone()));
        let palette_panel = cx.new(|cx| PalettePanel::new(cx, board.look.clone(), board.overrides.clone()));
        PalettePanel::wire_subscriptions(&palette_panel, app, cx, &mut subscriptions);
        let theme_usage_panel = cx.new(|cx| ThemeUsagePanel::new(cx, board.look.clone()));

        Self {
            tabs,
            style_guide_panel,
            controls_panel,
            palette_panel,
            theme_usage_panel,
            board,
            active_tab: ContentTab::Cards,
            catalog_picker,
            _subscriptions: subscriptions,
        }
    }

    pub fn set_active_tab(&mut self, tab: ContentTab, cx: &mut Context<Self>) {
        if self.active_tab == tab {
            return;
        }
        if self.active_tab == ContentTab::Controls {
            self.close_catalog_picker(cx);
            self.controls_panel.update(cx, |panel, cx| panel.dismiss_overlays(cx));
        }
        self.active_tab = tab;

        if tab == ContentTab::Typography {
            let look = self.board.look.clone();
            self.style_guide_panel.update(cx, |panel, cx| panel.sync_snapshot(look, cx));
        }

        if tab == ContentTab::Controls {
            let look = self.board.look.clone();
            self.controls_panel.update(cx, |panel, cx| {
                panel.sync_snapshot(look, cx);
                panel.request_layout_refresh(cx);
            });
        }

        if tab == ContentTab::Palette {
            let look = self.board.look.clone();
            let overrides = self.board.overrides.clone();
            self.palette_panel.update(cx, |panel, cx| panel.sync_snapshot(look, overrides, cx));
        }

        if tab == ContentTab::ThemeUsage {
            let look = self.board.look.clone();
            self.theme_usage_panel.update(cx, |panel, cx| panel.sync_snapshot(look, cx));
        }
        cx.notify();
    }

    fn toggle_catalog_picker(&mut self, bounds: Option<Bounds<Pixels>>, cx: &mut Context<Self>) {
        if let Some(bounds) = bounds {
            self.set_catalog_picker_anchor(bounds, cx);
        }
        let opener = self.tabs.read(cx).focus_handle(cx);
        self.catalog_picker.update(cx, |picker, cx| {
            picker.toggle_guarded_from(Some(opener), cx);
        });
    }

    fn set_catalog_picker_anchor(&mut self, bounds: Bounds<Pixels>, cx: &mut Context<Self>) {
        self.catalog_picker.update(cx, |picker, cx| {
            picker.set_anchor_bounds(bounds, cx);
        });
    }

    pub(super) fn close_catalog_picker(&mut self, cx: &mut Context<Self>) {
        self.catalog_picker.update(cx, |picker, cx| {
            picker.dismiss(cx);
        });
    }

    pub(super) fn render_catalog_picker(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let menu_look =
            floating_menu_look(self.board.look.mode_tokens().as_ref(), self.board.look.mode(), ControlSize::Sm);
        let selected_entry_id = self.controls_panel.read(cx).selected_entry_id();
        render_control_catalog_picker(&menu_look, Some(selected_entry_id), true, cx, |host, exposition_id, _, cx| {
            host.controls_panel.update(cx, |panel, cx| panel.select_entry(exposition_id, cx));
            host.close_catalog_picker(cx);
        })
    }

    pub fn notify_tabs(&self, cx: &mut Context<Self>) {
        self.tabs.update(cx, |_, cx| cx.notify());
    }

    pub fn request_controls_layout_refresh(&self, cx: &mut Context<Self>) {
        if self.active_tab != ContentTab::Controls {
            return;
        }
        self.controls_panel.update(cx, |panel, cx| panel.request_layout_refresh(cx));
    }

    pub fn sync_controls_host_content_width(&self, width: gpui::Pixels, cx: &mut Context<Self>) {
        if self.active_tab != ContentTab::Controls {
            return;
        }
        self.controls_panel.update(cx, |panel, cx| panel.set_host_content_width(width, cx));
    }

    pub fn sync_board_snapshot(&mut self, board: BoardSnapshot, cx: &mut Context<Self>) {
        let look = board.look.clone();
        let overrides = board.overrides.clone();
        self.board = board;
        self.tabs.update(cx, |tabs, cx| {
            tabs.set_size(ControlSize::Lg, cx);
            tabs.set_width_mode(TabsWidthMode::Uniform, cx);
            tabs.set_template(luma_studio_tabs_template(look.clone(), ControlSize::Lg), cx);
        });
        self.style_guide_panel.update(cx, |panel, cx| panel.sync_snapshot(look.clone(), cx));
        self.controls_panel.update(cx, |panel, cx| panel.sync_snapshot(look.clone(), cx));
        self.palette_panel.update(cx, |panel, cx| panel.sync_snapshot(look.clone(), overrides, cx));
        self.theme_usage_panel.update(cx, |panel, cx| panel.sync_snapshot(look, cx));
        cx.notify();
    }
}

impl Render for ContentPaneHost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let board = &self.board;
        let chrome = board.look.chrome();
        let board_bg = board.look.token_color("background").unwrap_or(chrome.app_background);
        let active_tab = self.active_tab;

        div()
            .id("luma-studio-content-pane")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(board_bg)
            .child(div().flex_shrink_0().pt(px(8.0)).child(div().w_full().child(self.tabs.clone())))
            .child(self.catalog_picker.clone())
            .child(match active_tab {
                ContentTab::Cards => {
                    scrollable_body().child(
                        div().id("luma-studio-cards-content").flex_1().min_h_0().overflow_y_scroll().child(
                            div().p(px(24.0)).child(render_demo_board(board.demos.clone(), active_tab.panels())),
                        ),
                    )
                }
                ContentTab::Dashboard => dashboard::viewport().child(board.demos.dashboard.clone()),
                ContentTab::Typography => style_guide::viewport().child(self.style_guide_panel.clone()),
                ContentTab::Controls => controls::viewport().child(self.controls_panel.clone()),
                ContentTab::Palette => palette::viewport().child(self.palette_panel.clone()),
                ContentTab::ThemeUsage => theme_usage::viewport().child(self.theme_usage_panel.clone()),
            })
    }
}

fn scrollable_body() -> gpui::Div {
    div().flex_1().min_h_0().size_full().flex().flex_col()
}
