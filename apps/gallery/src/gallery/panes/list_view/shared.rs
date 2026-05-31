use std::sync::Arc;

use gpui::{FontWeight, SharedString, div, prelude::*, px};
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma::{column, column_emphasis};
use gpui_luma::controls::list_view::{
    ListSelectionMode, ListViewBuilder, ListViewColumn, column_template_with_modifier, default_text_column_template,
};
use gpui_luma::theme::RadixTheme;
use lucide_icons::Icon as LucideIcon;

pub(super) const DEFAULT_VISIBLE_ROWS: usize = 10;
pub(super) const DEFAULT_PAGE_SIZE: usize = 10;

#[derive(Clone)]
pub(super) struct Task {
    pub id: SharedString,
    pub title: SharedString,
    pub email: SharedString,
    pub tag: &'static str,
    pub status: &'static str,
    pub enabled: bool,
}

pub(super) fn task_list_builder(
    id: impl Into<SharedString>,
    tasks: Vec<Task>,
    radix_theme: Arc<RadixTheme>,
) -> ListViewBuilder<Task> {
    gpui_luma::controls::list_view::new_typed(id)
        .theme(radix_theme.list_view_theme())
        .items(tasks)
        .selection_mode(ListSelectionMode::Single)
        .selected_index(1)
        .active_index(1)
        .row_label(|row| row.title.clone())
        .row_enabled(|row| row.enabled)
        .grid_view(task_grid_columns())
        .with_row_template(|model, cells, _window, _cx| {
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
        })
}

fn task_grid_columns() -> Vec<ListViewColumn<Task>> {
    vec![
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
    ]
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

pub(super) fn build_task_rows() -> Vec<Task> {
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
