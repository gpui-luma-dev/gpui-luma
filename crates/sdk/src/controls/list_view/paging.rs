use std::sync::Arc;

use gpui::{AppContext, Context, Entity, EventEmitter, Render, Window, div, prelude::*};

use crate::theme::LumaChrome;

use super::control::{ListViewControl, ListViewEvent};
use super::model::ListViewBuilder;
use super::toolbar::{PagingToolbar, PagingToolbarEvent, PagingToolbarLayout, PagingToolbarTemplate};

pub type PagingToolbarChrome = Arc<dyn Fn() -> LumaChrome + Send + Sync>;

pub type PagingListView<T> = Entity<PagingListViewControl<T>>;

pub struct PagingListViewControl<T: 'static> {
    list: Entity<ListViewControl<T>>,
    toolbar: Entity<PagingToolbar>,
}

impl<T: 'static> PagingListViewControl<T> {
    pub fn new(list: Entity<ListViewControl<T>>, toolbar: Entity<PagingToolbar>, cx: &mut Context<Self>) -> Self {
        let toolbar_clone = toolbar.clone();
        cx.subscribe(&list, move |_, list_control, event, cx| {
            let list = list_control.read(cx);
            match event {
                ListViewEvent::SelectionChanged { selected_indices } => {
                    let count = selected_indices.len();
                    let total = list.items().len();
                    toolbar_clone.update(cx, |toolbar, cx| toolbar.update_selection(count, total, cx));
                }
                ListViewEvent::PageChanged { page } => {
                    let page_size = list.page_size().unwrap_or(10);
                    let page_count = list.page_count();
                    toolbar_clone.update(cx, |toolbar, cx| toolbar.update_page(*page, page_size, page_count, cx));
                }
                ListViewEvent::PageSizeChanged { page_size } => {
                    let current_page = list.current_page();
                    let page_count = list.page_count();
                    toolbar_clone.update(cx, |toolbar, cx| {
                        toolbar.update_page(current_page, *page_size, page_count, cx);
                    });
                }
                ListViewEvent::ActiveIndexChanged { .. } => {}
            }
        })
        .detach();

        let list_clone = list.clone();
        cx.subscribe(&toolbar, move |_, _, event, cx| {
            list_clone.update(cx, |list_control, cx| match event {
                PagingToolbarEvent::FirstPage => list_control.first_page(cx),
                PagingToolbarEvent::PrevPage => list_control.prev_page(cx),
                PagingToolbarEvent::NextPage => list_control.next_page(cx),
                PagingToolbarEvent::LastPage => list_control.last_page(cx),
                PagingToolbarEvent::SetPageSize(size) => list_control.set_page_size(*size, cx),
            });
        })
        .detach();

        cx.subscribe(&list, |_, _, event, cx| {
            cx.emit(event.clone());
        })
        .detach();

        Self { list, toolbar }
    }

    pub fn list(&self) -> &Entity<ListViewControl<T>> {
        &self.list
    }

    pub fn toolbar(&self) -> &Entity<PagingToolbar> {
        &self.toolbar
    }
}

impl<T: 'static> EventEmitter<ListViewEvent> for PagingListViewControl<T> {}

impl<T: 'static> Render for PagingListViewControl<T> {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().w_full().flex().flex_col().child(self.list.clone()).child(self.toolbar.clone())
    }
}

pub struct PagingListViewBuilder<T: 'static> {
    list_builder: ListViewBuilder<T>,
    chrome: PagingToolbarChrome,
    toolbar_template: Option<PagingToolbarTemplate>,
}

impl<T: 'static> PagingListViewBuilder<T> {
    pub fn new(list_builder: ListViewBuilder<T>, chrome: PagingToolbarChrome) -> Self {
        Self { list_builder, chrome, toolbar_template: None }
    }

    pub fn toolbar_template(mut self, template: PagingToolbarTemplate) -> Self {
        self.toolbar_template = Some(template);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> PagingListView<T> {
        let list = cx.new(|cx| ListViewControl::from_builder(self.list_builder, cx));

        let initial_layout = list.read_with(cx, |list, _| PagingToolbarLayout {
            selected_count: list.selected_indices().len(),
            total_rows: list.items().len(),
            current_page: list.current_page(),
            page_count: list.page_count(),
            page_size: list.page_size().unwrap_or(10),
        });

        let mut toolbar = PagingToolbar::new(self.chrome.clone(), initial_layout);
        if let Some(template) = self.toolbar_template {
            toolbar = toolbar.with_custom_template(template);
        }

        let toolbar_entity = cx.new(|_| toolbar);
        cx.new(|cx| PagingListViewControl::new(list, toolbar_entity, cx))
    }
}
