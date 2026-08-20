//! Scrolling list view control exposition — scrollable task grid with fixed viewport.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::list_view::{ListSelectionMode, ListViewEvent, ScrollingListView};
use gpui_luma::{column, column_emphasis, scrolling_list_view};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::collection_theme_inspectors::ListViewThemeInspector;
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::list_view_demo::{Task, build_task_rows, email_column, selected_summary, status_cell, tag_pill};
use super::list_view_inspector_adapter::{ListViewInspectorAdapter, LIST_VIEW_INSPECTOR_SPEC};
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

const DEFAULT_VISIBLE_ROWS: usize = 25;

pub struct ScrollingListViewControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ScrollingListViewExpositionLeftPane>,
    theme_inspector: Entity<ListViewThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct ScrollingListViewExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    list_view: ScrollingListView<Task>,
    selected_indices: Vec<usize>,
    event_stream: Entity<ControlEventStream>,
}

impl ScrollingListViewExpositionLeftPane {
    fn handle_list_event(&mut self, event: &ListViewEvent, cx: &mut Context<Self>) {
        if let ListViewEvent::SelectionChanged { selected_indices } = event {
            self.selected_indices = selected_indices.clone();
            cx.notify();
        }
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.list_view.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ScrollingListViewExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();

            let preview = div()
                .w_full()
                .max_w(px(760.0))
                .flex()
                .flex_col()
                .items_start()
                .gap(px(16.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_sm()
                                .text_color(chrome.muted_text)
                                .child("Scrollable task grid with a fixed viewport and row snap scrolling."),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(chrome.body_text)
                                .child(selected_summary(&self.selected_indices)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(chrome.muted_text)
                                .child("Keyboard: Arrow keys move the active row. Enter or Space selects it."),
                        ),
                )
                .child(self.list_view.clone())
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-scrolling-list-view-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl ScrollingListViewControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("scrolling-list-view").expect("scrolling-list-view catalog entry");

        let tasks = build_task_rows();
        let list_view = scrolling_list_view! {
            list_view_theme = look.list_view_theme();
            id = "controls-doc-listview-scroll";
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
                        .child(gpui_luma::controls::icon::lucide_icon(
                            LucideIcon::EllipsisVertical,
                            gpui::hsla(0.0, 0.0, 0.5, 1.0),
                            16.0,
                        ))
                }),
            };
            row_template = |model, cells, _window, _cx| {
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .min_h(px(model.look.min_height))
                    .py(px(model.look.padding_y))
                    .bg(model.look.background)
                    .text_color(model.look.label_color)
                    .text_size(px(model.look.label_typography.size))
                    .line_height(px(model.look.label_typography.line_height))
                    .font_weight(model.look.label_typography.weight)
                    .child(cells)
            };
        }
        .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-scrolling-list-view-event-log",
                "Select rows and scroll; ListViewEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|_| ScrollingListViewExpositionLeftPane {
            look: look.clone(),
            entry,
            list_view: list_view.clone(),
            selected_indices: vec![1],
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-scrolling-list-view-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &LIST_VIEW_INSPECTOR_SPEC,
            ListViewInspectorAdapter::shared(),
        );

        let subscription = cx.subscribe(&list_view, {
            let event_stream = event_stream.clone();
            let left_pane = left_pane.clone();
            move |_, _, event: &ListViewEvent, cx| {
                left_pane.update(cx, |pane, cx| pane.handle_list_event(event, cx));
                if let Some(line) = format_list_view_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: vec![subscription] }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for ScrollingListViewControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-scrolling-list-view-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn format_list_view_event(event: &ListViewEvent) -> Option<String> {
    match event {
        ListViewEvent::SelectionChanged { selected_indices } => {
            Some(format!("ListViewEvent::SelectionChanged {{ selected_indices: {selected_indices:?} }}"))
        }
        ListViewEvent::ActiveIndexChanged { active_index } => {
            Some(format!("ListViewEvent::ActiveIndexChanged {{ active_index: {active_index:?} }}"))
        }
        ListViewEvent::ScrollChanged { top_index } => {
            Some(format!("ListViewEvent::ScrollChanged {{ top_index: {top_index} }}"))
        }
        ListViewEvent::PageChanged { page } => Some(format!("ListViewEvent::PageChanged {{ page: {page} }}")),
        ListViewEvent::PageSizeChanged { page_size } => {
            Some(format!("ListViewEvent::PageSizeChanged {{ page_size: {page_size} }}"))
        }
        ListViewEvent::FocusChanged { focused } => {
            Some(format!("ListViewEvent::FocusChanged {{ focused: {focused} }}"))
        }
        ListViewEvent::RowHoverChanged { index, hovered } => {
            Some(format!("ListViewEvent::RowHoverChanged {{ index: {index}, hovered: {hovered} }}"))
        }
        ListViewEvent::EnabledChanged { enabled } => {
            Some(format!("ListViewEvent::EnabledChanged {{ enabled: {enabled} }}"))
        }
        _ => None,
    }
}
