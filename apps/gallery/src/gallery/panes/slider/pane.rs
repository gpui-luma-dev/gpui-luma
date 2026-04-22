use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*};
use gpui_luma::controls::slider::{Slider, SliderEvent};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct SliderPane {
    slider: Entity<Slider>,
    disabled_slider: Entity<Slider>,
    value: f32,
}

impl SliderPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            slider: Slider::new("slider-example")
                .range(1..100)
                .step(10)
                .value(41)
                .template(theme.slider_template())
                .spawn(cx),
            disabled_slider: Slider::new("disabled-slider-example")
                .range(1..100)
                .step(10)
                .value(41)
                .enabled(false)
                .template(theme.slider_template())
                .spawn(cx),
            value: 41.0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.slider, |app, _, event: &SliderEvent, cx| {
            app.panes.slider.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage(
            "Slider",
            "Slider",
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
                        .child(self.slider.clone())
                        .child(self.disabled_slider.clone()),
                )
                .child(div().text_color(chrome.body_text).child(format!("Value: {:.0}", self.value)))
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.slider, cx);
        notify_entity(&self.disabled_slider, cx);
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
