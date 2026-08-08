use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonRenderModel, ControlIcon};
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma::controls::list_view::{ListSelectionMode, ListViewEvent, PagingListView};
use gpui_luma::controls::pager::PagerStyle;
use gpui_luma::controls::presenter::ControlPresenter;
use gpui_luma::controls::sidebar::{SidebarCollapsible, SidebarControl, SidebarEvent};
use gpui_luma::theme::ControlSize;
use gpui_luma::{column, column_emphasis, paging_list_view};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnTextSize, with_look};
use lucide_icons::Icon as LucideIcon;

use super::sidebar::{INITIAL_PROPERTY_SELECTION_ID, property_sidebar};
use super::task_list::{Task, build_task_rows, email_column, status_cell, tag_pill};

const DEFAULT_PAGE_SIZE: usize = 25;
const LIST_HEADER_TITLE: &str = "Documents";
const LIST_VIEW_OUTER_PADDING_PX: f32 = 16.0;
/// Matches controls → sidebar preview shell.
const SHELL_RADIUS_PX: f32 = 12.0;
const CONTENT_INSET_PX: f32 = 10.0;
const CONTENT_RADIUS_PX: f32 = 12.0;

pub struct DashboardPanel {
    look: Arc<ShadcnLook>,
    sidebar: Entity<SidebarControl>,
    sidebar_toggle: IconButton,
    list_view: PagingListView<Task>,
    sidebar_open: bool,
    _subscriptions: Vec<Subscription>,
}

impl DashboardPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let sidebar = look
            .sidebar_control("studio-dashboard-nav")
            .default_open(true)
            .collapsible(SidebarCollapsible::Icon)
            .selected_id(INITIAL_PROPERTY_SELECTION_ID)
            .sidebar(
                property_sidebar(&look, "studio-dashboard-nav-panel", "Properties", "Task workspace")
                    .rail(look.sidebar_rail()),
            )
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
        .fill_height()
        .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&sidebar, |panel, _, event: &SidebarEvent, cx| {
            if let SidebarEvent::OpenChanged { open, .. } = event {
                panel.sidebar_open = *open;
                cx.notify();
            }
        }));
        subscriptions.push(cx.subscribe(&sidebar_toggle, |panel, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                panel.sidebar.update(cx, |sidebar, cx| sidebar.toggle_open(cx));
            }
        }));
        subscriptions.push(cx.subscribe(&list_view, |_, _, _: &ListViewEvent, cx| {
            cx.notify();
        }));

        Self { look, sidebar, sidebar_toggle, list_view, sidebar_open: true, _subscriptions: subscriptions }
    }

    fn sync_sidebar_toggle_icon(&self, cx: &mut Context<Self>) {
        let icon = if self.sidebar_open {
            ControlIcon::Lucide(LucideIcon::PanelLeft)
        } else {
            ControlIcon::Lucide(LucideIcon::PanelLeftOpen)
        };
        self.sidebar_toggle.update(cx, |button, cx| {
            button.set_presenter(sidebar_toggle_presenter(icon), cx);
        });
    }
}

impl Render for DashboardPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_sidebar_toggle_icon(cx);

        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();
            let metrics = look.sidebar_metric_scale();
            let sidebar_width = if self.sidebar_open {
                metrics.width_expanded
            } else {
                metrics.width_icon_rail
            };
            let sidebar_bg = look.token_color("sidebar").unwrap_or(chrome.panel_background);
            let title_style = look.typography_scale(ShadcnTextSize::Lg);

            div().size_full().min_h_0().p(px(24.0)).child(
                div()
                    .id("studio-dashboard-shell")
                    .size_full()
                    .min_h_0()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    .rounded(px(SHELL_RADIUS_PX))
                    .border_1()
                    .border_color(chrome.border)
                    .bg(sidebar_bg)
                    .child(
                        div()
                            .id("studio-dashboard-rail")
                            .flex_none()
                            .w(sidebar_width)
                            .h_full()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .child(
                                div().flex_1().min_h(px(0.0)).w_full().overflow_hidden().child(self.sidebar.clone()),
                            ),
                    )
                    .child(
                        div()
                            .id("studio-dashboard-content")
                            .flex_1()
                            .min_w(px(0.0))
                            .h_full()
                            .p(px(CONTENT_INSET_PX))
                            .child(
                                div()
                                    .size_full()
                                    .min_h(px(0.0))
                                    .flex()
                                    .flex_col()
                                    .overflow_hidden()
                                    .rounded(px(CONTENT_RADIUS_PX))
                                    .border_1()
                                    .border_color(chrome.border)
                                    .bg(chrome.content_background)
                                    .child(
                                        div()
                                            .id("studio-dashboard-list-header")
                                            .w_full()
                                            .flex_shrink_0()
                                            .flex()
                                            .items_center()
                                            .gap(px(8.0))
                                            .h(px(44.0))
                                            .px(px(12.0))
                                            .border_b_1()
                                            .border_color(chrome.border)
                                            .bg(chrome.content_background)
                                            .child(self.sidebar_toggle.clone())
                                            .child(div().h(px(16.0)).w(px(1.0)).bg(chrome.border))
                                            .child(
                                                div()
                                                    .typography_style(title_style)
                                                    .text_color(chrome.title_text)
                                                    .child(LIST_HEADER_TITLE),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_h(px(0.0))
                                            .w_full()
                                            .p(px(LIST_VIEW_OUTER_PADDING_PX))
                                            .child(div().size_full().min_h_0().child(self.list_view.clone())),
                                    ),
                            ),
                    ),
            )
        })
    }
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
