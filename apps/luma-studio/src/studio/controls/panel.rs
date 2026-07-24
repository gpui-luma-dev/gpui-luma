use std::sync::Arc;

use gpui::{AnyElement, App, Context, FontWeight, MouseButton, ScrollHandle, Window, div, point, prelude::*, px};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use super::catalog::{ControlCategory, ControlDocEntry, CONTROL_CATALOG, entries_for_category};
use super::control_exposition::ControlExposition;

const CONTROL_INDEX_WIDTH: f32 = 168.0;

pub struct ControlsPanel {
    look: Arc<ShadcnLook>,
    expositions: Vec<ControlExposition>,
    scroll_handle: ScrollHandle,
    selected_entry_id: &'static str,
}

impl ControlsPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        Self {
            look: look.clone(),
            expositions: ControlExposition::spawn_all(look, cx),
            scroll_handle: ScrollHandle::new(),
            selected_entry_id: CONTROL_CATALOG.first().map(|entry| entry.id).unwrap_or("button"),
        }
    }

    pub fn sync_snapshot(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for exposition in &self.expositions {
            exposition.sync_look(look.clone(), cx);
        }
        cx.notify();
    }

    fn select_entry(&mut self, entry_id: &'static str, cx: &mut Context<Self>) {
        if self.selected_entry_id == entry_id {
            return;
        }
        self.selected_entry_id = entry_id;
        self.scroll_handle.set_offset(point(px(0.0), px(0.0)));
        cx.notify();
    }

    fn render_selected_page(&self, cx: &App) -> AnyElement {
        let Some(exposition) = ControlExposition::find(&self.expositions, self.selected_entry_id, cx) else {
            return div().child("Control documentation not found.").into_any_element();
        };
        exposition.render(cx)
    }
}

impl gpui::Render for ControlsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let page = self.render_selected_page(cx);

            div()
                .id("luma-studio-controls")
                .size_full()
                .min_h_0()
                .flex()
                .flex_col()
                .overflow_hidden()
                .bg(chrome.content_background)
                .px(px(28.0))
                .pt(px(12.0))
                .pb(px(28.0))
                .child(
                    div()
                        .w_full()
                        .min_h(px(0.0))
                        .flex_1()
                        .flex()
                        .items_stretch()
                        .gap(px(16.0))
                        .child(
                            div().relative().flex_1().min_h(px(0.0)).child(
                                div()
                                    .id("controls-content")
                                    .size_full()
                                    .overflow_y_scroll()
                                    .scrollbar_width(px(0.0))
                                    .track_scroll(&self.scroll_handle)
                                    .child(
                                        div()
                                            .w_full()
                                            .flex()
                                            .justify_start()
                                            .pb(px(12.0))
                                            .child(div().w_full().child(page)),
                                    ),
                            ),
                        )
                        .child(self.render_control_index(cx)),
                )
        })
    }
}

impl ControlsPanel {
    fn render_control_index(&self, cx: &mut Context<Self>) -> AnyElement {
        let chrome = self.look.chrome();

        div()
            .id("controls-control-index")
            .w(px(CONTROL_INDEX_WIDTH))
            .flex_shrink_0()
            .child(
                div().w_full().h_full().flex().items_center().justify_center().child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .text_size(px(18.0))
                                .line_height(px(24.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(chrome.title_text)
                                .child("Control Index"),
                        )
                        .children(ControlCategory::ALL.into_iter().filter_map(|category| {
                            let entries: Vec<_> = entries_for_category(category).collect();
                            if entries.is_empty() {
                                return None;
                            }
                            Some(
                                div()
                                    .w_full()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(
                                        div()
                                            .px(px(8.0))
                                            .text_size(px(11.0))
                                            .line_height(px(14.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(chrome.muted_text)
                                            .child(category.label()),
                                    )
                                    .children(entries.into_iter().map(|entry| {
                                        self.render_control_index_item(*entry, entry.id == self.selected_entry_id, cx)
                                    })),
                            )
                        })),
                ),
            )
            .into_any_element()
    }

    fn render_control_index_item(&self, entry: ControlDocEntry, active: bool, cx: &mut Context<Self>) -> AnyElement {
        let chrome = self.look.chrome();
        let color = if active { chrome.title_text } else { chrome.muted_text };

        div()
            .id(format!("controls-index-item-{}", entry.id))
            .w_full()
            .px(px(8.0))
            .py(px(2.0))
            .rounded(px(6.0))
            .text_size(px(13.0))
            .line_height(px(17.0))
            .font_weight(if active {
                FontWeight::SEMIBOLD
            } else {
                FontWeight::NORMAL
            })
            .text_color(gpui::hsla(color.h, color.s, color.l, if active { 1.0 } else { 0.82 }))
            .when(active, |item| {
                item.bg(gpui::hsla(chrome.muted_text.h, chrome.muted_text.s, chrome.muted_text.l, 0.10))
            })
            .cursor_pointer()
            .hover(move |style| {
                style
                    .bg(gpui::hsla(chrome.muted_text.h, chrome.muted_text.s, chrome.muted_text.l, 0.12))
                    .text_color(chrome.title_text)
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    this.select_entry(entry.id, cx);
                }),
            )
            .child(entry.title)
            .into_any_element()
    }
}
