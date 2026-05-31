use std::sync::Arc;

use gpui::{AnyElement, Context, Subscription, div, prelude::*, px};
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma::controls::list_view::{ListSelectionMode, ListViewEvent, ScrollingListView};
use gpui_luma::{column, column_emphasis, scrolling_list_view};
use gpui_luma::theme::RadixTheme;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use super::super::shared::gallery_pane_with_usage_top_aligned;
use super::common::{Task, build_task_rows, email_column, selected_summary, status_cell, tag_pill};

const DEFAULT_VISIBLE_ROWS: usize = 25;

#[derive(Clone)]
pub(in crate::gallery) struct ScrollingListViewPane {
    list_view: ScrollingListView<Task>,
    selected_indices: Vec<usize>,
}

impl ScrollingListViewPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let tasks = build_task_rows();
        let list_view = scrolling_list_view! {
            radix = radix_theme.clone();
            id = "listview-tasks-scroll";
            items = tasks;
            selection = ListSelectionMode::Single;
            selected_index = 1;
            active_index = 1;
            row_label = |row| row.title.clone();
            row_enabled = |row| row.enabled;
            visible_rows = DEFAULT_VISIBLE_ROWS;
            scroll_snap = true;
            grid_view = {
                column_emphasis!("Task", width = 108 => |row: &Task| row.id.clone()),
                column!("Title" => |row: &Task| {
                    div()
                        .w_full()
                        .min_w(px(0.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(tag_pill(row.tag))
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .truncate()
                                .child(row.title.clone()),
                        )
                }),
                column!("Status", width = 132 => |row: &Task| status_cell(row.status)),
                email_column(),
                column!("", width = 44 => |_row: &Task| {
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(lucide_glyph(LucideIcon::EllipsisVertical))
                }),
            };
            row_template = |model, cells, _window, _cx| {
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .min_h(px(model.appearance.min_height))
                    .py(px(model.appearance.padding_y))
                    .bg(model.appearance.background)
                    .text_color(model.appearance.label_color)
                    .text_size(px(model.appearance.label_typography.size))
                    .line_height(px(model.appearance.label_typography.line_height))
                    .font_weight(model.appearance.label_typography.weight)
                    .child(cells)
            };
        }
        .spawn(cx);

        Self { list_view, selected_indices: vec![1] }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.list_view, |app, _, event: &ListViewEvent, cx| {
            app.panes.scrolling_list_view.handle_list_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        let chrome = radix_theme.chrome();

        gallery_pane_with_usage_top_aligned(
            "Scrolling List View",
            "ListView",
            div()
                .w(px(760.0))
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.muted_text)
                                .child("Scrollable task grid with a fixed viewport and row snap scrolling."),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(selected_summary(&self.selected_indices)),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.muted_text)
                                .child("Keyboard: Arrow keys move the active row. Enter or Space selects it."),
                        ),
                )
                .child(self.list_view.clone())
                .into_any_element(),
            radix_theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        cx.notify();
    }

    fn handle_list_event(&mut self, event: &ListViewEvent, cx: &mut Context<GalleryApp>) {
        if let ListViewEvent::SelectionChanged { selected_indices } = event {
            self.selected_indices = selected_indices.clone();
            cx.notify();
        }
    }
}
