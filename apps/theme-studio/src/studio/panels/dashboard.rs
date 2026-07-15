use std::sync::Arc;

use gpui_luma::controls::presenter::ControlPresenter;
use gpui_luma::controls::command::button::ButtonRenderModel;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{ButtonEvent, ControlIcon};
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma::controls::list_view::{ListSelectionMode, ListViewEvent, PagingListView};
use gpui_luma::controls::pager::PagerStyle;
use gpui_luma::controls::navigation_sidebar::NavigationSidebar;
use gpui_luma::controls::split_view::{SplitView, SplitViewEvent, SplitViewSeparatorVisibility, render_pane};
use gpui_luma::theme::ControlSize;
use gpui_luma::{column, column_emphasis, paging_list_view};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnTextSize};
use lucide_icons::Icon as LucideIcon;

use crate::studio::content_tabs::cards::panel_box_shadow;
use super::navigation_sidebar::{
    property_navigation_footer_nodes, property_navigation_nodes, INITIAL_PROPERTY_SELECTION_ID,
};
use super::task_list::{Task, build_task_rows, email_column, status_cell, tag_pill};

const DEFAULT_PAGE_SIZE: usize = 25;
const LIST_HEADER_TITLE: &str = "Documents";
const LIST_VIEW_OUTER_PADDING_PX: f32 = 16.0;

pub struct DashboardPanel {
    look: Arc<ShadcnLook>,
    split_view: Entity<SplitView>,
    navigation_sidebar: Entity<NavigationSidebar>,
    sidebar_toggle: IconButton,
    list_view: PagingListView<Task>,
    sidebar_collapsed: bool,
    _subscriptions: Vec<Subscription>,
}

impl DashboardPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let split_view = look
            .split_view("studio-dashboard-shell")
            .sidebar_width(px(280.0))
            .sidebar_min_width(px(220.0))
            .sidebar_max_width(px(420.0))
            .sidebar_collapsed_width(px(0.0))
            .collapsed(false)
            .resizable(true)
            .separator_visibility(SplitViewSeparatorVisibility::Hover)
            .spawn(cx);

        let navigation_sidebar = look
            .navigation_sidebar("studio-dashboard-nav")
            .title("Properties")
            .subtitle("Task workspace")
            .collapsible(false)
            .selected_id(INITIAL_PROPERTY_SELECTION_ID)
            .items(property_navigation_nodes())
            .footer_nodes(property_navigation_footer_nodes())
            .spawn(cx);

        let sidebar_toggle = look
            .content_only_icon_button("studio-dashboard-sidebar-toggle", LucideIcon::PanelLeft)
            .size(ControlSize::Sm)
            .spawn(cx);

        let tasks = build_task_rows();
        let list_view = paging_list_view! {
            list_view_theme = look.list_view_theme();
            id = "studio-dashboard-tasks";
            items = tasks;
            page_size = DEFAULT_PAGE_SIZE;
            pager = look
                .pager("studio-dashboard-tasks-pager")
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
                    .typography_style(model.look.label_typography)
                    .child(cells)
            };
        }
        .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&split_view, |panel, _, event: &SplitViewEvent, cx| {
            panel.handle_split_view_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&sidebar_toggle, |panel, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                panel.toggle_sidebar(cx);
            }
        }));
        subscriptions.push(cx.subscribe(&list_view, |_, _, _: &ListViewEvent, cx| {
            cx.notify();
        }));

        Self {
            look,
            split_view,
            navigation_sidebar,
            sidebar_toggle,
            list_view,
            sidebar_collapsed: false,
            _subscriptions: subscriptions,
        }
    }

    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.split_view.update(cx, |split_view, cx| {
            split_view.toggle_collapsed(cx);
        });
    }

    fn handle_split_view_event(&mut self, event: &SplitViewEvent, cx: &mut Context<Self>) {
        match event {
            SplitViewEvent::ResizeStart => {}
            SplitViewEvent::SidebarWidthChanged { .. } | SplitViewEvent::ResizeEnd { .. } => {
                cx.notify();
            }
            SplitViewEvent::CollapsedChanged { collapsed } => {
                self.sidebar_collapsed = *collapsed;
                cx.notify();
            }
        }
    }

    fn sync_sidebar_toggle_icon(&self, cx: &mut Context<Self>) {
        let icon = if self.sidebar_collapsed {
            ControlIcon::Lucide(LucideIcon::PanelLeftOpen)
        } else {
            ControlIcon::Lucide(LucideIcon::PanelLeft)
        };
        self.sidebar_toggle.update(cx, |button, cx| {
            button.set_presenter(sidebar_toggle_presenter(icon), cx);
        });
    }
}

impl Render for DashboardPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_sidebar_toggle_icon(cx);

        let navigation_sidebar = self.navigation_sidebar.clone();
        let list_view = self.list_view.clone();
        let sidebar_toggle = self.sidebar_toggle.clone();
        let look = self.look.clone();
        let chrome = look.chrome();

        self.split_view.update(cx, |split_view, cx| {
            split_view.set_panes(
                render_pane(move || navigation_sidebar.clone()),
                render_pane(move || {
                    div()
                        .size_full()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .border_l_1()
                        .border_color(chrome.border)
                        .bg(chrome.content_background)
                        .child(render_list_header(sidebar_toggle.clone(), &look))
                        .child(render_list_header_divider(chrome.border))
                        .child(
                            div()
                                .flex_1()
                                .min_h_0()
                                .px(px(LIST_VIEW_OUTER_PADDING_PX))
                                .pt(px(LIST_VIEW_OUTER_PADDING_PX))
                                .child(list_view.clone()),
                        )
                        .into_any_element()
                }),
                cx,
            );
        });

        div().size_full().min_h_0().p(px(24.0)).child(
            div()
                .size_full()
                .min_h_0()
                .flex()
                .flex_col()
                .overflow_hidden()
                .border_1()
                .border_color(chrome.border)
                .rounded(px(12.0))
                .bg(chrome.panel_background)
                .shadow(panel_box_shadow())
                .child(self.split_view.clone()),
        )
    }
}

fn render_list_header(sidebar_toggle: IconButton, look: &ShadcnLook) -> gpui::AnyElement {
    let chrome = look.chrome();
    let title_style = look.typography_scale(ShadcnTextSize::Lg);

    div()
        .id("studio-dashboard-list-header")
        .w_full()
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(8.0))
        .h(px(44.0))
        .px(px(12.0))
        .bg(chrome.panel_background)
        .child(sidebar_toggle)
        .child(div().h(px(16.0)).w(px(1.0)).bg(chrome.border))
        .child(div().typography_style(title_style).text_color(chrome.title_text).child(LIST_HEADER_TITLE))
        .into_any_element()
}

fn render_list_header_divider(border: gpui::Hsla) -> gpui::AnyElement {
    div().w_full().flex_shrink_0().h(px(1.0)).bg(border).into_any_element()
}

fn sidebar_toggle_presenter(icon: ControlIcon) -> ControlPresenter<ButtonRenderModel<()>> {
    Arc::new(move |_, _| match &icon {
        ControlIcon::Lucide(lucide) => div()
            .font_family("lucide")
            .text_size(px(16.0))
            .child(char::from(*lucide).to_string())
            .into_any_element(),
        ControlIcon::SvgPath(path) => gpui::svg().size(px(16.0)).path(path.clone()).into_any_element(),
    })
}
