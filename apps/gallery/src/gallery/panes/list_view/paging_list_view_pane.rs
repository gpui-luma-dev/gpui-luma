use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Subscription, div, prelude::*, px};
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma::controls::list_view::{ListSelectionMode, ListViewEvent, PagingListView};
use gpui_luma::controls::pager::PagerStyle;
use gpui_luma::{column, column_emphasis, paging_list_view};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use super::inspector_tree::build_list_view_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector, notify_entity};
use super::common::{Task, build_task_rows, email_column, selected_summary, status_cell, tag_pill};

const DEFAULT_PAGE_SIZE: usize = 10;

#[derive(Clone)]
pub(in crate::gallery) struct PagingListViewPane {
    list_view: PagingListView<Task>,
    selected_indices: Vec<usize>,
    inspector: Entity<ColorInspectorShell>,
}

impl PagingListViewPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree(
            "list-view-paging-inspector-tree",
            look.clone(),
            build_list_view_inspect_tree,
            cx,
        );
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "list-view-paging-inspector",
                "list-view-paging-inspector-split",
                "list-view-paging-inspector-detail",
                build_list_view_inspect_tree,
                cx,
            )
        });
        let tasks = build_task_rows();
        let list_view = paging_list_view! {
            list_view_theme = look.list_view_theme();
            id = "listview-tasks-paged";
            items = tasks;
            page_size = DEFAULT_PAGE_SIZE;
            pager = look
                .pager("listview-tasks-paged-pager")
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

        Self { list_view, selected_indices: vec![1], inspector }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.list_view, |app, _, event: &ListViewEvent, cx| {
            app.panes.paging_list_view.handle_list_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();

        gallery_pane_with_inspector(
            "Paging List View",
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
                .into_any_element(),
            self.inspector.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.list_view, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
        cx.notify();
    }

    fn handle_list_event(&mut self, event: &ListViewEvent, cx: &mut Context<GalleryApp>) {
        if let ListViewEvent::SelectionChanged { selected_indices } = event {
            self.selected_indices = selected_indices.clone();
            cx.notify();
        }
    }
}
