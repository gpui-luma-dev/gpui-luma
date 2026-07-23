use std::sync::Arc;

use gpui::{AnyElement, App, Context, FontWeight, MouseButton, ScrollHandle, Window, div, point, prelude::*, px};
use gpui_luma::controls::color::style::ElementExt;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use super::catalog::{ControlCategory, ControlDocEntry, entries_for_category};
use super::control_exposition::{ControlExposition, render_category_heading};
use crate::studio::doc_shell::{
    StickySectionHeadingTracker, render_sticky_section_heading_lane, with_sticky_heading_tracker,
};

const CONTROL_INDEX_WIDTH: f32 = 168.0;

pub struct ControlsPanel {
    look: Arc<ShadcnLook>,
    expositions: Vec<ControlExposition>,
    scroll_handle: ScrollHandle,
    sticky_heading_tracker: std::rc::Rc<std::cell::RefCell<StickySectionHeadingTracker>>,
    last_scroll_offset: std::rc::Rc<std::cell::Cell<f32>>,
    last_max_scroll: std::rc::Rc<std::cell::Cell<f32>>,
}

impl ControlsPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        Self {
            look: look.clone(),
            expositions: ControlExposition::spawn_all(look, cx),
            scroll_handle: ScrollHandle::new(),
            sticky_heading_tracker: std::rc::Rc::new(std::cell::RefCell::new(StickySectionHeadingTracker::default())),
            last_scroll_offset: std::rc::Rc::new(std::cell::Cell::new(0.0)),
            last_max_scroll: std::rc::Rc::new(std::cell::Cell::new(0.0)),
        }
    }

    pub fn sync_snapshot(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for exposition in &self.expositions {
            exposition.sync_look(look.clone(), cx);
        }
        self.sticky_heading_tracker.borrow_mut().reset();
        cx.notify();
    }

    fn set_vertical_offset(&self, value: f32, cx: &mut Context<Self>) {
        self.scroll_handle.set_offset(point(px(0.0), px(-value.max(0.0))));
        cx.notify();
    }

    fn scroll_to_entry(&mut self, entry: ControlDocEntry, cx: &mut Context<Self>) {
        let Some(top) = self.sticky_heading_tracker.borrow().anchor_top(entry.title) else {
            return;
        };
        let max = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        self.set_vertical_offset(top.clamp(0.0, max), cx);
    }
}

impl gpui::Render for ControlsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let scroll_handle = self.scroll_handle.clone();
            let last_scroll_offset = self.last_scroll_offset.clone();
            let last_max_scroll = self.last_max_scroll.clone();
            let sticky_heading_tracker = self.sticky_heading_tracker.clone();
            let scroll_y = (-self.scroll_handle.offset().y.as_f32()).max(0.0);
            let sticky_snapshot = sticky_heading_tracker.borrow().snapshot(scroll_y);
            let active_section_title = sticky_heading_tracker.borrow().active_title(scroll_y);

            with_sticky_heading_tracker(sticky_heading_tracker.clone(), || {
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
                                div()
                                    .relative()
                                    .flex_1()
                                    .min_h(px(0.0))
                                    .child(
                                        div()
                                            .id("controls-content")
                                            .size_full()
                                            .overflow_y_scroll()
                                            .scrollbar_width(px(0.0))
                                            .track_scroll(&self.scroll_handle)
                                            .on_prepaint({
                                                let scroll_handle = scroll_handle.clone();
                                                let last_scroll_offset = last_scroll_offset.clone();
                                                let last_max_scroll = last_max_scroll.clone();
                                                move |_, window: &mut gpui::Window, cx: &mut gpui::App| {
                                                    let offset = (-scroll_handle.offset().y.as_f32()).max(0.0);
                                                    let max_scroll = scroll_handle.max_offset().y.as_f32().max(0.0);
                                                    let offset_changed =
                                                        (last_scroll_offset.get() - offset).abs() > 0.5;
                                                    let max_changed = (last_max_scroll.get() - max_scroll).abs() > 0.5;

                                                    if offset_changed || max_changed {
                                                        last_scroll_offset.set(offset);
                                                        last_max_scroll.set(max_scroll);
                                                        cx.notify(window.current_view());
                                                    }
                                                }
                                            })
                                            .child(
                                                div().w_full().flex().justify_start().pb(px(12.0)).child(
                                                    div()
                                                        .w_full()
                                                        .max_w(px(980.0))
                                                        .flex()
                                                        .flex_col()
                                                        .gap(px(20.0))
                                                        .on_prepaint({
                                                            let sticky_heading_tracker = sticky_heading_tracker.clone();
                                                            move |bounds, window, cx| {
                                                                let changed = sticky_heading_tracker
                                                                    .borrow_mut()
                                                                    .set_content_origin_y(bounds.origin.y.as_f32());
                                                                if changed {
                                                                    cx.notify(window.current_view());
                                                                }
                                                            }
                                                        })
                                                        .children(ControlCategory::ALL.into_iter().filter_map(
                                                            |category| {
                                                                let entries: Vec<_> =
                                                                    entries_for_category(category).collect();
                                                                if entries.is_empty() {
                                                                    return None;
                                                                }
                                                                Some(render_category_section(
                                                                    &self.look,
                                                                    category,
                                                                    &entries,
                                                                    &self.expositions,
                                                                    cx,
                                                                ))
                                                            },
                                                        )),
                                                ),
                                            ),
                                    )
                                    .child(render_sticky_section_heading_lane(
                                        sticky_snapshot,
                                        chrome.content_background,
                                        980.0,
                                    )),
                            )
                            .child(self.render_control_index(active_section_title, cx)),
                    )
            })
        })
    }
}

fn render_category_section(
    look: &ShadcnLook,
    category: ControlCategory,
    entries: &[&ControlDocEntry],
    expositions: &[ControlExposition],
    cx: &App,
) -> AnyElement {
    let chrome = look.chrome();
    let category_order = category.index_order() * 1000;

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(render_category_heading(
            category.label(),
            category_description(category),
            category_order,
            chrome.title_text,
            chrome.muted_text,
            chrome.border,
        ))
        .children(entries.iter().filter_map(|entry| {
            ControlExposition::find(expositions, entry.id, cx).map(|exposition| exposition.render(cx))
        }))
        .into_any_element()
}

fn category_description(category: ControlCategory) -> &'static str {
    match category {
        ControlCategory::Command => "Action triggers such as buttons and toggles.",
        ControlCategory::Choice => "Single and multi-select choice controls.",
        ControlCategory::Inputs => "Text entry surfaces for forms and filters.",
        ControlCategory::Selection => "List-backed pickers, comboboxes, and selectors.",
        ControlCategory::NavigationPanels => "Tabs, sidebars, accordions, and pagers.",
        ControlCategory::OverlaysDialogs => "Menus, popups, and modal overlay windows.",
    }
}

impl ControlsPanel {
    fn render_control_index(&self, active_title: Option<&'static str>, cx: &mut Context<Self>) -> AnyElement {
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
                                        self.render_control_index_item(*entry, active_title == Some(entry.title), cx)
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
                    this.scroll_to_entry(entry, cx);
                }),
            )
            .child(entry.title)
            .into_any_element()
    }
}
