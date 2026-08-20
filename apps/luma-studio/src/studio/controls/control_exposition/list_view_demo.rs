//! Shared task grid data and column helpers for list view expositions.

use gpui::{FontWeight, SharedString, div, prelude::*, px};
use gpui_luma::controls::list_view::{ListViewColumn, column_template_with_modifier, default_text_column_template};
use lucide_svg_static::Icon as LucideIcon;

#[derive(Clone)]
pub(super) struct Task {
    pub id: SharedString,
    pub title: SharedString,
    pub email: SharedString,
    pub tag: &'static str,
    pub status: &'static str,
    pub enabled: bool,
}

pub(super) fn build_task_rows() -> Vec<Task> {
    const TASK_COUNT: usize = 100;
    (0..TASK_COUNT).map(make_task).collect()
}

fn make_task(index: usize) -> Task {
    const TAGS: &[&str] = &["UI", "API", "Docs", "Bug"];
    const STATUSES: &[&str] = &["Open", "In progress", "Done", "Blocked"];

    Task {
        id: format!("T-{index:04}").into(),
        title: format!("Ship list view row {index}").into(),
        email: format!("owner+{index}@luma.dev").into(),
        tag: TAGS[index % TAGS.len()],
        status: STATUSES[index % STATUSES.len()],
        enabled: !index.is_multiple_of(17),
    }
}

pub(super) fn email_column() -> ListViewColumn<Task> {
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

pub(super) fn tag_pill(tag: &'static str) -> impl IntoElement {
    div()
        .px(px(6.0))
        .py(px(2.0))
        .rounded(px(4.0))
        .bg(gpui::hsla(0.12, 0.55, 0.92, 0.18))
        .text_xs()
        .line_height(px(14.0))
        .font_weight(FontWeight::MEDIUM)
        .child(tag)
}

pub(super) fn status_cell(status: &'static str) -> impl IntoElement {
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
        .child(div().text_color(color).child(gpui_luma::controls::icon::lucide_icon(icon, color, 16.0)))
        .child(div().flex_1().min_w(px(0.0)).truncate().child(status))
}

pub(super) fn selected_summary(selected_indices: &[usize]) -> String {
    if let Some(&index) = selected_indices.first() {
        format!("Selected row index: {index} (Task T-{index:04})")
    } else {
        "No row selected".to_string()
    }
}
