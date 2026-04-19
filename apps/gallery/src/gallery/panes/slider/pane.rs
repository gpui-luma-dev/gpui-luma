use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, rgb};
use gpui_luma::controls::slider::{Slider, SliderEvent};

use crate::gallery::control::GalleryApp;

use super::super::shared::gallery_pane;

pub(in crate::gallery) struct SliderPane {
    slider: Entity<Slider>,
    disabled_slider: Entity<Slider>,
    value: f32,
}

impl SliderPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            slider: Slider::new("slider-example").range(1..100).step(10).value(41).spawn(cx),
            disabled_slider: Slider::new("disabled-slider-example")
                .range(1..100)
                .step(10)
                .value(41)
                .enabled(false)
                .spawn(cx),
            value: 41.0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.slider, |app, _, event: &SliderEvent, cx| {
            app.panes.slider.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self) -> AnyElement {
        gallery_pane(
            "Slider",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(
                    div().flex().items_center().gap_3().child(self.slider.clone()).child(self.disabled_slider.clone()),
                )
                .child(div().text_color(rgb(0x334155)).child(format!("Value: {:.0}", self.value)))
                .into_any_element(),
        )
    }

    fn handle_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        match event {
            SliderEvent::Change { value } => {
                self.value = *value;
                cx.notify();
            }
        }
    }
}
