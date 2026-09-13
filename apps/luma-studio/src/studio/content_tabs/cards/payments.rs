use std::sync::Arc;

use gpui::{Context, Entity, Render, SharedString, Subscription, Window, div, prelude::*, px};
use lucide_svg_static::Icon as LucideIcon;
use luma::column_numeric;
use luma::column_text;
use luma::controls::checkbox::{Checkbox, CheckboxEvent};
use luma::controls::button::{Button, ButtonEvent};
use luma::controls::list_view::{
    ListViewColumn, ListViewColumnCellTemplate, ListViewControl, ListViewEvent, ListViewRowRenderModel,
};
use luma::infra::menu_item::MenuItem;
use luma::controls::popup_menu::PopupMenu;
use luma::infra::presenter::HasPresenter;
use luma_look_shadcn as shadcn;
use luma_look_shadcn::ShadcnLook;
use luma::{hstack, vstack};

use super::common::titled_card;

const PAYMENTS_CARD_WIDTH: f32 = 720.0;
const PAYMENTS_PAGE_SIZE: usize = 6;
const PAYMENTS_VISIBLE_ROWS: usize = 6;

/// Data-table row height: fits text rows without reserving full command height.
fn payments_row_height(size: shadcn::ShadcnSize) -> f32 {
    match size {
        shadcn::ShadcnSize::Sm => 28.0,
        shadcn::ShadcnSize::Md => 32.0,
        shadcn::ShadcnSize::Lg => 36.0,
    }
}

#[derive(Clone)]
struct PaymentRow {
    status: &'static str,
    email: SharedString,
    amount: SharedString,
}

pub struct PaymentsPanel {
    look: Arc<ShadcnLook>,
    list_view: Entity<ListViewControl<PaymentRow>>,
    row_checkboxes: Arc<Vec<Checkbox>>,
    prev_button: Entity<Button>,
    next_button: Entity<Button>,
    selected_count: usize,
    _subscriptions: Vec<Subscription>,
}

impl PaymentsPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>, size: shadcn::ShadcnSize) -> Self {
        let payments = sample_payments();
        let row_checkboxes = Arc::new(spawn_row_checkboxes(look.clone(), payments.len(), cx));
        let row_menus = Arc::new(spawn_row_menus(look.clone(), payments.len(), cx));
        let list_size = shadcn::ShadcnSize::Sm;
        let list_view = shadcn::ListView::new("studio-payments")
            .look(look.as_ref())
            .items(payments)
            .multiple()
            .select_on_row_click(false)
            .size(list_size)
            .visible_rows(PAYMENTS_VISIBLE_ROWS)
            .visible_row_height(payments_row_height(list_size))
            .paged(PAYMENTS_PAGE_SIZE)
            .row_label(|row| row.email.clone())
            .grid_view(payment_columns(Arc::clone(&row_checkboxes), Arc::clone(&row_menus)))
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&list_view, |panel, _, event, cx| {
            match event {
                ListViewEvent::SelectionChanged { .. } => {
                    panel.selected_count = panel.list_view.read(cx).selected_indices().len();
                    panel.sync_checkboxes_from_list(cx);
                }
                ListViewEvent::PageChanged { .. } => {}
                _ => return,
            }
            cx.notify();
        }));

        for (index, checkbox) in row_checkboxes.iter().enumerate() {
            subscriptions.push(cx.subscribe(checkbox, move |panel, _, event, cx| {
                if matches!(event, CheckboxEvent::Change { .. }) {
                    panel.toggle_row_selection(index, cx);
                }
            }));
        }

        let prev_button = shadcn::Button::new("payments-prev")
            .look(look.as_ref())
            .outline()
            .label("Previous")
            .size(size)
            .spawn(cx);
        let next_button = shadcn::Button::new("payments-next")
            .look(look.as_ref())
            .outline()
            .label("Next")
            .size(size)
            .spawn(cx);
        subscriptions.push(cx.subscribe(&prev_button, |panel, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                panel.list_view.update(cx, |list, cx| list.prev_page(cx));
            }
        }));
        subscriptions.push(cx.subscribe(&next_button, |panel, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                panel.list_view.update(cx, |list, cx| list.next_page(cx));
            }
        }));

        Self {
            prev_button,
            next_button,
            list_view,
            row_checkboxes,
            look,
            selected_count: 0,
            _subscriptions: subscriptions,
        }
    }

    fn toggle_row_selection(&mut self, index: usize, cx: &mut Context<Self>) {
        self.list_view.update(cx, |list, cx| {
            list.toggle_selected_index(index, cx);
        });
    }

    fn sync_checkboxes_from_list(&self, cx: &mut Context<Self>) {
        let selected: Vec<_> = self.list_view.read(cx).selected_indices().to_vec();
        for (index, checkbox) in self.row_checkboxes.iter().enumerate() {
            let checked = selected.contains(&index);
            checkbox.update(cx, |button, cx| {
                if *button.data() != checked {
                    button.set_data(checked, cx);
                }
            });
        }
    }
}

impl Render for PaymentsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let total_rows = self.list_view.read(cx).items().len();
        let at_first = self.list_view.read(cx).current_page() == 0;
        let at_last = self.list_view.read(cx).current_page() + 1 >= self.list_view.read(cx).page_count().max(1);
        let selected_count = self.selected_count;
        let list_view = self.list_view.clone();
        let prev_button = self.prev_button.clone();
        let next_button = self.next_button.clone();

        self.prev_button.update(cx, |button, cx| button.set_enabled(!at_first, cx));
        self.next_button.update(cx, |button, cx| button.set_enabled(!at_last, cx));

        titled_card(
            "luma-studio-payments-card",
            &self.look,
            PAYMENTS_CARD_WIDTH,
            "Payments",
            "Manage your payments.",
            move |_, _| {
                vstack! {
                    gap=12;
                    div()
                        .w_full()
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(chrome.border)
                        .overflow_hidden()
                        .child(list_view.clone()),
                    hstack! {
                        justify=between align=center gap=12;
                        format!("{} of {total_rows} row(s) selected.", selected_count),
                        hstack! {
                            gap=8 align=center;
                            prev_button.clone(),
                            next_button.clone(),
                        },
                    }
                    .w_full()
                    .text_xs()
                    .line_height(px(14.0))
                    .text_color(chrome.muted_text),
                }
                .w_full()
                .overflow_hidden()
                .into_any_element()
            },
            _window,
            cx,
        )
    }
}

fn spawn_row_checkboxes(look: Arc<ShadcnLook>, row_count: usize, cx: &mut Context<PaymentsPanel>) -> Vec<Checkbox> {
    (0..row_count)
        .map(|index| {
            shadcn::Checkbox::new(format!("studio-payments-row-{index}"))
                .look(look.as_ref())
                .primary()
                .with_data(false)
                .size(shadcn::ShadcnSize::Sm)
                .indicator_only()
                .tab_stop(false)
                .spawn(cx)
        })
        .collect()
}

fn spawn_row_menus(look: Arc<ShadcnLook>, row_count: usize, cx: &mut Context<PaymentsPanel>) -> Vec<Entity<PopupMenu>> {
    (0..row_count)
        .map(|index| {
            shadcn::PopupMenu::new(format!("studio-payments-row-menu-{index}"))
                .look(look.as_ref())
                .ghost()
                .size(shadcn::ShadcnSize::Sm)
                .icon(LucideIcon::EllipsisVertical)
                .items(payment_action_items())
                .spawn(cx)
        })
        .collect()
}

fn payment_action_items() -> Vec<MenuItem> {
    vec![
        MenuItem::new("view").label("View"),
        MenuItem::new("copy").label("Copy"),
        MenuItem::new("delete").label("Delete"),
    ]
}

fn payment_columns(
    row_checkboxes: Arc<Vec<Checkbox>>,
    row_menus: Arc<Vec<Entity<PopupMenu>>>,
) -> Vec<ListViewColumn<PaymentRow>> {
    vec![
        ListViewColumn::fixed_control("", 48.0, selection_checkbox_column(row_checkboxes)),
        ListViewColumn::fixed_control("Status", 72.0, |row: &PaymentRow| status_icon_cell(row.status)),
        column_text!("Email" => |row: &PaymentRow| row.email.clone()),
        column_numeric!("Amount", width = 96 => |row: &PaymentRow| row.amount.clone()),
        ListViewColumn::fixed_control("", 40.0, row_menu_column(row_menus)),
    ]
}

fn status_icon_cell(status: &'static str) -> impl IntoElement {
    let (icon, color) = match status {
        "Success" => (LucideIcon::CircleCheck, gpui::hsla(0.35, 0.7, 0.45, 1.0)),
        "Failed" => (LucideIcon::CircleX, gpui::hsla(0.0, 0.7, 0.55, 1.0)),
        "Processing" => (LucideIcon::LoaderCircle, gpui::hsla(0.58, 0.75, 0.5, 1.0)),
        _ => (LucideIcon::Circle, gpui::hsla(0.0, 0.0, 0.55, 1.0)),
    };

    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .child(luma::infra::icon::lucide_icon(icon, color, 16.0))
}

fn selection_checkbox_column(row_checkboxes: Arc<Vec<Checkbox>>) -> ListViewColumnCellTemplate<PaymentRow> {
    Arc::new(move |model: &ListViewRowRenderModel<'_, PaymentRow>, _, _| {
        row_checkboxes[model.index].clone().into_any_element()
    })
}

fn row_menu_column(row_menus: Arc<Vec<Entity<PopupMenu>>>) -> ListViewColumnCellTemplate<PaymentRow> {
    Arc::new(move |model: &ListViewRowRenderModel<'_, PaymentRow>, _, _| {
        div()
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .child(row_menus[model.index].clone())
            .into_any_element()
    })
}

fn sample_payments() -> Vec<PaymentRow> {
    vec![
        PaymentRow { status: "Success", email: "ken99@example.com".into(), amount: "$316.00".into() },
        PaymentRow { status: "Success", email: "abe45@example.com".into(), amount: "$242.00".into() },
        PaymentRow { status: "Processing", email: "Mason@example.com".into(), amount: "$837.00".into() },
        PaymentRow { status: "Failed", email: "so456@example.com".into(), amount: "$721.00".into() },
        PaymentRow { status: "Pending", email: "lee39@example.com".into(), amount: "$150.00".into() },
        PaymentRow { status: "Success", email: "ashain@example.com".into(), amount: "$410.00".into() },
        PaymentRow { status: "Success", email: "noah12@example.com".into(), amount: "$529.00".into() },
        PaymentRow { status: "Processing", email: "ava88@example.com".into(), amount: "$198.00".into() },
        PaymentRow { status: "Pending", email: "mia27@example.com".into(), amount: "$364.00".into() },
        PaymentRow { status: "Failed", email: "liam63@example.com".into(), amount: "$912.00".into() },
        PaymentRow { status: "Success", email: "emma14@example.com".into(), amount: "$275.00".into() },
        PaymentRow { status: "Processing", email: "oliver52@example.com".into(), amount: "$648.00".into() },
        PaymentRow { status: "Success", email: "sophia31@example.com".into(), amount: "$483.00".into() },
        PaymentRow { status: "Pending", email: "james76@example.com".into(), amount: "$207.00".into() },
        PaymentRow { status: "Failed", email: "amelia08@example.com".into(), amount: "$756.00".into() },
        PaymentRow { status: "Success", email: "henry45@example.com".into(), amount: "$331.00".into() },
        PaymentRow { status: "Processing", email: "isla19@example.com".into(), amount: "$584.00".into() },
        PaymentRow { status: "Success", email: "charlie67@example.com".into(), amount: "$429.00".into() },
    ]
}
