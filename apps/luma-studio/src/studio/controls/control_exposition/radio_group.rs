//! Radio group control exposition — gallery-aligned vertical, horizontal, indented, and card layouts.

use std::sync::Arc;

use gpui::{AnyElement, Context, Div, Entity, Render, Subscription, Window, div, prelude::*, px};
use luma::controls::control_group::ControlGroupItemElementTemplate;
use luma::controls::radio_group::{
    self as sdk_radio_group, RadioGroup, RadioGroupEvent, RadioGroupItem, RadioGroupItemLike, SelectionMode,
};
use luma_look_shadcn::prelude::*;
use luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

const DENSITY_OPTION_COUNT: usize = 3;
const PANE_EXAMPLE_GAP: f32 = 24.0;
const EXAMPLE_CONTENT_GAP: f32 = 8.0;
const EXAMPLE_BORDER_RADIUS: f32 = 6.0;
const EXAMPLE_BORDER_WIDTH: f32 = 1.0;
const EXAMPLE_BORDER_OPACITY: f32 = 0.24;
const EXAMPLE_PADDING_X: f32 = 12.0;
const EXAMPLE_PADDING_Y: f32 = 12.0;
const INDENTED_EXAMPLE_PADDING_Y: f32 = 6.0;
const INDENTED_BUTTON_INDENT_STEP: f32 = 24.0;
const INDENTED_BUTTON_VERTICAL_OVERLAP: f32 = 10.0;
const FIRST_INDENTED_BUTTON_INDEX: usize = 0;
const EXAMPLE_LABEL_TEXT_SIZE: f32 = 12.0;
const EXAMPLE_LABEL_LINE_HEIGHT: f32 = 16.0;
const CHOICE_TEXT_SIZE: f32 = 12.0;
const CHOICE_LINE_HEIGHT: f32 = 16.0;
const DELIVERY_OPTION_COUNT: usize = 4;
const DELIVERY_GROUP_GAP: f32 = 4.0;
const DELIVERY_CARD_MIN_WIDTH: f32 = 116.0;
const DELIVERY_CARD_HEIGHT: f32 = 80.0;
const DELIVERY_CARD_RADIUS: f32 = 8.0;
const DELIVERY_CARD_PADDING_X: f32 = 12.0;
const DELIVERY_DAY_TEXT_SIZE: f32 = 16.0;
const DELIVERY_DAY_LINE_HEIGHT: f32 = 22.0;
const DELIVERY_DATE_TEXT_SIZE: f32 = 14.0;
const DELIVERY_DATE_LINE_HEIGHT: f32 = 20.0;
const DELIVERY_SELECTED_BORDER_WIDTH: f32 = 0.0;
const DELIVERY_FOCUS_BORDER_WIDTH: f32 = 2.0;
const DELIVERY_DISABLED_OPACITY: f32 = 0.56;

pub struct RadioGroupControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    vertical_group: RadioGroup<RadioGroupItem>,
    horizontal_group: RadioGroup<RadioGroupItem>,
    indented_group: RadioGroup<RadioGroupItem>,
    delivery_group: RadioGroup<DeliveryWindowItem>,
    vertical_choice: String,
    horizontal_choice: String,
    indented_choice: String,
    delivery_choice: String,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl RadioGroupControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("radio-group").expect("radio-group catalog entry");

        let secondary_vertical = look.radio_group_template(ShadcnButtonStyle::Secondary);
        let secondary_horizontal = look.radio_group_horizontal_template(ShadcnButtonStyle::Secondary);
        let radio_template = look.radio_button_template(ShadcnButtonStyle::Secondary);

        let vertical_group = look
            .radio_group("controls-doc-radio-group-vertical")
            .template(secondary_vertical)
            .items(density_items())
            .selected(Density::Comfortable.id())
            .spawn(cx);
        let horizontal_group = look
            .radio_group_horizontal("controls-doc-radio-group-horizontal")
            .template(secondary_horizontal)
            .items(density_items())
            .selected(Density::Comfortable.id())
            .spawn(cx);
        let indented_group = sdk_radio_group::new("controls-doc-radio-group-indented")
            .items(density_items())
            .item_element_template(sdk_radio_group::radio_group_button_item_element_template(radio_template))
            .with_item_layout(|mut items, _, _, _| {
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .child(indented_option(0, items.take_index(0)))
                    .child(indented_option(1, items.take_index(1)))
                    .child(indented_option(2, items.take_index(2)))
            })
            .selected(Density::Comfortable.id())
            .spawn(cx);
        let delivery_group = sdk_radio_group::new("controls-doc-radio-group-delivery")
            .items(delivery_window_items())
            .item_element_template(delivery_window_item_element_template(look.clone()))
            .with_item_layout(|items, _, _, _| {
                div().flex().items_center().gap(px(DELIVERY_GROUP_GAP)).children(items.into_elements())
            })
            .selected(DeliveryWindow::Today.id())
            .selection_mode(SelectionMode::SingleAllowNone)
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-radio-group-event-log",
                "Select radio group items; RadioGroupEvent variants from the vertical group appear below.",
            )
        });

        let subscriptions = vec![
            wire_group(cx, &vertical_group, "vertical", GroupTarget::Vertical, event_stream.clone()),
            wire_group(cx, &horizontal_group, "horizontal", GroupTarget::Horizontal, event_stream.clone()),
            wire_group(cx, &indented_group, "indented", GroupTarget::Indented, event_stream.clone()),
            wire_group(cx, &delivery_group, "delivery", GroupTarget::Delivery, event_stream.clone()),
        ];

        Self {
            look,
            entry,
            vertical_group,
            horizontal_group,
            indented_group,
            delivery_group,
            vertical_choice: Density::Comfortable.id().to_string(),
            horizontal_choice: Density::Comfortable.id().to_string(),
            indented_choice: Density::Comfortable.id().to_string(),
            delivery_choice: DeliveryWindow::Today.id().to_string(),
            event_stream,
            _subscriptions: subscriptions,
        }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for group in [&self.vertical_group, &self.horizontal_group, &self.indented_group] {
            group.update(cx, |_, cx| cx.notify());
        }
        self.delivery_group.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

#[derive(Clone, Copy)]
enum GroupTarget {
    Vertical,
    Horizontal,
    Indented,
    Delivery,
}

fn wire_group<T: RadioGroupItemLike + 'static>(
    cx: &mut Context<RadioGroupControlExposition>,
    group: &RadioGroup<T>,
    label: &'static str,
    target: GroupTarget,
    event_stream: Entity<ControlEventStream>,
) -> Subscription {
    cx.subscribe(group, move |this, _, event: &RadioGroupEvent, cx| {
        if let RadioGroupEvent::Change { value } = event {
            let choice = value.as_ref().map_or_else(|| "None".to_string(), ToString::to_string);
            match target {
                GroupTarget::Vertical => this.vertical_choice = choice,
                GroupTarget::Horizontal => this.horizontal_choice = choice,
                GroupTarget::Indented => this.indented_choice = choice,
                GroupTarget::Delivery => this.delivery_choice = choice,
            }
            cx.notify();
        }
        if matches!(target, GroupTarget::Vertical)
            && let Some(line) = format_radio_group_event(label, event)
        {
            event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
        }
    })
}

impl Render for RadioGroupControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(PANE_EXAMPLE_GAP))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(px(PANE_EXAMPLE_GAP))
                        .child(render_example(
                            "Vertical default",
                            self.vertical_group.clone(),
                            &self.vertical_choice,
                            chrome.body_text,
                        ))
                        .child(render_example(
                            "Horizontal default",
                            self.horizontal_group.clone(),
                            &self.horizontal_choice,
                            chrome.body_text,
                        )),
                )
                .child(render_example_with_padding(
                    "Horizontal custom indent",
                    self.indented_group.clone(),
                    &self.indented_choice,
                    chrome.body_text,
                    INDENTED_EXAMPLE_PADDING_Y,
                ))
                .child(render_example(
                    "Delivery window",
                    self.delivery_group.clone(),
                    &self.delivery_choice,
                    chrome.body_text,
                ))
                .child(self.event_stream.clone());

            render_control_exposition_card(
                &self.look,
                self.entry,
                preview.into_any_element(),
                None,
                ControlExpositionLayout::BORDERLESS,
            )
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Density {
    Compact,
    Comfortable,
    Expanded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DeliveryWindow {
    Today,
    Wednesday,
    Thursday,
    Friday,
}

impl Density {
    fn id(&self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Comfortable => "comfortable",
            Self::Expanded => "expanded",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Comfortable => "Comfortable",
            Self::Expanded => "Expanded",
        }
    }
}

impl DeliveryWindow {
    fn id(&self) -> &'static str {
        match self {
            Self::Today => "today",
            Self::Wednesday => "wednesday",
            Self::Thursday => "thursday",
            Self::Friday => "friday",
        }
    }

    fn day(&self) -> &'static str {
        match self {
            Self::Today => "Today",
            Self::Wednesday => "Wed",
            Self::Thursday => "Thu",
            Self::Friday => "Fri",
        }
    }

    fn date(&self) -> &'static str {
        match self {
            Self::Today => "May 15",
            Self::Wednesday => "May 16",
            Self::Thursday => "May 17",
            Self::Friday => "May 18",
        }
    }
}

#[derive(Clone, Debug)]
struct DeliveryWindowItem {
    id: gpui::SharedString,
    day: gpui::SharedString,
    date: gpui::SharedString,
}

impl DeliveryWindowItem {
    fn new(
        id: impl Into<gpui::SharedString>,
        day: impl Into<gpui::SharedString>,
        date: impl Into<gpui::SharedString>,
    ) -> Self {
        Self { id: id.into(), day: day.into(), date: date.into() }
    }
}

impl RadioGroupItemLike for DeliveryWindowItem {
    fn id(&self) -> &gpui::SharedString {
        &self.id
    }

    fn label(&self) -> &gpui::SharedString {
        &self.day
    }
}

struct DeliveryWindowTheme {
    background: gpui::Hsla,
    foreground: gpui::Hsla,
    muted_foreground: gpui::Hsla,
    selected_background: gpui::Hsla,
    selected_foreground: gpui::Hsla,
    focus_ring: gpui::Hsla,
}

impl DeliveryWindowTheme {
    fn resolve(look: &ShadcnLook) -> Self {
        let chrome = look.chrome();
        Self {
            background: theme_color(look, "card", chrome.panel_background),
            foreground: theme_color(look, "card-foreground", chrome.body_text),
            muted_foreground: theme_color(look, "muted-foreground", chrome.muted_text),
            selected_background: theme_color(look, "primary", theme_color(look, "accent", chrome.panel_background)),
            selected_foreground: theme_color(
                look,
                "primary-foreground",
                theme_color(look, "accent-foreground", chrome.title_text),
            ),
            focus_ring: theme_color(look, "ring", chrome.border),
        }
    }
}

fn theme_color(look: &ShadcnLook, token: &str, fallback: gpui::Hsla) -> gpui::Hsla {
    look.token_color(token).unwrap_or(fallback)
}

fn density_items() -> [RadioGroupItem; DENSITY_OPTION_COUNT] {
    [
        RadioGroupItem::new(Density::Compact.id()).label(Density::Compact.label()),
        RadioGroupItem::new(Density::Comfortable.id()).label(Density::Comfortable.label()),
        RadioGroupItem::new(Density::Expanded.id()).label(Density::Expanded.label()),
    ]
}

fn delivery_window_items() -> [DeliveryWindowItem; DELIVERY_OPTION_COUNT] {
    [
        DeliveryWindowItem::new(DeliveryWindow::Today.id(), DeliveryWindow::Today.day(), DeliveryWindow::Today.date()),
        DeliveryWindowItem::new(
            DeliveryWindow::Wednesday.id(),
            DeliveryWindow::Wednesday.day(),
            DeliveryWindow::Wednesday.date(),
        ),
        DeliveryWindowItem::new(
            DeliveryWindow::Thursday.id(),
            DeliveryWindow::Thursday.day(),
            DeliveryWindow::Thursday.date(),
        ),
        DeliveryWindowItem::new(
            DeliveryWindow::Friday.id(),
            DeliveryWindow::Friday.day(),
            DeliveryWindow::Friday.date(),
        ),
    ]
}

fn delivery_window_card(
    item: &DeliveryWindowItem,
    selected: bool,
    state: luma::controls::state::CompositeItemState,
    theme: &DeliveryWindowTheme,
) -> Div {
    let day = &item.day;
    let date = &item.date;
    let foreground = if selected {
        theme.selected_foreground
    } else {
        theme.foreground
    };
    let date_color = if selected {
        theme.selected_foreground
    } else {
        theme.muted_foreground
    };

    let mut card = div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .min_w(px(DELIVERY_CARD_MIN_WIDTH))
        .h(px(DELIVERY_CARD_HEIGHT))
        .px(px(DELIVERY_CARD_PADDING_X))
        .rounded(px(DELIVERY_CARD_RADIUS))
        .bg(if selected {
            theme.selected_background
        } else {
            theme.background
        })
        .text_color(foreground)
        .child(
            div()
                .text_size(px(DELIVERY_DAY_TEXT_SIZE))
                .line_height(px(DELIVERY_DAY_LINE_HEIGHT))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(day.clone()),
        )
        .child(
            div()
                .text_size(px(DELIVERY_DATE_TEXT_SIZE))
                .line_height(px(DELIVERY_DATE_LINE_HEIGHT))
                .text_color(date_color)
                .child(date.clone()),
        );

    if state.active {
        card = card.border(px(DELIVERY_FOCUS_BORDER_WIDTH)).border_color(theme.focus_ring);
    } else if selected {
        card = card.border(px(DELIVERY_SELECTED_BORDER_WIDTH));
    }

    if !state.disabled {
        card.cursor_pointer()
    } else {
        card.opacity(DELIVERY_DISABLED_OPACITY)
    }
}

fn indented_option(index: usize, option: AnyElement) -> AnyElement {
    div()
        .pl(px(index as f32 * INDENTED_BUTTON_INDENT_STEP))
        .when(index > FIRST_INDENTED_BUTTON_INDEX, |row| row.mt(px(-INDENTED_BUTTON_VERTICAL_OVERLAP)))
        .child(option)
        .into_any_element()
}

fn delivery_window_item_element_template(look: Arc<ShadcnLook>) -> ControlGroupItemElementTemplate<DeliveryWindowItem> {
    Arc::new(move |item, _, _, _| {
        let theme = DeliveryWindowTheme::resolve(&look);
        delivery_window_card(item.item, item.selected, item.state, &theme).id(format!(
            "{}-{}",
            item.group_id,
            item.item.id()
        ))
    })
}

fn render_example<T>(label: &'static str, group: RadioGroup<T>, choice: &str, text_color: gpui::Hsla) -> AnyElement
where
    T: RadioGroupItemLike + 'static,
{
    render_example_with_padding(label, group, choice, text_color, EXAMPLE_PADDING_Y)
}

fn render_example_with_padding<T>(
    label: &'static str,
    group: RadioGroup<T>,
    choice: &str,
    text_color: gpui::Hsla,
    padding_y: f32,
) -> AnyElement
where
    T: RadioGroupItemLike + 'static,
{
    div()
        .flex()
        .flex_col()
        .flex_none()
        .items_start()
        .gap(px(EXAMPLE_CONTENT_GAP))
        .rounded(px(EXAMPLE_BORDER_RADIUS))
        .border(px(EXAMPLE_BORDER_WIDTH))
        .border_color(text_color.opacity(EXAMPLE_BORDER_OPACITY))
        .px(px(EXAMPLE_PADDING_X))
        .py(px(padding_y))
        .child(
            div()
                .text_size(px(EXAMPLE_LABEL_TEXT_SIZE))
                .line_height(px(EXAMPLE_LABEL_LINE_HEIGHT))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(text_color)
                .child(label),
        )
        .child(group)
        .child(
            div()
                .text_size(px(CHOICE_TEXT_SIZE))
                .line_height(px(CHOICE_LINE_HEIGHT))
                .text_color(text_color)
                .child(format!("Choice: {choice}")),
        )
        .into_any_element()
}

fn format_radio_group_event(source: &str, event: &RadioGroupEvent) -> Option<String> {
    match event {
        RadioGroupEvent::Change { value } => {
            let value = value.as_ref().map_or_else(|| "None".to_string(), |value| format!("\"{value}\""));
            Some(format!("RadioGroupEvent::Change - {source} (value: {value})"))
        }
        RadioGroupEvent::FocusChanged { focused } => {
            Some(format!("RadioGroupEvent::FocusChanged - {source} (focused: {focused})"))
        }
        _ => None,
    }
}
