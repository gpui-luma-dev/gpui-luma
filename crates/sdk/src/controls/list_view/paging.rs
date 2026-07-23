use std::sync::Arc;

use gpui::{App, AppContext, Context, Entity, EventEmitter, Render, Window, div, prelude::*, px};

use crate::controls::pager::{
    PagerBuilder, PagerControl, PagerEvent, PagerRenderModel, PagerStyle, PagerTemplate, PagerTemplateHandlers,
    PagerTheme, default_pager_theme, render_info_slot, render_nav_group, render_page_indicator,
    render_page_size_select,
};

use super::control::{ListViewControl, ListViewEvent};
use super::model::ListViewBuilder;

pub type PagingListView<T> = Entity<PagingListViewControl<T>>;

pub struct PagingListViewControl<T: 'static> {
    list: Entity<ListViewControl<T>>,
    pager: Entity<PagerControl>,
}

impl<T: 'static> PagingListViewControl<T> {
    pub fn new(list: Entity<ListViewControl<T>>, pager: Entity<PagerControl>, cx: &mut Context<Self>) -> Self {
        let pager_clone = pager.clone();
        cx.subscribe(&list, move |_, list_control, event, cx| {
            let list = list_control.read(cx);
            match event {
                ListViewEvent::SelectionChanged { selected_indices } => {
                    let count = selected_indices.len();
                    let total = list.items().len();
                    pager_clone.update(cx, |pager, cx| pager.set_info_text(Some(selection_summary(count, total)), cx));
                }
                ListViewEvent::PageChanged { page } => {
                    let page_size = list.page_size().unwrap_or(10);
                    let page_count = list.page_count();
                    pager_clone.update(cx, |pager, cx| {
                        pager.set_page_count(page_count, cx);
                        pager.set_page_size(page_size, cx);
                        pager.set_page(*page, cx);
                    });
                }
                ListViewEvent::PageSizeChanged { page_size } => {
                    let current_page = list.current_page();
                    let page_count = list.page_count();
                    pager_clone.update(cx, |pager, cx| {
                        pager.set_page_size(*page_size, cx);
                        pager.set_page_count(page_count, cx);
                        pager.set_page(current_page, cx);
                    });
                }
                ListViewEvent::ActiveIndexChanged { .. } => {}
                _ => {}
            }
        })
        .detach();

        let list_clone = list.clone();
        cx.subscribe(&pager, move |_, _, event, cx| {
            list_clone.update(cx, |list_control, cx| {
                if let PagerEvent::PageChanged { page } = event {
                    list_control.set_page(*page, cx);
                } else if let PagerEvent::PageSizeChanged { page_size } = event {
                    list_control.set_page_size(*page_size, cx);
                }
            });
        })
        .detach();

        cx.subscribe(&list, |_, _, event, cx| {
            cx.emit(event.clone());
        })
        .detach();

        Self { list, pager }
    }

    pub fn list(&self) -> &Entity<ListViewControl<T>> {
        &self.list
    }

    pub fn pager(&self) -> &Entity<PagerControl> {
        &self.pager
    }

    fn fills_height(&self, cx: &App) -> bool {
        self.list.read(cx).fills_height()
    }
}

impl<T: 'static> EventEmitter<ListViewEvent> for PagingListViewControl<T> {}

impl<T: 'static> Render for PagingListViewControl<T> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let list = self.list.clone();
        let pager = self.pager.clone();
        if self.fills_height(cx) {
            div()
                .size_full()
                .min_h_0()
                .flex()
                .flex_col()
                .child(div().flex_1().min_h_0().w_full().child(list))
                .child(div().flex_none().w_full().child(pager))
        } else {
            div().w_full().flex().flex_col().child(list).child(pager)
        }
    }
}

struct ListViewPagerTemplate {
    theme: Arc<dyn PagerTheme>,
}

impl ListViewPagerTemplate {
    fn new(theme: Arc<dyn PagerTheme>) -> Self {
        Self { theme }
    }
}

impl PagerTemplate for ListViewPagerTemplate {
    fn render(
        &self,
        model: &PagerRenderModel<'_>,
        handlers: PagerTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::AnyElement {
        let look = self.theme.resolve(model.enabled, PagerStyle::MinimalEdge);

        div()
            .w_full()
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(look.group_gap))
            .py(px(look.padding_y))
            .text_color(look.muted_text)
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .child(
                render_info_slot(model, window, cx).unwrap_or_else(|| div().flex_1().min_w(px(0.0)).into_any_element()),
            )
            .child(
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap(px(look.group_gap))
                    .child(div().flex_none().flex().items_center().gap(px(8.0)).when(
                        !model.page_size_options.is_empty(),
                        |slot| {
                            slot.when_some(model.page_size_label(), |slot, label| {
                                slot.child(
                                    div()
                                        .flex_none()
                                        .text_color(look.muted_text)
                                        .text_size(px(look.typography.size))
                                        .line_height(px(look.typography.line_height))
                                        .child(label.clone()),
                                )
                            })
                            .child(render_page_size_select(model, &look, &handlers))
                        },
                    ))
                    .child(render_page_indicator(model, &look))
                    .child(render_nav_group(model, &look, &self.theme, &handlers, true, window, cx)),
            )
            .into_any_element()
    }
}

pub struct PagingListViewBuilder<T: 'static> {
    list_builder: ListViewBuilder<T>,
    pager_builder: PagerBuilder,
}

impl<T: 'static> PagingListViewBuilder<T> {
    pub fn new(list_builder: ListViewBuilder<T>, pager_builder: PagerBuilder) -> Self {
        Self { list_builder, pager_builder }
    }

    /// Fill the parent height; see [`ListViewBuilder::fill_height`].
    pub fn fill_height(mut self) -> Self {
        self.list_builder = self.list_builder.fill_height();
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> PagingListView<T> {
        let fill_height = self.list_builder.model.fill_height;
        let list = cx.new(|cx| ListViewControl::from_builder(self.list_builder, cx));

        let (current_page, page_count, page_size, selected_count, total_rows) = list.read_with(cx, |list, _| {
            (
                list.current_page(),
                list.page_count(),
                list.page_size().unwrap_or(10),
                list.selected_indices().len(),
                list.items().len(),
            )
        });

        let mut pager_builder = self.pager_builder;
        // Viewport-driven page size: hide the manual "rows per page" control.
        if fill_height {
            pager_builder = pager_builder.page_size_options([]);
        } else if pager_builder.model.template_parameters.page_size_label.is_none() {
            pager_builder = pager_builder.page_size_label("Rows per page");
        }
        let pager_theme = pager_builder.model.theme.clone().unwrap_or_else(default_pager_theme);
        pager_builder = pager_builder.style(PagerStyle::MinimalEdge);
        pager_builder = pager_builder.template(Arc::new(ListViewPagerTemplate::new(pager_theme)));

        let pager = pager_builder
            .current_page(current_page)
            .page_count(page_count)
            .page_size(page_size)
            .info_text(selection_summary(selected_count, total_rows))
            .spawn(cx);

        cx.new(|cx| PagingListViewControl::new(list, pager, cx))
    }
}

fn selection_summary(selected_count: usize, total_rows: usize) -> String {
    format!("{} of {} row(s) selected.", selected_count, total_rows)
}
