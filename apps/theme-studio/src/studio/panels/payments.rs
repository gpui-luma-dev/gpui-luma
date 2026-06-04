use std::sync::Arc;

use gpui::{Context, Entity, MouseButton, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::column;
use gpui_luma::column_numeric;
use gpui_luma::column_text;
use gpui_luma::controls::checkbox::Checkbox;
use gpui_luma::controls::command::button::{Button, ButtonEvent};
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

/// Data-table row height: fits text rows without reserving full [`ControlSize`] command height.
fn payments_row_height(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 28.0,
        ControlSize::Md => 32.0,
        ControlSize::Lg => 36.0,
    }
}

#[derive(Clone)]
struct PaymentRow {
    status: &'static str,
    email: SharedString,
    amount: SharedString,
}

pub struct PaymentsPanel {
    radix_theme: Arc<RadixTheme>,
    list_view: Entity<ListViewControl<PaymentRow>>,
    row_checkboxes: Arc<Vec<Checkbox>>,
    add_button: Entity<Button>,
    selected_count: usize,
    _subscriptions: Vec<Subscription>,
}

impl PaymentsPanel {
    pub fn new(cx: &mut Context<Self>, radix_theme: Arc<RadixTheme>, size: ControlSize) -> Self {
        let row_checkboxes = Arc::new(spawn_row_checkboxes(radix_theme.clone(), sample_payments().len(), cx));
        let list_view = radix_theme
            .list_view("studio-payments")
            .items(sample_payments())
            .multiple()
            .select_on_row_click(false)
            .size(size)
            .visible_rows(PAYMENTS_VISIBLE_ROWS)
            .visible_row_height(payments_row_height(size))
            .paged(PAYMENTS_PAGE_SIZE)
            .row_label(|row| row.email.clone())
            .grid_view(payment_columns(Arc::clone(&row_checkboxes)))
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&list_view, |panel, _, event, cx| {
            if let ListViewEvent::SelectionChanged { .. } = event {
                panel.selected_count = panel.list_view.read(cx).selected_indices().len();
                panel.sync_checkboxes_from_list(cx);
                cx.notify();
            }
        }));

        for (index, checkbox) in row_checkboxes.iter().enumerate() {
            subscriptions.push(cx.subscribe(checkbox, move |panel, _, event, cx| {
                if matches!(event, ButtonEvent::Click) {
                    panel.toggle_row_selection(index, cx);
                }
            }));
        }

        Self {
            add_button: radix_theme.primary_button("payments-add").label("Add Payment").size(size).spawn(cx),
            list_view,
            row_checkboxes,
            radix_theme,
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

fn spawn_row_checkboxes(
    radix_theme: Arc<RadixTheme>,
    row_count: usize,
    cx: &mut Context<PaymentsPanel>,
) -> Vec<Checkbox> {
    (0..row_count)
        .map(|index| {
            radix_theme
                .primary_checkbox(format!("studio-payments-row-{index}"))
                .with_data(false)
                .size(ControlSize::Sm)
                .indicator_only()
                .tab_stop(false)
                .spawn(cx)
        })
        .collect()
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

fn payment_columns(row_checkboxes: Arc<Vec<Checkbox>>) -> Vec<ListViewColumn<PaymentRow>> {
    vec![
        ListViewColumn::fixed_control("", 48.0, selection_checkbox_column(row_checkboxes)),
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

fn selection_checkbox_column(row_checkboxes: Arc<Vec<Checkbox>>) -> ListViewColumnCellTemplate<PaymentRow> {
    Arc::new(move |model: &ListViewRowRenderModel<'_, PaymentRow>, _, _| {
        row_checkboxes[model.index].clone().into_any_element()
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
