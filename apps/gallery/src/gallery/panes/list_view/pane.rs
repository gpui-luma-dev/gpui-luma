use std::sync::Arc;

use gpui::{AnyElement, Context, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::list_view::{ListSelectionMode, ListViewEvent};
use gpui_luma::theme::RadixTheme;

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

#[derive(Clone)]
pub(in crate::gallery) struct ListViewPane {
    list: gpui_luma::controls::list_view::ListView<DemoUser>,
    selected_indices: Vec<usize>,
}

impl ListViewPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let list = gpui_luma::list_view! {
            radix = radix_theme;
            id = "listview-users";
            items = (0..DEMO_ROW_COUNT).map(make_demo_user);
            selection = ListSelectionMode::Single;
            selected_index = 4;
            active_index = 4;
            grid_view = {
                column!("Name", width = 180 => |user| user.name.clone()),
                column!("Email", width = 250 => |user| user.email.clone()),
                column!("Role", width = 120 => |user| user.role.clone()),
                column!("Status", width = 96 => |user| user.status.clone())
            };
        }
        // .square_corners()
        .spawn(cx);

        Self { list, selected_indices: vec![4] }
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
