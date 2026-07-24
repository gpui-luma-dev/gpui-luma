use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, Context, Corner, Entity, FocusHandle, FocusOutEvent, MouseDownEvent, Pixels, ScrollHandle,
    Subscription, Window, anchored, deferred, div, point, prelude::*, px,
};
use gpui_luma::controls::color::style::ElementExt;
use gpui_luma::controls::tabs_navigation::TabsNavigation;
use gpui_luma::focus::EscapeFocus;
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::paint::floating_menu_look;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::components::catalog::first_controls_exposition_id;
use crate::studio::content_tabs::controls_tab_chrome::ControlsTabChrome;

use super::control_catalog_picker::render_control_catalog_picker;
use super::control_exposition::ControlExposition;

const PICKER_FOCUS_CONTEXT: &str = "LumaFocus";

pub struct ControlsPanel {
    look: Arc<ShadcnLook>,
    expositions: Vec<ControlExposition>,
    scroll_handle: ScrollHandle,
    selected_entry_id: &'static str,
    catalog_picker_open: bool,
    controls_tab_visited: bool,
    controls_tab_chrome: ControlsTabChrome,
    content_tabs: Entity<TabsNavigation>,
    picker_focus: FocusHandle,
    picker_focus_out_subscription: Option<Subscription>,
    /// Skips the first focus-out after opening from the Controls tab click.
    picker_dismiss_guard: bool,
    picker_focus_pending: bool,
}

impl ControlsPanel {
    pub fn new(
        cx: &mut Context<Self>,
        look: Arc<ShadcnLook>,
        controls_tab_chrome: ControlsTabChrome,
        content_tabs: Entity<TabsNavigation>,
    ) -> Self {
        Self {
            look: look.clone(),
            expositions: ControlExposition::spawn_all(look, cx),
            scroll_handle: ScrollHandle::new(),
            selected_entry_id: first_controls_exposition_id().unwrap_or("button"),
            catalog_picker_open: false,
            controls_tab_visited: false,
            controls_tab_chrome,
            content_tabs,
            picker_focus: cx.focus_handle().tab_stop(true),
            picker_focus_out_subscription: None,
            picker_dismiss_guard: false,
            picker_focus_pending: false,
        }
    }

    pub fn sync_snapshot(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
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

    pub fn open_catalog_picker(&mut self, cx: &mut Context<Self>) {
        if self.catalog_picker_open {
            return;
        }
        self.picker_dismiss_guard = true;
        self.picker_focus_pending = true;
        self.set_picker_open(true, cx);
    }

    pub fn toggle_catalog_picker(&mut self, cx: &mut Context<Self>) {
        self.do_toggle_catalog_picker(cx);
    }

    fn do_toggle_catalog_picker(&mut self, cx: &mut Context<Self>) {
        if self.catalog_picker_open {
            self.close_catalog_picker(cx);
            return;
        }
        self.open_catalog_picker(cx);
    }

    pub fn close_catalog_picker(&mut self, cx: &mut Context<Self>) {
        if !self.catalog_picker_open {
            return;
        }
        self.picker_dismiss_guard = false;
        self.picker_focus_pending = false;
        self.set_picker_open(false, cx);
    }

    fn set_picker_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.catalog_picker_open = open;
        self.controls_tab_chrome.set_picker_open(open);
        self.content_tabs.update(cx, |_, cx| cx.notify());
        cx.notify();
    }

    fn select_entry(&mut self, entry_id: &'static str, cx: &mut Context<Self>) {
        if self.selected_entry_id != entry_id {
            self.selected_entry_id = entry_id;
            self.scroll_handle.set_offset(point(px(0.0), px(0.0)));
        }
        self.close_catalog_picker(cx);
    }

    fn handle_picker_focus_out(&mut self, _: FocusOutEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.picker_dismiss_guard {
            self.picker_dismiss_guard = false;
            return;
        }
        if !window.is_window_active() {
            return;
        }
        self.close_catalog_picker(cx);
    }

    fn handle_picker_mouse_down_out(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.controls_tab_chrome.tab_bounds().is_some_and(|bounds| bounds.contains(&event.position)) {
            return;
        }
        if self.picker_dismiss_guard {
            self.picker_dismiss_guard = false;
            return;
        }
        if !window.is_window_active() {
            return;
        }
        self.close_catalog_picker(cx);
    }

    fn handle_picker_escape(&mut self, _: &EscapeFocus, _window: &mut Window, cx: &mut Context<Self>) {
        self.close_catalog_picker(cx);
    }

    fn ensure_picker_focus_subscription(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.picker_focus_out_subscription.is_some() {
            return;
        }
        let focus = self.picker_focus.clone();
        self.picker_focus_out_subscription = Some(cx.on_focus_out(&focus, window, Self::handle_picker_focus_out));
    }

    fn render_selected_page(&self, cx: &App) -> AnyElement {
        let Some(exposition) = ControlExposition::find(&self.expositions, self.selected_entry_id, cx) else {
            return div().child("Control documentation not found.").into_any_element();
        };
        exposition.render(cx)
    }

    fn render_catalog_picker(&self, cx: &mut Context<Self>) -> AnyElement {
        let menu_look = floating_menu_look(self.look.mode_tokens().as_ref(), self.look.mode(), ControlSize::Md);
        render_control_catalog_picker(
            &menu_look,
            Some(self.selected_entry_id),
            true,
            cx,
            |this, exposition_id, _, cx| this.select_entry(exposition_id, cx),
        )
    }

    fn wrap_picker_for_width_measure(
        picker: AnyElement,
        chrome: ControlsTabChrome,
        panel: Entity<ControlsPanel>,
    ) -> AnyElement {
        div()
            .on_prepaint(move |bounds: Bounds<Pixels>, _, cx| {
                if bounds.size.width <= px(0.0) {
                    return;
                }
                if chrome.set_catalog_picker_width(bounds.size.width) {
                    panel.update(cx, |_, cx| cx.notify());
                }
            })
            .child(picker)
            .into_any_element()
    }

    fn render_catalog_picker_popup(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let picker = self.render_catalog_picker(cx);
        let chrome = self.controls_tab_chrome.clone();
        let panel = cx.entity();
        let measured_picker = Self::wrap_picker_for_width_measure(picker, chrome, panel);

        let Some(trigger_bounds) = self.controls_tab_chrome.tab_bounds() else {
            return div().into_any_element();
        };

        let offset_y = self.look.parse_pixel_token("spacing").unwrap_or(4.0);
        let horizontal_offset = self.controls_tab_chrome.catalog_picker_width().map_or(px(0.0), |width| -(width * 0.5));

        div()
            .id("controls-catalog-picker-root")
            .track_focus(&self.picker_focus)
            .key_context(PICKER_FOCUS_CONTEXT)
            .occlude()
            .on_mouse_down_out(cx.listener(Self::handle_picker_mouse_down_out))
            .on_action(cx.listener(Self::handle_picker_escape))
            .child(
                deferred(
                    anchored()
                        .snap_to_window_with_margin(px(8.0))
                        .anchor(Corner::TopLeft)
                        .position(point(trigger_bounds.center().x, trigger_bounds.bottom()))
                        .offset(point(horizontal_offset, px(offset_y)))
                        .child(measured_picker),
                )
                .with_priority(1),
            )
            .into_any_element()
    }

    fn picker_ready_for_focus(&self) -> bool {
        self.catalog_picker_open && self.controls_tab_chrome.tab_bounds().is_some()
    }

    fn schedule_picker_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.picker_focus_pending || !self.picker_ready_for_focus() {
            return;
        }
        self.picker_focus_pending = false;
        self.ensure_picker_focus_subscription(window, cx);
        let focus = self.picker_focus.clone();
        cx.on_next_frame(window, move |this, window, cx| {
            if this.catalog_picker_open {
                focus.focus(window, cx);
            }
        });
        self.picker_dismiss_guard = false;
    }
}

impl gpui::Render for ControlsPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.schedule_picker_focus(window, cx);

        let catalog_open = self.catalog_picker_open;
        let popup = catalog_open.then(|| self.render_catalog_picker_popup(cx));

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
                .when_some(popup, |panel, popup| panel.child(popup))
        })
    }
}
