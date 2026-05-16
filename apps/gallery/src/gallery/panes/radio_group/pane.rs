use gpui::{AnyElement, Context, Entity, Subscription, div, prelude::*, px};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::radio_group::{self, RadioGroup, RadioGroupEvent, SelectionMode};
use super::super::shared::{gallery_pane, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct RadioGroupPane {
    group: Entity<RadioGroup<Density>>,
    choice: String,
}

impl RadioGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let group = cx.new(|cx| {
            radio_group::new(
                "radio-group-density",
                |density: &Density| density.id().into(),
                |density: &Density| density.label().into(),
                theme.radio_button_template(),
            )
            .items(Density::all())
            .selected(Density::Comfortable)
            .mode(SelectionMode::SingleRequired)
            .build(cx)
        });

        Self { group, choice: Density::Comfortable.id().to_string() }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.group, |app, _, event: &RadioGroupEvent, cx| {
            app.panes.radio_group.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane(
            "Radio Group",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(self.group.clone())
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(chrome.body_text)
                        .child(format!("Choice: {}", self.choice)),
                )
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.group, cx);
    }

    fn handle_event(&mut self, event: &RadioGroupEvent, cx: &mut Context<GalleryApp>) {
        let RadioGroupEvent::Change { selected_id } = event;
        self.choice = selected_id.as_ref().map(|selected_id| selected_id.as_ref()).unwrap_or("None").to_string();
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
