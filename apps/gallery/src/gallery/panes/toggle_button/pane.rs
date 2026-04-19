use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, rgb};
use gpui_luma::controls::toggle_button::{ToggleButton, ToggleButtonEvent};

use crate::gallery::control::GalleryApp;

use super::super::shared::gallery_pane;

pub(in crate::gallery) struct ToggleButtonPane {
    toggle_button: Entity<ToggleButton>,
    disabled_toggle_button: Entity<ToggleButton>,
    selected: bool,
}

impl ToggleButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            toggle_button: ToggleButton::new("toggle-button-example").label("Toggle").selected(true).spawn(cx),
            disabled_toggle_button: ToggleButton::new("disabled-toggle-button")
                .label("Disabled toggle")
                .selected(true)
                .enabled(false)
                .spawn(cx),
            selected: true,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.toggle_button, |app, _, event: &ToggleButtonEvent, cx| {
            app.panes.toggle_button.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self) -> AnyElement {
        gallery_pane(
            "Toggle Button",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(self.toggle_button.clone())
                        .child(self.disabled_toggle_button.clone()),
                )
                .child(div().text_color(rgb(0x334155)).child(format!("Selected: {}", self.selected)))
                .into_any_element(),
        )
    }

    fn handle_event(&mut self, event: &ToggleButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ToggleButtonEvent::Change { selected } => {
                self.selected = *selected;
                cx.notify();
            }
        }
    }
}
