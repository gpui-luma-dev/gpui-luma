use std::sync::Arc;

use gpui::{AnyElement, Context, FontWeight, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma::{column, column_emphasis, list_view};
use gpui_luma::controls::list_view::{
    ListSelectionMode, ListViewColumn, ListViewEvent, PagingListView, PagingListViewBuilder,
    column_template_with_modifier, default_text_column_template,
};
use gpui_luma::theme::RadixTheme;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use super::super::shared::gallery_pane_with_usage_top_aligned;

const DEFAULT_PAGE_SIZE: usize = 10;

#[derive(Clone)]
struct Task {
    id: SharedString,
    title: SharedString,
    email: SharedString,
    tag: &'static str,
    status: &'static str,
    enabled: bool,
}

#[derive(Clone)]
pub(in crate::gallery) struct ListViewPane {
    list_view: PagingListView<Task>,
    selected_indices: Vec<usize>,
}

impl ListViewPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let tasks = build_task_rows();
        let list_builder = list_view! {
            radix = radix_theme.clone();
            id = "listview-tasks";
            items = tasks;
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
                    .min_h(px(model.appearance.min_height))
                    .py(px(model.appearance.padding_y))
                    .bg(model.appearance.background)
                    .text_color(model.appearance.label_color)
                    .text_size(px(model.appearance.label_typography.size))
                    .line_height(px(model.appearance.label_typography.line_height))
                    .font_weight(model.appearance.label_typography.weight)
                    .child(cells)
                    .into_any_element()
            };
        };

        let list_view = PagingListViewBuilder::new(list_builder.paged(DEFAULT_PAGE_SIZE), radix_theme).spawn(cx);

        Self { list_view, selected_indices: vec![1] }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.list_view, |app, _, event: &ListViewEvent, cx| {
            app.panes.list_view.handle_list_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        let chrome = radix_theme.chrome();

        gallery_pane_with_usage_top_aligned(
            "ListView",
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
                                .child("Paged task grid with the SDK paging toolbar wired to ListView commands."),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(self.selected_summary()),
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

    fn selected_summary(&self) -> String {
        if let Some(&index) = self.selected_indices.first() {
            format!("Selected row index: {index} (Task T-{index:04})")
        } else {
            "No row selected".to_string()
        }
    }
}

fn email_column() -> ListViewColumn<Task> {
    ListViewColumn::fixed(
        "Email",
        200.0,
        column_template_with_modifier(
            default_text_column_template(|row: &Task| row.email.clone()),
            |cell, model, _window, _cx| {
                if model.selected {
                    div().font_weight(FontWeight::SEMIBOLD).child(cell).into_any_element()
                } else {
                    cell
                }
            },
        ),
    )
}

fn build_task_rows() -> Vec<Task> {
    const TASK_COUNT: usize = 100;
    (0..TASK_COUNT).map(make_task).collect()
}

fn make_task(index: usize) -> Task {
    const TAGS: &[&str] = &["UI", "API", "Docs", "Bug"];
    const STATUSES: &[&str] = &["Open", "In progress", "Done", "Blocked"];

    let tag = TAGS[index % TAGS.len()];
    let status = STATUSES[index % STATUSES.len()];

    Task {
        id: format!("T-{index:04}").into(),
        title: format!("Ship list view row {index}").into(),
        email: format!("owner+{index}@luma.dev").into(),
        tag,
        status,
        enabled: !index.is_multiple_of(17),
    }
}

fn tag_pill(tag: &'static str) -> impl IntoElement {
    div()
        .px(px(6.0))
        .py(px(2.0))
        .rounded(px(4.0))
        .bg(gpui::hsla(0.12, 0.55, 0.92, 0.18))
        .text_size(px(11.0))
        .line_height(px(14.0))
        .font_weight(FontWeight::MEDIUM)
        .child(tag)
}

fn status_cell(status: &'static str) -> impl IntoElement {
    let (icon, color) = match status {
        "Done" => (LucideIcon::CircleCheck, gpui::hsla(0.35, 0.7, 0.45, 1.0)),
        "Blocked" => (LucideIcon::CircleX, gpui::hsla(0.0, 0.7, 0.55, 1.0)),
        "In progress" => (LucideIcon::LoaderCircle, gpui::hsla(0.58, 0.75, 0.5, 1.0)),
        _ => (LucideIcon::Circle, gpui::hsla(0.0, 0.0, 0.55, 1.0)),
    };

    div()
        .w_full()
        .min_w(px(0.0))
        .flex()
        .items_center()
        .gap(px(6.0))
        .child(div().text_color(color).child(lucide_glyph(icon)))
        .child(div().flex_1().min_w(px(0.0)).truncate().child(status))
}
