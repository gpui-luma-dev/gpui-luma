//! Paging list view control exposition — paged task grid with embedded pager.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma::controls::list_view::{ListSelectionMode, ListViewEvent, PagingListView};
use gpui_luma::controls::pager::PagerStyle;
use gpui_luma::{column, column_emphasis, paging_list_view};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::list_view_demo::{Task, build_task_rows, email_column, selected_summary, status_cell, tag_pill};
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const DEFAULT_PAGE_SIZE: usize = 10;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "ListViewEvent::SelectionChanged { selected_indices }",
        trigger: "Enter or Space on active row, or pointer click",
        notes: "Indices refer to the full item vector across all pages.",
    },
    EventReferenceSpec {
        event: "ListViewEvent::PageChanged { page }",
        trigger: "Pager prev/next or page jump",
        notes: "List view and embedded pager stay in sync.",
    },
    EventReferenceSpec {
        event: "ListViewEvent::ActiveIndexChanged { active_index }",
        trigger: "Arrow keys move roving focus",
        notes: "Active row may differ from selection in some modes.",
    },
    EventReferenceSpec {
        event: "ListViewEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the list",
        notes: "Useful for form-level focus coordination.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "PagingListView<T>",
        surface: "Type",
        notes: "Entity<PagingListViewControl<T>> — grid with embedded pager toolbar.",
    },
    PublicInterfaceSpec {
        symbol: "paging_list_view!",
        surface: "Macro",
        notes: "Builder macro wiring page_size, pager factory, and grid columns.",
    },
    PublicInterfaceSpec {
        symbol: "look.pager(id).style(PagerStyle::MinimalEdge)",
        surface: "Look",
        notes: "Pager chrome paired with list view page commands.",
    },
    PublicInterfaceSpec {
        symbol: "ListViewBuilder::page_size",
        surface: "Builder",
        notes: "Rows per page; pager derives page count from item length.",
    },
];

pub struct PagingListViewControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    list_view: PagingListView<Task>,
    selected_indices: Vec<usize>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl PagingListViewControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("paging-list-view").expect("paging-list-view catalog entry");

        let tasks = build_task_rows();
        let list_view = paging_list_view! {
            list_view_theme = look.list_view_theme();
            id = "controls-doc-listview-paged";
            items = tasks;
            page_size = DEFAULT_PAGE_SIZE;
            pager = look
                .pager("controls-doc-listview-paged-pager")
                .style(PagerStyle::MinimalEdge)
                .page_size(DEFAULT_PAGE_SIZE);
            selection = ListSelectionMode::Single;
            selected_index = 1;
            active_index = 1;
            row_label = |row| row.title.clone();
            row_enabled = |row| row.enabled;
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
                "controls-paging-list-view-event-log",
                "Select rows and change pages; ListViewEvent variants appear below.",
            )
        });

        let subscriptions = vec![cx.subscribe(&list_view, {
            let event_stream = event_stream.clone();
            move |this, _, event: &ListViewEvent, cx| {
                this.handle_list_event(event, cx);
                if let Some(line) = format_list_view_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        })];

        Self { look, entry, list_view, selected_indices: vec![1], event_stream, _subscriptions: subscriptions }
    }

    fn handle_list_event(&mut self, event: &ListViewEvent, cx: &mut Context<Self>) {
        if let ListViewEvent::SelectionChanged { selected_indices } = event {
            self.selected_indices = selected_indices.clone();
            cx.notify();
        }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.list_view.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for PagingListViewControlExposition {
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
                                .child("Paged task grid with the SDK paging toolbar wired to ListView commands."),
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

            render_control_exposition_card(
                look,
                self.entry,
                preview.into_any_element(),
                Some(render_exposition_doc_sections(look, EVENT_SPECS, PUBLIC_INTERFACE_SPECS)),
                ControlExpositionLayout::BORDERLESS,
            )
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
