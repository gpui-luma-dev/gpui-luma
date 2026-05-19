use std::sync::Arc;

use gpui::{AnyElement, App, Context, Div, Entity, Stateful, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{ButtonRenderModel, ButtonTemplate};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::radio_group::{self, RadioGroup, RadioGroupEvent, RadioItemState, SelectionMode};
use super::super::shared::{gallery_pane, notify_entity};

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
    vertical_group: Entity<RadioGroup<Density>>,
    horizontal_group: Entity<RadioGroup<Density>>,
    indented_group: Entity<RadioGroup<Density>>,
    delivery_group: Entity<RadioGroup<DeliveryWindow>>,
    vertical_choice: String,
    horizontal_choice: String,
    indented_choice: String,
    delivery_choice: String,
}

impl RadioGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let vertical_group = cx.new(|cx| {
            radio_group::new("radio-group-density-vertical")
                .items(Density::all())
                .item_id(|density: &Density| density.id().into())
                .item_label(|density: &Density| density.label().into())
                .item_template(radio_group::selection_state_template(theme.radio_button_template()))
                .template(radio_group::vertical_group_template())
                .selected(Density::Comfortable)
                .mode(SelectionMode::SingleRequired)
                .build(cx)
        });

        let horizontal_group = cx.new(|cx| {
            radio_group::new("radio-group-density-horizontal")
                .items(Density::all())
                .item_id(|density: &Density| density.id().into())
                .item_label(|density: &Density| density.label().into())
                .item_template(radio_group::selection_state_template(theme.radio_button_template()))
                .template(radio_group::horizontal_group_template())
                .selected(Density::Comfortable)
                .mode(SelectionMode::SingleRequired)
                .build(cx)
        });

        let indented_group = cx.new(|cx| {
            radio_group::new("radio-group-density-indented")
                .items(Density::all())
                .item_id(|density: &Density| density.id().into())
                .item_label(|density: &Density| density.label().into())
                .item_template(radio_group::selection_state_template(theme.radio_button_template()))
                .selected(Density::Comfortable)
                .mode(SelectionMode::SingleRequired)
                .with_template(|group, _window, _cx| {
                    div()
                        .flex()
                        .flex_col()
                        .items_start()
                        .children(group.buttons().iter().cloned().enumerate().map(|(index, button)| {
                            div()
                                .pl(px(index as f32 * INDENTED_BUTTON_INDENT_STEP))
                                .when(index > FIRST_INDENTED_BUTTON_INDEX, |row| {
                                    row.mt(px(-INDENTED_BUTTON_VERTICAL_OVERLAP))
                                })
                                .child(button)
                        }))
                        .into_any_element()
                })
                .build(cx)
        });

        let delivery_group = cx.new(|cx| {
            radio_group::new("radio-group-delivery-window")
                .items(DeliveryWindow::all())
                .item_id(|delivery: &DeliveryWindow| delivery.id().into())
                .item_label(|delivery: &DeliveryWindow| delivery.day().into())
                .item_template(delivery_window_template(theme))
                .mode(SelectionMode::SingleAllowNone)
                .with_template(|group, _window, _cx| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(DELIVERY_GROUP_GAP))
                        .children(group.buttons().iter().cloned())
                        .into_any_element()
                })
                .build(cx)
        });

        Self {
            vertical_group,
            horizontal_group,
            indented_group,
            delivery_group,
            vertical_choice: Density::Comfortable.id().to_string(),
            horizontal_choice: Density::Comfortable.id().to_string(),
            indented_choice: Density::Comfortable.id().to_string(),
            delivery_choice: DeliveryWindow::Today.id().to_string(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.vertical_group, |app, _, event: &RadioGroupEvent<Density>, cx| {
            app.panes.radio_group.handle_vertical_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.horizontal_group, |app, _, event: &RadioGroupEvent<Density>, cx| {
            app.panes.radio_group.handle_horizontal_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.indented_group, |app, _, event: &RadioGroupEvent<Density>, cx| {
            app.panes.radio_group.handle_indented_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(
            &self.delivery_group,
            |app, _, event: &RadioGroupEvent<DeliveryWindow>, cx| {
                app.panes.radio_group.handle_delivery_event(event, cx);
            },
        ));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane(
            "Radio Group",
            div()
                .flex()
                .flex_col()
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
                ))
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
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.vertical_group, cx);
        notify_entity(&self.horizontal_group, cx);
        notify_entity(&self.indented_group, cx);
        notify_entity(&self.delivery_group, cx);
    }

    fn handle_vertical_event(&mut self, event: &RadioGroupEvent<Density>, cx: &mut Context<GalleryApp>) {
        let RadioGroupEvent::Change { selected_value, .. } = event;
        self.vertical_choice =
            selected_value.as_ref().map_or_else(|| "None".to_string(), |value| value.id().to_string());
        cx.notify();
    }

    fn handle_horizontal_event(&mut self, event: &RadioGroupEvent<Density>, cx: &mut Context<GalleryApp>) {
        let RadioGroupEvent::Change { selected_value, .. } = event;
        self.horizontal_choice =
            selected_value.as_ref().map_or_else(|| "None".to_string(), |value| value.id().to_string());
        cx.notify();
    }

    fn handle_indented_event(&mut self, event: &RadioGroupEvent<Density>, cx: &mut Context<GalleryApp>) {
        let RadioGroupEvent::Change { selected_value, .. } = event;
        self.indented_choice =
            selected_value.as_ref().map_or_else(|| "None".to_string(), |value| value.id().to_string());
        cx.notify();
    }

    fn handle_delivery_event(&mut self, event: &RadioGroupEvent<DeliveryWindow>, cx: &mut Context<GalleryApp>) {
        let RadioGroupEvent::Change { selected_value, .. } = event;
        self.delivery_choice =
            selected_value.as_ref().map_or_else(|| "None".to_string(), |value| value.id().to_string());
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

    fn all() -> [Self; DENSITY_OPTION_COUNT] {
        [Self::Compact, Self::Comfortable, Self::Expanded]
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

    fn all() -> [Self; DELIVERY_OPTION_COUNT] {
        [Self::Today, Self::Wednesday, Self::Thursday, Self::Friday]
    }
}

struct DeliveryWindowTemplate {
    selected_background: gpui::Hsla,
    selected_foreground: gpui::Hsla,
    background: gpui::Hsla,
    foreground: gpui::Hsla,
    muted_foreground: gpui::Hsla,
    focus_ring: gpui::Hsla,
}

impl ButtonTemplate<RadioItemState<DeliveryWindow>> for DeliveryWindowTemplate {
    fn render(
        &self,
        model: &ButtonRenderModel<RadioItemState<DeliveryWindow>>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let delivery = &model.data.value;
        let selected = model.data.is_selected;
        let day = delivery.day();
        let date = delivery.date();
        let foreground = if selected {
            self.selected_foreground
        } else {
            self.foreground
        };
        let date_color = if selected {
            self.selected_foreground
        } else {
            self.muted_foreground
        };

        let mut card = div()
            .id(model.id.clone())
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .min_w(px(DELIVERY_CARD_MIN_WIDTH))
            .h(px(DELIVERY_CARD_HEIGHT))
            .px(px(DELIVERY_CARD_PADDING_X))
            .rounded(px(DELIVERY_CARD_RADIUS))
            .bg(if selected {
                self.selected_background
            } else {
                self.background
            })
            .text_color(foreground)
            .child(
                div()
                    .text_size(px(DELIVERY_DAY_TEXT_SIZE))
                    .line_height(px(DELIVERY_DAY_LINE_HEIGHT))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(day),
            )
            .child(
                div()
                    .text_size(px(DELIVERY_DATE_TEXT_SIZE))
                    .line_height(px(DELIVERY_DATE_LINE_HEIGHT))
                    .text_color(date_color)
                    .child(date),
            );

        if model.state.focused {
            card = card.border(px(DELIVERY_FOCUS_BORDER_WIDTH)).border_color(self.focus_ring);
        } else if selected {
            card = card.border(px(DELIVERY_SELECTED_BORDER_WIDTH));
        }

        if !model.state.disabled {
            card.cursor_pointer()
        } else {
            card.opacity(DELIVERY_DISABLED_OPACITY)
        }
    }
}

fn delivery_window_template(theme: &GalleryThemePack) -> Arc<dyn ButtonTemplate<RadioItemState<DeliveryWindow>>> {
    let tokens = theme.tokens();
    Arc::new(DeliveryWindowTemplate {
        selected_background: tokens.palette.data.accent_4,
        selected_foreground: tokens.palette.action.prominent.foreground,
        background: tokens.palette.surface.subtle.background,
        foreground: tokens.palette.app.foreground,
        muted_foreground: tokens.palette.app.muted_foreground,
        focus_ring: tokens.palette.focus.ring,
    })
}

fn render_example<T>(
    label: &'static str,
    group: Entity<RadioGroup<T>>,
    choice: &str,
    text_color: gpui::Hsla,
) -> AnyElement
where
    T: Clone + Eq + 'static,
{
    render_example_with_padding(label, group, choice, text_color, EXAMPLE_PADDING_Y)
}

fn render_example_with_padding<T>(
    label: &'static str,
    group: Entity<RadioGroup<T>>,
    choice: &str,
    text_color: gpui::Hsla,
    padding_y: f32,
) -> AnyElement
where
    T: Clone + Eq + 'static,
{
    div()
        .flex()
        .flex_col()
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
