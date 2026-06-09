use std::sync::Arc;

use gpui::{AnyElement, Context, Div, Entity, Subscription, div, prelude::*, px};
use gpui_luma::controls::command::button::ButtonTemplate;
use gpui_luma::controls::radio_group::{
    self as sdk_radio_group, RadioGroup, RadioGroupEvent, RadioGroupItem, RadioGroupItemLike, RadioGroupTemplate,
    RadioGroupTemplateHandlers, SelectionMode, render_radio_button_rows,
};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_radio_group_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector_description, notify_entity};

const RADIO_GROUP_DESCRIPTION: &str = concat!(
    "Radio groups use single-selection semantics with custom item templates. ",
    "Examples below show vertical, horizontal, and card-style layouts."
);

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

#[derive(Clone)]
pub(in crate::gallery) struct RadioGroupPane {
    vertical_group: RadioGroup<RadioGroupItem>,
    horizontal_group: RadioGroup<RadioGroupItem>,
    indented_group: RadioGroup<RadioGroupItem>,
    delivery_group: RadioGroup<DeliveryWindowItem>,
    vertical_choice: String,
    horizontal_choice: String,
    indented_choice: String,
    delivery_choice: String,
    inspector: Entity<ColorInspectorShell>,
}

impl RadioGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree =
            spawn_color_inspector_tree("radio-group-inspector-tree", look.clone(), build_radio_group_inspect_tree, cx);
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "radio-group-inspector",
                "radio-group-inspector-split",
                "radio-group-inspector-detail",
                build_radio_group_inspect_tree,
                cx,
            )
        });

        let secondary_vertical = look.radio_group_template(ShadcnButtonStyle::Secondary);
        let secondary_horizontal = look.radio_group_horizontal_template(ShadcnButtonStyle::Secondary);
        let radio_template = look.radio_button_template(ShadcnButtonStyle::Secondary);

        let vertical_group = look
            .radio_group("radio-group-density-vertical")
            .template(secondary_vertical)
            .items(density_items())
            .selected(Density::Comfortable.id())
            .spawn(cx);

        let horizontal_group = look
            .radio_group_horizontal("radio-group-density-horizontal")
            .template(secondary_horizontal)
            .items(density_items())
            .selected(Density::Comfortable.id())
            .spawn(cx);

        let indented_group = sdk_radio_group::new("radio-group-density-indented")
            .items(density_items())
            .template(radio_button_indented_template(radio_template))
            .selected(Density::Comfortable.id())
            .spawn(cx);

        let delivery_group = sdk_radio_group::horizontal("radio-group-delivery-window")
            .items(delivery_window_items())
            .template(delivery_window_template(look.clone()))
            .selected(DeliveryWindow::Today.id())
            .selection_mode(SelectionMode::SingleAllowNone)
            .spawn(cx);

        Self {
            vertical_group,
            horizontal_group,
            indented_group,
            delivery_group,
            vertical_choice: Density::Comfortable.id().to_string(),
            horizontal_choice: Density::Comfortable.id().to_string(),
            indented_choice: Density::Comfortable.id().to_string(),
            delivery_choice: DeliveryWindow::Today.id().to_string(),
            inspector,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.vertical_group, |app, _, event: &RadioGroupEvent, cx| {
            app.panes.radio_group.handle_vertical_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.horizontal_group, |app, _, event: &RadioGroupEvent, cx| {
            app.panes.radio_group.handle_horizontal_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.indented_group, |app, _, event: &RadioGroupEvent, cx| {
            app.panes.radio_group.handle_indented_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.delivery_group, |app, _, event: &RadioGroupEvent, cx| {
            app.panes.radio_group.handle_delivery_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();

        gallery_pane_with_inspector_description(
            "Radio Group",
            Some(RADIO_GROUP_DESCRIPTION),
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(PANE_EXAMPLE_GAP))
                .child(
                    div()
                        .flex()
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
                .into_any_element(),
            self.inspector.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.vertical_group, cx);
        notify_entity(&self.horizontal_group, cx);
        notify_entity(&self.indented_group, cx);
        notify_entity(&self.delivery_group, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn handle_vertical_event(&mut self, event: &RadioGroupEvent, cx: &mut Context<GalleryApp>) {
        let RadioGroupEvent::Change { selected_ids, .. } = event;
        self.vertical_choice = selected_ids.first().map_or_else(|| "None".to_string(), ToString::to_string);
        cx.notify();
    }

    fn handle_horizontal_event(&mut self, event: &RadioGroupEvent, cx: &mut Context<GalleryApp>) {
        let RadioGroupEvent::Change { selected_ids, .. } = event;
        self.horizontal_choice = selected_ids.first().map_or_else(|| "None".to_string(), ToString::to_string);
        cx.notify();
    }

    fn handle_indented_event(&mut self, event: &RadioGroupEvent, cx: &mut Context<GalleryApp>) {
        let RadioGroupEvent::Change { selected_ids, .. } = event;
        self.indented_choice = selected_ids.first().map_or_else(|| "None".to_string(), ToString::to_string);
        cx.notify();
    }

    fn handle_delivery_event(&mut self, event: &RadioGroupEvent, cx: &mut Context<GalleryApp>) {
        let RadioGroupEvent::Change { selected_ids, .. } = event;
        self.delivery_choice = selected_ids.first().map_or_else(|| "None".to_string(), ToString::to_string);
        cx.notify();
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

struct DeliveryWindowTemplateSpec {
    selected_background: gpui::Hsla,
    selected_foreground: gpui::Hsla,
    background: gpui::Hsla,
    foreground: gpui::Hsla,
    muted_foreground: gpui::Hsla,
    focus_ring: gpui::Hsla,
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
    state: gpui_luma::controls::state::CompositeItemState,
    spec: &DeliveryWindowTemplateSpec,
) -> Div {
    let day = &item.day;
    let date = &item.date;
    let foreground = if selected {
        spec.selected_foreground
    } else {
        spec.foreground
    };
    let date_color = if selected {
        spec.selected_foreground
    } else {
        spec.muted_foreground
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
            spec.selected_background
        } else {
            spec.background
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
        card = card.border(px(DELIVERY_FOCUS_BORDER_WIDTH)).border_color(spec.focus_ring);
    } else if selected {
        card = card.border(px(DELIVERY_SELECTED_BORDER_WIDTH));
    }

    if !state.disabled {
        card.cursor_pointer()
    } else {
        card.opacity(DELIVERY_DISABLED_OPACITY)
    }
}

fn radio_button_indented_template(
    button_template: Arc<dyn ButtonTemplate<bool>>,
) -> RadioGroupTemplate<RadioGroupItem> {
    Arc::new(move |model, handlers, window, cx| {
        div().id(model.id.clone()).flex().flex_col().items_start().children(
            render_radio_button_rows(model, handlers, &button_template, window, cx).into_iter().enumerate().map(
                |(index, button)| {
                    div()
                        .pl(px(index as f32 * INDENTED_BUTTON_INDENT_STEP))
                        .when(index > FIRST_INDENTED_BUTTON_INDEX, |row| row.mt(px(-INDENTED_BUTTON_VERTICAL_OVERLAP)))
                        .child(button)
                        .into_any_element()
                },
            ),
        )
    })
}

fn delivery_window_template(look: Arc<ShadcnLook>) -> RadioGroupTemplate<DeliveryWindowItem> {
    let chrome = look.chrome();
    let spec = Arc::new(DeliveryWindowTemplateSpec {
        selected_background: look.token_color("accent").unwrap_or(chrome.panel_background),
        selected_foreground: look.token_color("accent-foreground").unwrap_or(chrome.title_text),
        background: chrome.panel_background,
        foreground: chrome.body_text,
        muted_foreground: chrome.muted_text,
        focus_ring: look.token_color("ring").unwrap_or(chrome.border),
    });

    Arc::new(move |model, handlers, _window, _cx| {
        let RadioGroupTemplateHandlers {
            item_hovers,
            item_mouse_downs,
            item_mouse_ups,
            item_mouse_up_outs,
            item_clicks,
        } = handlers;

        let mut item_hovers = item_hovers.into_iter();
        let mut item_mouse_downs = item_mouse_downs.into_iter();
        let mut item_mouse_ups = item_mouse_ups.into_iter();
        let mut item_mouse_up_outs = item_mouse_up_outs.into_iter();
        let mut item_clicks = item_clicks.into_iter();

        let mut root = div().id(model.id.clone()).flex().items_center().gap(px(DELIVERY_GROUP_GAP));

        for item in &model.items {
            let Some(item_hover) = item_hovers.next() else {
                break;
            };
            let Some(item_mouse_down) = item_mouse_downs.next() else {
                break;
            };
            let Some(item_mouse_up) = item_mouse_ups.next() else {
                break;
            };
            let Some(item_mouse_up_out) = item_mouse_up_outs.next() else {
                break;
            };
            let Some(item_click) = item_clicks.next() else {
                break;
            };

            let card = delivery_window_card(item.item, item.selected, item.state, &spec)
                .id(format!("{}-{}", model.id, item.item.id()))
                .on_hover(item_hover)
                .on_mouse_down(gpui::MouseButton::Left, item_mouse_down)
                .on_mouse_up(gpui::MouseButton::Left, item_mouse_up)
                .on_mouse_up_out(gpui::MouseButton::Left, item_mouse_up_out)
                .on_click(item_click);

            root = root.child(card);
        }

        root
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
                .child(format!("Choice: {}", choice)),
        )
        .into_any_element()
}
