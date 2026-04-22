use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*};
use gpui_luma::controls::radio_group::{RadioGroup, RadioGroupEvent, RadioGroupItem};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct RadioGroupPane {
    radio_group: Entity<RadioGroup>,
    disabled_radio_group: Entity<RadioGroup>,
    choice: String,
}

impl RadioGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            radio_group: RadioGroup::new("density-radio-group")
                .items(density_items())
                .selected("comfortable")
                .template(theme.radio_group_template())
                .spawn(cx),
            disabled_radio_group: RadioGroup::new("disabled-density-radio-group")
                .items(density_items())
                .selected("comfortable")
                .enabled(false)
                .template(theme.radio_group_template())
                .spawn(cx),
            choice: "Comfortable".to_string(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.radio_group, |app, _, event: &RadioGroupEvent, cx| {
            app.panes.radio_group.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage(
            "Radio Group",
            "Radio Group",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_3()
                        .child(self.radio_group.clone())
                        .child(self.disabled_radio_group.clone()),
                )
                .child(div().text_color(chrome.body_text).child(format!("Choice: {}", self.choice)))
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.radio_group, cx);
        notify_entity(&self.disabled_radio_group, cx);
    }

    fn handle_event(&mut self, event: &RadioGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            RadioGroupEvent::Change { label, .. } => {
                self.choice = label.to_string();
                cx.notify();
            }
        }
    }
}

fn density_items() -> [RadioGroupItem; 3] {
    [
        RadioGroupItem::new("compact").label("Compact"),
        RadioGroupItem::new("comfortable").label("Comfortable"),
        RadioGroupItem::new("expanded").label("Expanded"),
    ]
}
