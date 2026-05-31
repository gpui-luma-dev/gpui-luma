use std::sync::Arc;

use gpui::{AnyElement, Context, Subscription, div, prelude::*, px};
use gpui_luma::controls::list_view::{ListViewEvent, PagingListView, PagingListViewBuilder};
use gpui_luma::theme::RadixTheme;

use crate::gallery::control::GalleryApp;
use super::super::shared::gallery_pane_with_usage_top_aligned;
use super::shared::{DEFAULT_PAGE_SIZE, Task, build_task_rows, task_list_builder};

#[derive(Clone)]
pub(in crate::gallery) struct PagingListViewPane {
    list_view: PagingListView<Task>,
    selected_indices: Vec<usize>,
}

impl PagingListViewPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let tasks = build_task_rows();
        let list_builder = task_list_builder("listview-tasks-paged", tasks, radix_theme.clone());
        let list_view = PagingListViewBuilder::new(list_builder.paged(DEFAULT_PAGE_SIZE), radix_theme).spawn(cx);

        Self { list_view, selected_indices: vec![1] }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.list_view, |app, _, event: &ListViewEvent, cx| {
            app.panes.paging_list_view.handle_list_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        let chrome = radix_theme.chrome();

        gallery_pane_with_usage_top_aligned(
            "Paging List View",
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
