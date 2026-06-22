use std::f32::consts::PI;
use std::sync::Arc;

use gpui::{AnyElement, Context, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::slider2::{Slider2, Slider2Event, default_hue_domain_track};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

use super::super::shared::{gallery_pane_scrollable, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct Slider2Pane {
    fill_slider: Slider2,
    vertical_slider: Slider2,
    vertical_reversed_slider: Slider2,
    domain_slider: Slider2,
    blocked_slider: Slider2,
    reversed_slider: Slider2,
    angular_slider: Slider2,
    wrapping_slider: Slider2,
    fill_value: f32,
    vertical_value: f32,
    vertical_reversed_value: f32,
    domain_value: f32,
    blocked_value: f32,
    reversed_value: f32,
    angular_value: f32,
    wrapping_value: f32,
}

impl Slider2Pane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        Self {
            fill_slider: look.slider2("slider2-fill").range(1..100).step(1).value(41).spawn(cx),
            vertical_slider: look.slider2("slider2-vertical").vertical().range(1..100).step(1).value(62).spawn(cx),
            vertical_reversed_slider: look
                .slider2("slider2-vertical-reversed")
                .vertical()
                .reversed(true)
                .range(1..100)
                .step(1)
                .value(62)
                .spawn(cx),
            domain_slider: look
                .slider2("slider2-domain")
                .domain_track(default_hue_domain_track())
                .range(0.0..360.0)
                .step(1.0)
                .value(180.0)
                .spawn(cx),
            blocked_slider: look
                .slider2("slider2-blocked")
                .range(0.0..360.0)
                .step(10.0)
                .value(100.0)
                .allowed_intervals(vec![0.0..=120.0, 180.0..=240.0, 300.0..=360.0])
                .spawn(cx),
            reversed_slider: look.slider2("slider2-reversed").reversed(true).range(1..100).step(1).value(41).spawn(cx),
            angular_slider: look
                .slider2("slider2-angular")
                .angular(-1.25 * PI, 0.25 * PI)
                .template(look.slider2_angular_template())
                .range(0..100)
                .step(1)
                .value(50)
                .spawn(cx),
            wrapping_slider: look
                .slider2("slider2-wrapping")
                .angular(0.0, 2.0 * PI)
                .wrapping(true)
                .template(look.slider2_circular_ring_template())
                .range(0.0..360.0)
                .step(1.0)
                .value(180.0)
                .spawn(cx),
            fill_value: 41.0,
            vertical_value: 62.0,
            vertical_reversed_value: 62.0,
            domain_value: 180.0,
            blocked_value: 100.0,
            reversed_value: 41.0,
            angular_value: 50.0,
            wrapping_value: 180.0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.fill_slider, |app, _, event: &Slider2Event, cx| {
            app.panes.slider2.handle_fill_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.vertical_slider, |app, _, event: &Slider2Event, cx| {
            app.panes.slider2.handle_vertical_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.vertical_reversed_slider, |app, _, event: &Slider2Event, cx| {
            app.panes.slider2.handle_vertical_reversed_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.domain_slider, |app, _, event: &Slider2Event, cx| {
            app.panes.slider2.handle_domain_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.blocked_slider, |app, _, event: &Slider2Event, cx| {
            app.panes.slider2.handle_blocked_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.reversed_slider, |app, _, event: &Slider2Event, cx| {
            app.panes.slider2.handle_reversed_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.angular_slider, |app, _, event: &Slider2Event, cx| {
            app.panes.slider2.handle_angular_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.wrapping_slider, |app, _, event: &Slider2Event, cx| {
            app.panes.slider2.handle_wrapping_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();

        gallery_pane_scrollable(
            "Slider 2",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_6()
                .child(section_label("Fill track", chrome.muted_text))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(320.0)).child(self.fill_slider.clone()))
                        .child(value_label(self.fill_value, chrome.body_text)),
                )
                .child(section_label("Vertical fill track", chrome.muted_text))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_start()
                        .justify_center()
                        .gap_8()
                        .child(vertical_demo("Default", self.vertical_slider.clone(), self.vertical_value, look))
                        .child(vertical_demo(
                            "Reversed",
                            self.vertical_reversed_slider.clone(),
                            self.vertical_reversed_value,
                            look,
                        )),
                )
                .child(section_label("Reversed fill track", chrome.muted_text))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(320.0)).child(self.reversed_slider.clone()))
                        .child(value_label(self.reversed_value, chrome.body_text)),
                )
                .child(section_label("Domain track (hue spectrum)", chrome.muted_text))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(320.0)).child(self.domain_slider.clone()))
                        .child(value_label(self.domain_value, chrome.body_text)),
                )
                .child(section_label("Blocked intervals", chrome.muted_text))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(320.0)).child(self.blocked_slider.clone()))
                        .child(value_label(self.blocked_value, chrome.body_text)),
                )
                .child(section_label("Angular dial", chrome.muted_text))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_2()
                        .child(self.angular_slider.clone())
                        .child(value_label(self.angular_value, chrome.body_text)),
                )
                .child(section_label("Wrapping dial", chrome.muted_text))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_2()
                        .child(self.wrapping_slider.clone())
                        .child(value_label(self.wrapping_value, chrome.body_text)),
                )
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.fill_slider, cx);
        notify_entity(&self.vertical_slider, cx);
        notify_entity(&self.vertical_reversed_slider, cx);
        notify_entity(&self.domain_slider, cx);
        notify_entity(&self.blocked_slider, cx);
        notify_entity(&self.reversed_slider, cx);
        notify_entity(&self.angular_slider, cx);
        notify_entity(&self.wrapping_slider, cx);
    }

    fn handle_fill_event(&mut self, event: &Slider2Event, cx: &mut Context<GalleryApp>) {
        let (Slider2Event::Change { value } | Slider2Event::Release { value }) = event;
        self.fill_value = *value;
        cx.notify();
    }

    fn handle_vertical_event(&mut self, event: &Slider2Event, cx: &mut Context<GalleryApp>) {
        let (Slider2Event::Change { value } | Slider2Event::Release { value }) = event;
        self.vertical_value = *value;
        cx.notify();
    }

    fn handle_vertical_reversed_event(&mut self, event: &Slider2Event, cx: &mut Context<GalleryApp>) {
        let (Slider2Event::Change { value } | Slider2Event::Release { value }) = event;
        self.vertical_reversed_value = *value;
        cx.notify();
    }

    fn handle_domain_event(&mut self, event: &Slider2Event, cx: &mut Context<GalleryApp>) {
        let (Slider2Event::Change { value } | Slider2Event::Release { value }) = event;
        self.domain_value = *value;
        cx.notify();
    }

    fn handle_reversed_event(&mut self, event: &Slider2Event, cx: &mut Context<GalleryApp>) {
        let (Slider2Event::Change { value } | Slider2Event::Release { value }) = event;
        self.reversed_value = *value;
        cx.notify();
    }

    fn handle_blocked_event(&mut self, event: &Slider2Event, cx: &mut Context<GalleryApp>) {
        let (Slider2Event::Change { value } | Slider2Event::Release { value }) = event;
        self.blocked_value = *value;
        cx.notify();
    }

    fn handle_angular_event(&mut self, event: &Slider2Event, cx: &mut Context<GalleryApp>) {
        let (Slider2Event::Change { value } | Slider2Event::Release { value }) = event;
        self.angular_value = *value;
        cx.notify();
    }

    fn handle_wrapping_event(&mut self, event: &Slider2Event, cx: &mut Context<GalleryApp>) {
        let (Slider2Event::Change { value } | Slider2Event::Release { value }) = event;
        self.wrapping_value = *value;
        cx.notify();
    }
}

fn vertical_demo(label: &'static str, slider: Slider2, value: f32, look: &ShadcnLook) -> impl IntoElement {
    let chrome = look.chrome();
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_2()
        .child(div().text_size(px(11.0)).line_height(px(14.0)).text_color(chrome.muted_text).child(label))
        .child(div().h(px(260.0)).child(slider))
        .child(value_label(value, chrome.body_text))
}

fn section_label(label: &'static str, color: gpui::Hsla) -> impl IntoElement {
    div().text_size(px(12.0)).line_height(px(16.0)).text_color(color).child(label)
}

fn value_label(value: f32, color: gpui::Hsla) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .text_color(color)
        .child(format!("Value: {:.0}", value))
}
