use std::sync::Arc;

use gpui::{Context, Entity, MouseButton, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::column;
use gpui_luma::column_numeric;
use gpui_luma::column_text;
use gpui_luma::controls::command::button::Button;
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma::controls::list_view::{
    ListViewColumn, ListViewColumnCellTemplate, ListViewControl, ListViewEvent, ListViewRowRenderModel,
};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::theme::radix::prelude::*;
use gpui_luma::theme::{ControlSize, RadixTheme};
use gpui_luma::{hstack, vstack};
use lucide_icons::Icon as LucideIcon;

use super::common::{card, card_header};

const PAYMENTS_CARD_WIDTH: f32 = 720.0;
const PAYMENTS_PAGE_SIZE: usize = 6;
const PAYMENTS_VISIBLE_ROWS: usize = 6;

#[derive(Clone)]
struct PaymentRow {
    status: &'static str,
    email: SharedString,
    amount: SharedString,
}

pub struct PaymentsPanel {
    radix_theme: Arc<RadixTheme>,
    list_view: Entity<ListViewControl<PaymentRow>>,
    add_button: Entity<Button>,
    selected_count: usize,
    _subscriptions: Vec<Subscription>,
}

impl PaymentsPanel {
    pub fn new(cx: &mut Context<Self>, radix_theme: Arc<RadixTheme>, size: ControlSize) -> Self {
        let list_view = radix_theme
            .list_view("studio-payments")
            .items(sample_payments())
            .multiple()
            .visible_rows(PAYMENTS_VISIBLE_ROWS)
            .paged(PAYMENTS_PAGE_SIZE)
            .row_label(|row| row.email.clone())
            .grid_view(payment_columns())
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&list_view, |panel, _, event, cx| {
            if let ListViewEvent::SelectionChanged { selected_indices } = event {
                panel.selected_count = selected_indices.len();
                cx.notify();
            }
        }));

        Self {
            add_button: radix_theme.primary_button("payments-add").label("Add Payment").size(size).spawn(cx),
            list_view,
            radix_theme,
            selected_count: 0,
            _subscriptions: subscriptions,
        }
    }
}

impl Render for PaymentsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();
        let total_rows = PAYMENTS_PAGE_SIZE;
        let at_first = self.list_view.read(cx).current_page() == 0;
        let at_last = self.list_view.read(cx).current_page() + 1 >= self.list_view.read(cx).page_count().max(1);

        card(
            PAYMENTS_CARD_WIDTH,
            chrome.border,
            chrome.panel_background,
            vstack! {
                gap=12;
                hstack! {
                    justify=between align=start gap=12;
                    card_header("Payments", "Manage your payments.", chrome.title_text, chrome.muted_text),
                    self.add_button.clone(),
                },
                div()
                    .w_full()
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(chrome.border)
                    .overflow_hidden()
                    .child(self.list_view.clone()),
                hstack! {
                    justify=between align=center gap=12;
                    format!("{} of {total_rows} row(s) selected.", self.selected_count),
                    hstack! {
                        gap=8 align=center;
                        pagination_button(cx, "payments-prev", "Previous", at_first, |panel, cx| {
                            panel.list_view.update(cx, |list, cx| list.prev_page(cx));
                        }),
                        pagination_button(cx, "payments-next", "Next", at_last, |panel, cx| {
                            panel.list_view.update(cx, |list, cx| list.next_page(cx));
                        }),
                    },
                }
                .w_full()
                .text_size(px(11.0))
                .line_height(px(14.0))
                .text_color(chrome.muted_text),
            }
            .w_full()
            .overflow_hidden(),
        )
    }
}

fn pagination_button(
    cx: &mut Context<PaymentsPanel>,
    id: &'static str,
    label: &'static str,
    disabled: bool,
    action: fn(&mut PaymentsPanel, &mut Context<PaymentsPanel>),
) -> impl IntoElement {
    div()
        .id(id)
        .px(px(12.0))
        .py(px(6.0))
        .rounded(px(6.0))
        .border_1()
        .border_color(gpui::hsla(0.0, 0.0, 1.0, 0.12))
        .text_size(px(11.0))
        .line_height(px(14.0))
        .when(disabled, |slot| slot.opacity(0.45).cursor_default())
        .when(!disabled, |slot| slot.cursor_pointer())
        .child(label)
        .when(!disabled, |slot| {
            slot.on_mouse_down(MouseButton::Left, cx.listener(move |panel, _, _, cx| action(panel, cx)))
        })
}

fn payment_columns() -> Vec<ListViewColumn<PaymentRow>> {
    vec![
        ListViewColumn::fixed("", 36.0, selection_checkbox_column()),
        column_text!("Status", width = 108 => |row: &PaymentRow| row.status),
        column_text!("Email" => |row: &PaymentRow| row.email.clone()),
        column_numeric!("Amount", width = 96 => |row: &PaymentRow| row.amount.clone()),
        column!("", width = 40 => |_row: &PaymentRow| {
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_center()
                .child(lucide_glyph(LucideIcon::EllipsisVertical))
        }),
    ]
}

fn selection_checkbox_column() -> ListViewColumnCellTemplate<PaymentRow> {
    Arc::new(|model: &ListViewRowRenderModel<'_, PaymentRow>, _, _| {
        selection_checkbox(model.selected).into_any_element()
    })
}

fn selection_checkbox(selected: bool) -> impl IntoElement {
    div()
        .size(px(16.0))
        .rounded(px(4.0))
        .border_1()
        .border_color(gpui::hsla(0.0, 0.0, 1.0, 0.25))
        .flex()
        .items_center()
        .justify_center()
        .when(selected, |slot| {
            slot.bg(gpui::hsla(0.55, 0.45, 0.45, 1.0))
                .border_color(gpui::hsla(0.55, 0.45, 0.45, 1.0))
                .text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                .text_size(px(10.0))
                .child("✓")
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
    ]
}
