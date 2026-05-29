use std::sync::Arc;

use gpui::{AnyElement, App, Context, FontWeight, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::list_view::{ListSelectionMode, ListViewEvent, ListViewItemLike};
use gpui_luma::theme::RadixTheme;
use gpui_luma::theme::radix::prelude::*;

use crate::gallery::control::GalleryApp;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

const DEMO_ROW_COUNT: usize = 10_000;

#[derive(Clone, Debug)]
struct DemoUser {
    name: SharedString,
    email: SharedString,
    role: SharedString,
    status: SharedString,
}

impl ListViewItemLike for DemoUser {
    fn label(&self) -> &SharedString {
        &self.name
    }
}

#[derive(Clone)]
pub(in crate::gallery) struct ListViewPane {
    list: gpui_luma::controls::list_view::ListView<DemoUser>,
    selected_indices: Vec<usize>,
}

impl ListViewPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let list = radix_theme
            .list_view("listview-users")
            .items((0..DEMO_ROW_COUNT).map(make_demo_user))
            .selection_mode(ListSelectionMode::Single)
            .selected_index(2048)
            .active_index(2048)
            .with_header_template(render_demo_user_header)
            .with_item_template(render_demo_user_row)
            .spawn(cx);

        Self { list, selected_indices: vec![2048] }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.list, |app, _, event: &ListViewEvent, cx| {
            app.panes.list_view.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        let chrome = radix_theme.chrome();

        gallery_pane_with_usage(
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
                                .child("Virtualized 10,000-row list. Only visible rows are rendered."),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(selected_summary(self.selected_indices.first().copied())),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.muted_text)
                                .child("Keyboard: Arrow keys move the active row. Enter or Space selects it."),
                        ),
                )
                .child(div().w_full().h(px(420.0)).flex().child(self.list.clone()))
                .into_any_element(),
            radix_theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.list, cx);
    }

    fn handle_event(&mut self, event: &ListViewEvent, cx: &mut Context<GalleryApp>) {
        if let ListViewEvent::SelectionChanged { selected_indices } = event {
            self.selected_indices = selected_indices.clone();
            cx.notify();
        }
    }
}

fn render_demo_user_row(
    model: &gpui_luma::controls::list_view::ListViewItemRenderModel<'_, DemoUser>,
    _window: &mut Window,
    _cx: &mut App,
) -> gpui::AnyElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(12.0))
        .child(div().w(px(180.0)).min_w(px(180.0)).truncate().child(model.item.name.clone()))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .truncate()
                .opacity(if model.selected { 1.0 } else { 0.82 })
                .child(model.item.email.clone()),
        )
        .child(div().w(px(120.0)).min_w(px(120.0)).truncate().child(model.item.role.clone()))
        .child(
            div().w(px(96.0)).min_w(px(96.0)).flex().justify_end().child(
                div()
                    .px(px(8.0))
                    .py(px(2.0))
                    .rounded(px(999.0))
                    .text_size(px(11.0))
                    .line_height(px(14.0))
                    .child(model.item.status.clone()),
            ),
        )
        .into_any_element()
}

fn render_demo_user_header(
    _model: &gpui_luma::controls::list_view::ListViewRenderModel<'_>,
    _window: &mut Window,
    _cx: &mut App,
) -> gpui::AnyElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(12.0))
        .text_size(px(11.0))
        .line_height(px(14.0))
        .font_weight(FontWeight::SEMIBOLD)
        .child(div().w(px(180.0)).min_w(px(180.0)).flex().justify_center().child("Name"))
        .child(div().flex_1().min_w(px(0.0)).flex().justify_center().child("Email"))
        .child(div().w(px(120.0)).min_w(px(120.0)).flex().justify_center().child("Role"))
        .child(div().w(px(96.0)).min_w(px(96.0)).flex().justify_center().child("Status"))
        .into_any_element()
}

fn make_demo_user(index: usize) -> DemoUser {
    const FIRST_NAMES: &[&str] = &["Avery", "Morgan", "Jordan", "Taylor", "Riley", "Cameron", "Quinn", "Parker"];
    const LAST_NAMES: &[&str] = &["Reed", "Nguyen", "Patel", "Kim", "Lopez", "Singh", "Howard", "Brooks"];
    const ROLES: &[&str] = &["Design", "Platform", "Infra", "Support", "Product", "Data"];
    const STATUSES: &[&str] = &["Active", "Review", "Paused", "Queued"];

    let first = FIRST_NAMES[index % FIRST_NAMES.len()];
    let last = LAST_NAMES[(index / FIRST_NAMES.len()) % LAST_NAMES.len()];
    let role = ROLES[index % ROLES.len()];
    let status = STATUSES[index % STATUSES.len()];
    let slug = format!("{}.{}", first.to_lowercase(), last.to_lowercase());

    DemoUser {
        name: format!("{first} {last} #{index:04}").into(),
        email: format!("{slug}+{index}@luma.dev").into(),
        role: role.into(),
        status: status.into(),
    }
}

fn selected_summary(selected_index: Option<usize>) -> String {
    match selected_index {
        Some(index) => {
            let user = make_demo_user(index);
            format!("Selected row {index}: {} ({})", user.name, user.email)
        }
        None => "No row selected".to_string(),
    }
}
