use gpui::{AnyElement, Context, SharedString, Subscription, div, hsla, prelude::*, px};
use gpui_luma::controls::choice_group::{self, ChoiceGroup, ChoiceGroupEvent, ChoiceGroupItem};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct RadioGroupPane {
    group: ChoiceGroup,
    choice: String,
    selected_id: SharedString,
}

impl RadioGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let transparent = hsla(0.0, 0.0, 0.0, 0.0);
        let initial = Density::Comfortable;

        let group = choice_group::single_select("radio-group-density")
            .managed_selected(initial.id())
            .items(density_items())
            .bool_button_template(theme.radio_button_template())
            .with_modifier(move |el, _| el.bg(transparent).border_color(transparent))
            .spawn(cx);

        Self { group, choice: initial.label().to_string(), selected_id: SharedString::from(initial.id()) }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.group, |app, _, event: &ChoiceGroupEvent, cx| {
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

    fn handle_event(&mut self, event: &ChoiceGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ChoiceGroupEvent::Change { item_id, label, selected, selected_ids, .. } => {
                if *selected {
                    self.selected_id = item_id.clone();
                    self.choice = label.to_string();
                }

                let fallback = self.selected_id.clone();
                let commit = if selected_ids.is_empty() {
                    vec![fallback]
                } else {
                    selected_ids.clone()
                };

                self.group.update(cx, move |group, cx| group.set_managed_selected_ids(commit.clone(), cx));

                cx.notify();
            }
        }
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

fn density_items() -> Vec<ChoiceGroupItem> {
    Density::all()
        .into_iter()
        .map(|density| ChoiceGroupItem::new(density.id(), density.label()))
        .collect()
}
