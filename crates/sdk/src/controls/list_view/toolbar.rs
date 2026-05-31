use std::sync::Arc;

use gpui::{AnyElement, App, ClickEvent, Context, EventEmitter, Render, Window, div, prelude::*, px};
use lucide_icons::Icon as LucideIcon;

use crate::controls::icon::lucide_glyph;
use crate::theme::RadixTheme;

const PAGE_SIZE_OPTIONS: [usize; 2] = [10, 25];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PagingToolbarLayout {
    pub selected_count: usize,
    pub total_rows: usize,
    pub current_page: usize,
    pub page_count: usize,
    pub page_size: usize,
}

impl PagingToolbarLayout {
    fn selection_summary(&self) -> String {
        format!("{} of {} row(s) selected.", self.selected_count, self.total_rows)
    }

    fn page_indicator(&self) -> String {
        format!("Page {} of {}", self.current_page + 1, self.page_count.max(1))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PagingToolbarEvent {
    FirstPage,
    PrevPage,
    NextPage,
    LastPage,
    SetPageSize(usize),
}

pub type PagingToolbarTemplate =
    Arc<dyn Fn(&PagingToolbarLayout, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static>;

pub struct PagingToolbar {
    theme: Arc<RadixTheme>,
    layout: PagingToolbarLayout,
    page_size_open: bool,
    custom_template: Option<PagingToolbarTemplate>,
}

impl EventEmitter<PagingToolbarEvent> for PagingToolbar {}

impl PagingToolbar {
    pub fn new(theme: Arc<RadixTheme>, layout: PagingToolbarLayout) -> Self {
        Self { theme, layout, page_size_open: false, custom_template: None }
    }

    pub fn with_custom_template(mut self, template: PagingToolbarTemplate) -> Self {
        self.custom_template = Some(template);
        self
    }

    pub fn set_layout(&mut self, layout: PagingToolbarLayout, cx: &mut Context<Self>) {
        if self.layout != layout {
            self.layout = layout;
            cx.notify();
        }
    }

    pub fn update_selection(&mut self, selected_count: usize, total_rows: usize, cx: &mut Context<Self>) {
        if self.layout.selected_count != selected_count || self.layout.total_rows != total_rows {
            self.layout.selected_count = selected_count;
            self.layout.total_rows = total_rows;
            cx.notify();
        }
    }

    pub fn update_page(&mut self, current_page: usize, page_size: usize, page_count: usize, cx: &mut Context<Self>) {
        if self.layout.current_page != current_page
            || self.layout.page_size != page_size
            || self.layout.page_count != page_count
        {
            self.layout.current_page = current_page;
            self.layout.page_size = page_size;
            self.layout.page_count = page_count;
            cx.notify();
        }
    }
}

impl Render for PagingToolbar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(template) = &self.custom_template {
            return template(&self.layout, window, cx);
        }

        let chrome = self.theme.chrome();
        let layout = self.layout;
        let at_first = layout.current_page == 0;
        let at_last = layout.current_page + 1 >= layout.page_count.max(1);

        div()
            .w_full()
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(12.0))
            .py(px(8.0))
            .text_color(chrome.muted_text)
            .text_size(px(11.0))
            .line_height(px(14.0))
            .child(div().flex_1().min_w(px(0.0)).truncate().child(layout.selection_summary()))
            .child(
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap(px(16.0))
                    .child(
                        div()
                            .flex_none()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(div().flex_none().child("Rows per page"))
                            .child(render_page_size_select(cx, &self.theme, layout.page_size, self.page_size_open)),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(11.0))
                            .line_height(px(14.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(chrome.body_text)
                            .child(layout.page_indicator()),
                    )
                    .child(
                        div()
                            .flex_none()
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .child(render_nav_button(cx, &self.theme, "<<", at_first, PagingToolbarEvent::FirstPage))
                            .child(render_nav_button(cx, &self.theme, "<", at_first, PagingToolbarEvent::PrevPage))
                            .child(render_nav_button(cx, &self.theme, ">", at_last, PagingToolbarEvent::NextPage))
                            .child(render_nav_button(cx, &self.theme, ">>", at_last, PagingToolbarEvent::LastPage)),
                    ),
            )
            .into_any_element()
    }
}

fn render_page_size_select(
    cx: &mut Context<PagingToolbar>,
    theme: &RadixTheme,
    page_size: usize,
    open: bool,
) -> impl IntoElement {
    let chrome = theme.chrome();

    div()
        .relative()
        .flex_none()
        .w(px(56.0))
        .on_mouse_down_out(cx.listener(|this, _, _, cx| {
            if this.page_size_open {
                this.page_size_open = false;
                cx.notify();
            }
        }))
        .child(
            div()
                .id("paging-page-size-trigger")
                .w_full()
                .h(px(28.0))
                .flex()
                .items_center()
                .justify_between()
                .gap(px(4.0))
                .px(px(8.0))
                .rounded(px(6.0))
                .border_1()
                .border_color(chrome.border)
                .bg(chrome.panel_background)
                .text_color(chrome.body_text)
                .cursor_pointer()
                .child(format!("{page_size}"))
                .child(div().text_color(chrome.muted_text).child(lucide_glyph(if open {
                    LucideIcon::ChevronUp
                } else {
                    LucideIcon::ChevronDown
                })))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.page_size_open = !this.page_size_open;
                    cx.notify();
                })),
        )
        .when(open, |slot| {
            slot.child(
                div()
                    .absolute()
                    .top(px(32.0))
                    .left(px(0.0))
                    .occlude()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .p(px(4.0))
                    .min_w(px(56.0))
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(chrome.border)
                    .bg(chrome.panel_background)
                    .shadow_md()
                    .children(PAGE_SIZE_OPTIONS.iter().copied().map(|option| {
                        let is_selected = option == page_size;
                        div()
                            .id(format!("paging-page-size-{option}"))
                            .px(px(8.0))
                            .py(px(4.0))
                            .rounded(px(4.0))
                            .when(is_selected, |row| row.bg(chrome.border))
                            .text_size(px(11.0))
                            .line_height(px(14.0))
                            .text_color(chrome.body_text)
                            .cursor_pointer()
                            .child(format!("{option}"))
                            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                this.page_size_open = false;
                                cx.emit(PagingToolbarEvent::SetPageSize(option));
                                cx.notify();
                            }))
                    })),
            )
        })
}

fn render_nav_button(
    cx: &mut Context<PagingToolbar>,
    theme: &RadixTheme,
    label: &'static str,
    disabled: bool,
    event: PagingToolbarEvent,
) -> impl IntoElement {
    let chrome = theme.chrome();

    div()
        .id(format!("paging-nav-{label}"))
        .flex_none()
        .size(px(28.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(6.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.panel_background)
        .text_color(chrome.body_text)
        .text_size(px(11.0))
        .line_height(px(14.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .when(disabled, |slot| slot.opacity(0.4))
        .when(!disabled, |slot| slot.cursor_pointer())
        .child(label)
        .when(!disabled, |slot| {
            slot.on_click(cx.listener(move |_, _, _, cx| {
                cx.emit(event);
            }))
        })
}
