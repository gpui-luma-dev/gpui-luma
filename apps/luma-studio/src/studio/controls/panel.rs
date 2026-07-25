use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, Context, Entity, Pixels, ScrollHandle, Subscription, Window, div, point, prelude::*, px,
};
use gpui_luma::controls::anchored_panel::{
    AnchoredPanel, AnchoredPanelDismissPolicy, AnchoredPanelEvent, AnchoredPanelPlacement,
};
use gpui_luma::controls::tabs_navigation::TabsNavigation;
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::paint::floating_menu_look;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnFont, ShadcnLook};

use crate::studio::components::catalog::first_controls_exposition_id;

use super::control_catalog_picker::{estimate_control_catalog_picker_size, render_control_catalog_picker};
use super::control_exposition::ControlExposition;

pub struct ControlsPanel {
    look: Arc<ShadcnLook>,
    expositions: Vec<ControlExposition>,
    scroll_handle: ScrollHandle,
    selected_entry_id: &'static str,
    controls_tab_visited: bool,
    catalog_picker: Entity<AnchoredPanel>,
    content_tabs: Entity<TabsNavigation>,
    _subscriptions: Vec<Subscription>,
}

impl ControlsPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>, content_tabs: Entity<TabsNavigation>) -> Self {
        let panel = cx.entity();
        let catalog_picker = AnchoredPanel::new("controls-catalog-picker")
            .placement(AnchoredPanelPlacement::BelowCenter)
            .dismiss_policy(AnchoredPanelDismissPolicy::CloseOnClickAwayOrFocusLoss)
            .offset_y(px(look.parse_pixel_token("spacing").unwrap_or(4.0)))
            .content(move |_, _, cx| panel.update(cx, |panel, cx| panel.render_catalog_picker(cx)))
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&catalog_picker, |this, _, event: &AnchoredPanelEvent, cx| {
            let AnchoredPanelEvent::OpenChanged { open } = event else {
                return;
            };
            this.content_tabs.update(cx, |tabs, cx| {
                tabs.set_item_disclosure_open("controls", *open, cx);
            });
            cx.notify();
        }));

        let expositions = ControlExposition::spawn_all(look.clone(), cx);

        Self {
            look: look.clone(),
            expositions,
            scroll_handle: ScrollHandle::new(),
            selected_entry_id: first_controls_exposition_id().unwrap_or("button"),
            controls_tab_visited: false,
            catalog_picker,
            content_tabs,
            _subscriptions: subscriptions,
        }
    }

    pub fn sync_snapshot(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        let offset_y = px(self.look.parse_pixel_token("spacing").unwrap_or(4.0));
        self.catalog_picker.update(cx, |picker, cx| picker.set_offset_y(offset_y, cx));
        for exposition in &self.expositions {
            exposition.sync_look(look.clone(), cx);
        }
        cx.notify();
    }

    pub fn activate_controls_tab(&mut self, cx: &mut Context<Self>) {
        if !self.controls_tab_visited {
            self.controls_tab_visited = true;
            if let Some(entry_id) = first_controls_exposition_id() {
                self.selected_entry_id = entry_id;
                self.scroll_handle.set_offset(point(px(0.0), px(0.0)));
            }
            cx.notify();
        }
    }

    pub fn toggle_catalog_picker(&mut self, cx: &mut Context<Self>) {
        self.catalog_picker.update(cx, |picker, cx| picker.toggle_guarded(cx));
    }

    pub fn close_catalog_picker(&mut self, cx: &mut Context<Self>) {
        self.catalog_picker.update(cx, |picker, cx| picker.close(cx));
    }

    pub fn set_catalog_picker_anchor(&mut self, bounds: Bounds<Pixels>, cx: &mut Context<Self>) {
        self.catalog_picker.update(cx, |picker, cx| picker.set_anchor_bounds(bounds, cx));
    }

    fn select_entry(&mut self, entry_id: &'static str, cx: &mut Context<Self>) {
        if self.selected_entry_id != entry_id {
            self.selected_entry_id = entry_id;
            self.scroll_handle.set_offset(point(px(0.0), px(0.0)));
        }
        self.close_catalog_picker(cx);
    }

    fn render_selected_page(&self, cx: &App) -> AnyElement {
        let Some(exposition) = ControlExposition::find(&self.expositions, self.selected_entry_id, cx) else {
            return div().child("Control documentation not found.").into_any_element();
        };
        exposition.render(cx)
    }

    fn render_catalog_picker(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let menu_look = floating_menu_look(self.look.mode_tokens().as_ref(), self.look.mode(), ControlSize::Md);
        render_control_catalog_picker(
            &menu_look,
            Some(self.selected_entry_id),
            true,
            cx,
            |this, exposition_id, _, cx| this.select_entry(exposition_id, cx),
        )
    }

    fn sync_catalog_picker_layout_hint(&self, window: &mut Window, cx: &mut Context<Self>) {
        let menu_look = floating_menu_look(self.look.mode_tokens().as_ref(), self.look.mode(), ControlSize::Md);
        let size = estimate_control_catalog_picker_size(&menu_look, self.look.font(ShadcnFont::Sans), window);
        self.catalog_picker.update(cx, |picker, cx| picker.set_initial_content_size(size, cx));
    }
}

impl gpui::Render for ControlsPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.sync_catalog_picker_layout_hint(window, cx);

        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let page = self.render_selected_page(cx);

            div()
                .id("luma-studio-controls")
                .size_full()
                .min_h_0()
                .relative()
                .flex()
                .flex_col()
                .overflow_hidden()
                .bg(chrome.content_background)
                .px(px(28.0))
                .pt(px(12.0))
                .pb(px(28.0))
                .child(
                    div().relative().flex_1().min_h(px(0.0)).child(
                        div()
                            .id("controls-content")
                            .size_full()
                            .overflow_y_scroll()
                            .scrollbar_width(px(0.0))
                            .track_scroll(&self.scroll_handle)
                            .child(
                                div().w_full().flex().justify_start().pb(px(12.0)).child(div().w_full().child(page)),
                            ),
                    ),
                )
                .child(self.catalog_picker.clone())
        })
    }
}
