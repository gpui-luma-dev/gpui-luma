use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent, HasPresenter};
use gpui_luma::controls::pager::{Pager, PagerEvent, PagerStyle};
use gpui_luma::{vstack, wrappanel};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnTextSize};

use crate::gallery::control::GalleryApp;

use super::super::shared::{gallery_pane_with_description, notify_entity};

const PAGE_SIZES: [usize; 3] = [10, 25, 50];

#[derive(Clone)]
pub(in crate::gallery) struct PagerPane {
    minimal: Pager,
    minimal_edge: Pager,
    numeric: Pager,
    numeric_no_edges: Pager,
    numeric_compact: Pager,
    page_prev_button: Entity<Button>,
    page_next_button: Entity<Button>,
    total_down_button: Entity<Button>,
    total_up_button: Entity<Button>,
    size_down_button: Entity<Button>,
    size_up_button: Entity<Button>,
    enabled_toggle_button: Entity<Button>,
    current_page: usize,
    total_items: usize,
    page_size: usize,
    enabled: bool,
}

impl PagerPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let current_page = 2;
        let total_items = 281;
        let page_size = 25;
        let enabled = true;
        let page_count = page_count_for(total_items, page_size);

        let minimal = look
            .pager("gallery-pager-minimal")
            .style(PagerStyle::Minimal)
            .page_size(page_size)
            .page_size_options(PAGE_SIZES)
            .page_count(page_count)
            .current_page(current_page)
            .page_indicator_formatter(|current, total| SharedString::from(format!("{current} / {total}")))
            .spawn(cx);
        let minimal_edge = look
            .pager("gallery-pager-minimal-edge")
            .style(PagerStyle::MinimalEdge)
            .page_size(page_size)
            .page_size_options(PAGE_SIZES)
            .page_count(page_count)
            .current_page(current_page)
            .page_indicator_formatter(|current, total| SharedString::from(format!("{current} of {total}")))
            .spawn(cx);
        let numeric = look
            .pager("gallery-pager-numeric")
            .style(PagerStyle::Numeric)
            .page_size(page_size)
            .page_size_options(PAGE_SIZES)
            .page_count(page_count)
            .current_page(current_page)
            .spawn(cx);
        let numeric_no_edges = look
            .pager("gallery-pager-numeric-no-edges")
            .style(PagerStyle::Numeric)
            .page_size(page_size)
            .page_size_options(PAGE_SIZES)
            .page_count(page_count)
            .current_page(current_page)
            .show_first_last(false)
            .previous_label("Back")
            .next_label("Next")
            .spawn(cx);
        let numeric_compact = look
            .pager("gallery-pager-numeric-compact")
            .style(PagerStyle::Numeric)
            .page_size(page_size)
            .page_size_options(PAGE_SIZES)
            .page_count(page_count)
            .current_page(current_page)
            .numeric_slot_count(5)
            .first_label("First")
            .previous_label("Back")
            .next_label("Next")
            .last_label("Last")
            .spawn(cx);
        Self {
            minimal,
            minimal_edge,
            numeric,
            numeric_no_edges,
            numeric_compact,
            page_prev_button: look.secondary_button("gallery-pager-page-prev").label("Page -").spawn(cx),
            page_next_button: look.secondary_button("gallery-pager-page-next").label("Page +").spawn(cx),
            total_down_button: look.secondary_button("gallery-pager-total-down").label("Items -").spawn(cx),
            total_up_button: look.secondary_button("gallery-pager-total-up").label("Items +").spawn(cx),
            size_down_button: look.secondary_button("gallery-pager-size-down").label("Per page -").spawn(cx),
            size_up_button: look.secondary_button("gallery-pager-size-up").label("Per page +").spawn(cx),
            enabled_toggle_button: look
                .outline_button("gallery-pager-enabled-toggle")
                .label("Toggle enabled")
                .spawn(cx),
            current_page,
            total_items,
            page_size,
            enabled,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        for pager in self.pagers() {
            subscriptions.push(cx.subscribe(&pager, |app, _, event: &PagerEvent, cx| {
                app.panes.pager.handle_pager_event(event, cx);
            }));
        }

        subscriptions.push(cx.subscribe(&self.page_prev_button, |app, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.panes.pager.step_page(-1, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&self.page_next_button, |app, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.panes.pager.step_page(1, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&self.total_down_button, |app, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.panes.pager.step_total_items(-1, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&self.total_up_button, |app, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.panes.pager.step_total_items(1, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&self.size_down_button, |app, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.panes.pager.step_page_size(-1, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&self.size_up_button, |app, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.panes.pager.step_page_size(1, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&self.enabled_toggle_button, |app, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.panes.pager.toggle_enabled(cx);
            }
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();
        let detail_style = look.typography_scale(ShadcnTextSize::Sm);
        let label_style = look.typography_scale(ShadcnTextSize::Xs);

        gallery_pane_with_description(
            "Pager",
            Some(
                "Standalone pager control preview with the built-in styles: Minimal, Minimal + first/last, and Numeric. Page count is derived from total items and per-page size.",
            ),
            vstack! {
                gap=16.0;
                vstack! {
                    gap=12.0;
                    div().typography_style(detail_style).text_color(chrome.body_text).child(
                        format!(
                            "Current page: {} · Page count: {} · Total items: {} · Per page: {} · Enabled: {}",
                            self.current_page + 1,
                            self.page_count(),
                            self.total_items,
                            self.page_size,
                            if self.enabled { "Yes" } else { "No" }
                        ),
                    ),
                    wrappanel! {
                        gap=8.0;
                        self.page_prev_button.clone(),
                        self.page_next_button.clone(),
                        self.total_down_button.clone(),
                        self.total_up_button.clone(),
                        self.size_down_button.clone(),
                        self.size_up_button.clone(),
                        self.enabled_toggle_button.clone(),
                    },
                }
                .w_full(),
                render_sample("Minimal · formatter: \"X / Y\"", self.minimal.clone(), chrome.muted_text, label_style),
                render_sample(
                    "Minimal + first/last · formatter: \"X of Y\"",
                    self.minimal_edge.clone(),
                    chrome.muted_text,
                    label_style,
                ),
                render_sample("Numeric", self.numeric.clone(), chrome.muted_text, label_style),
                render_sample("Numeric · no first/last", self.numeric_no_edges.clone(), chrome.muted_text, label_style),
                render_sample("Numeric · 5 fixed slots", self.numeric_compact.clone(), chrome.muted_text, label_style),
            }
            .w(px(820.0))
            .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        for pager in self.pagers() {
            notify_entity(&pager, cx);
        }
        notify_entity(&self.page_prev_button, cx);
        notify_entity(&self.page_next_button, cx);
        notify_entity(&self.total_down_button, cx);
        notify_entity(&self.total_up_button, cx);
        notify_entity(&self.size_down_button, cx);
        notify_entity(&self.size_up_button, cx);
        notify_entity(&self.enabled_toggle_button, cx);
        cx.notify();
    }

    fn pagers(&self) -> [Pager; 5] {
        [
            self.minimal.clone(),
            self.minimal_edge.clone(),
            self.numeric.clone(),
            self.numeric_no_edges.clone(),
            self.numeric_compact.clone(),
        ]
    }

    fn handle_pager_event(&mut self, event: &PagerEvent, cx: &mut Context<GalleryApp>) {
        match event {
            PagerEvent::PageChanged { page } => {
                self.current_page = clamp_page(*page, self.page_count());
                self.sync_pagers(cx);
            }
            PagerEvent::PageSizeChanged { page_size } => {
                self.page_size = *page_size;
                self.sync_pagers(cx);
            }
            _ => {}
        }
    }

    fn step_page(&mut self, delta: isize, cx: &mut Context<GalleryApp>) {
        self.current_page = offset_usize(self.current_page, delta);
        self.current_page = clamp_page(self.current_page, self.page_count());
        self.sync_pagers(cx);
    }

    fn step_total_items(&mut self, delta: isize, cx: &mut Context<GalleryApp>) {
        let step = self.page_size.max(1);
        self.total_items = if delta < 0 {
            self.total_items.saturating_sub(step)
        } else {
            self.total_items.saturating_add(step)
        };
        self.current_page = clamp_page(self.current_page, self.page_count());
        self.sync_pagers(cx);
    }

    fn step_page_size(&mut self, delta: isize, cx: &mut Context<GalleryApp>) {
        let index = PAGE_SIZES.iter().position(|size| *size == self.page_size).unwrap_or(1);
        let next = offset_usize(index, delta).min(PAGE_SIZES.len().saturating_sub(1));
        self.page_size = PAGE_SIZES[next];
        self.sync_pagers(cx);
    }

    fn toggle_enabled(&mut self, cx: &mut Context<GalleryApp>) {
        self.enabled = !self.enabled;
        self.sync_pagers(cx);
    }

    fn sync_pagers(&mut self, cx: &mut Context<GalleryApp>) {
        let page_count = self.page_count();
        self.current_page = clamp_page(self.current_page, page_count);

        for pager in self.pagers() {
            pager.update(cx, |pager, cx| {
                pager.set_page_count(page_count, cx);
                pager.set_page_size(self.page_size, cx);
                pager.set_page(self.current_page, cx);
                pager.set_enabled(self.enabled, cx);
            });
        }

        cx.notify();
    }

    fn page_count(&self) -> usize {
        page_count_for(self.total_items, self.page_size)
    }
}

fn render_sample(
    label: &'static str,
    pager: Pager,
    muted_text: gpui::Hsla,
    label_style: gpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    vstack! {
        gap=8.0;
        div()
            .typography_style(label_style)
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(muted_text)
            .child(label),
        pager,
    }
    .w_full()
    .into_any_element()
}

fn page_count_for(total_items: usize, page_size: usize) -> usize {
    let page_size = page_size.max(1);
    total_items.div_ceil(page_size).max(1)
}

fn clamp_page(page: usize, page_count: usize) -> usize {
    if page_count == 0 { 0 } else { page.min(page_count - 1) }
}

fn offset_usize(value: usize, delta: isize) -> usize {
    if delta < 0 {
        value.saturating_sub(delta.unsigned_abs())
    } else {
        value.saturating_add(delta as usize)
    }
}
