use gpui::{AnyElement, Context, Entity, Subscription, div, prelude::*, px};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::radio_group::{self, RadioGroup, RadioGroupEvent, SelectionMode};
use super::super::shared::{gallery_pane, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct RadioGroupPane {
    vertical_group: Entity<RadioGroup<Density>>,
    horizontal_group: Entity<RadioGroup<Density>>,
    vertical_choice: String,
    horizontal_choice: String,
}

impl RadioGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let vertical_group = cx.new(|cx| {
            radio_group::vertical_group(
                "radio-group-density-vertical",
                |density: &Density| density.id().into(),
                |density: &Density| density.label().into(),
                theme.radio_button_template(),
            )
            .items(Density::all())
            .selected(Density::Comfortable)
            .mode(SelectionMode::SingleRequired)
            .build(cx)
        });

        let horizontal_group = cx.new(|cx| {
            radio_group::horizontal_group(
                "radio-group-density-horizontal",
                |density: &Density| density.id().into(),
                |density: &Density| density.label().into(),
                theme.radio_button_template(),
            )
            .items(Density::all())
            .selected(Density::Comfortable)
            .mode(SelectionMode::SingleRequired)
            .build(cx)
        });

        Self {
            vertical_group,
            horizontal_group,
            vertical_choice: Density::Comfortable.id().to_string(),
            horizontal_choice: Density::Comfortable.id().to_string(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.vertical_group, |app, _, event: &RadioGroupEvent, cx| {
            app.panes.radio_group.handle_vertical_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.horizontal_group, |app, _, event: &RadioGroupEvent, cx| {
            app.panes.radio_group.handle_horizontal_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane(
            "Radio Group",
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap_6()
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
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.vertical_group, cx);
        notify_entity(&self.horizontal_group, cx);
    }

    fn handle_vertical_event(&mut self, event: &RadioGroupEvent, cx: &mut Context<GalleryApp>) {
        let RadioGroupEvent::Change { selected_id } = event;
        self.vertical_choice =
            selected_id.as_ref().map(|selected_id| selected_id.as_ref()).unwrap_or("None").to_string();
        cx.notify();
    }

    fn handle_horizontal_event(&mut self, event: &RadioGroupEvent, cx: &mut Context<GalleryApp>) {
        let RadioGroupEvent::Change { selected_id } = event;
        self.horizontal_choice =
            selected_id.as_ref().map(|selected_id| selected_id.as_ref()).unwrap_or("None").to_string();
        cx.notify();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Density {
    Compact,
    Comfortable,
    Expanded,
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

    fn all() -> [Self; 3] {
        [Self::Compact, Self::Comfortable, Self::Expanded]
    }
}

fn render_example(
    label: &'static str,
    group: Entity<RadioGroup<Density>>,
    choice: &str,
    text_color: gpui::Hsla,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .rounded_md()
        .border_1()
        .border_color(text_color.opacity(0.24))
        .p_3()
        .child(
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(text_color)
                .child(label),
        )
        .child(group)
        .child(
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .text_color(text_color)
                .child(format!("Choice: {}", choice)),
        )
        .into_any_element()
}
