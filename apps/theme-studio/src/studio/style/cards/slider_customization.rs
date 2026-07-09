use std::f32::consts::PI;
use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, Render, Window, div, prelude::*, px};
use gpui_luma::controls::slider::{Slider, SliderEvent, SliderThumbPolicy, ThumbId};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

pub(in crate::studio::style::style_guide) struct SliderCustomizationPreview {
    look: Arc<ShadcnLook>,
    reversed_slider: Slider,
    blocked_slider: Slider,
    stops_slider: Slider,
    angular_slider: Slider,
    wrapping_slider: Slider,
    reversed_value: f32,
    blocked_value: f32,
    angular_value: f32,
    wrapping_value: f32,
    selected_stop: Option<ThumbId>,
    stop_summary: String,
    stops_interaction_hint: Option<String>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl SliderCustomizationPreview {
    pub(in crate::studio::style::style_guide) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let stops_policy = SliderThumbPolicy::multi_stop();
        let reversed_slider =
            look.slider("theme-studio-slider-reversed").reversed(true).range(1..100).step(1).value(41).spawn(cx);
        let blocked_slider = look
            .slider("theme-studio-slider-blocked")
            .range(0.0..360.0)
            .step(10.0)
            .value(100.0)
            .allowed_intervals(vec![0.0..=120.0, 180.0..=240.0, 300.0..=360.0])
            .spawn(cx);
        let stops_slider = look
            .slider("theme-studio-slider-stops")
            .multi_stop()
            .range(0.0..360.0)
            .step(1.0)
            .thumb_values([(0.0, None), (180.0, None), (300.0, None)])
            .spawn(cx);
        let angular_slider = look
            .slider("theme-studio-slider-angular")
            .angular(-1.25 * PI, 0.25 * PI)
            .template(look.slider_angular_template())
            .range(0..100)
            .step(1)
            .value(50)
            .spawn(cx);
        let wrapping_slider = look
            .slider("theme-studio-slider-wrapping")
            .angular(0.0, 2.0 * PI)
            .wrapping(true)
            .domain()
            .template(look.slider_circular_ring_template())
            .range(0.0..360.0)
            .step(1.0)
            .value(180.0)
            .spawn(cx);

        let mut preview = Self {
            look,
            reversed_slider,
            blocked_slider,
            stops_slider,
            angular_slider,
            wrapping_slider,
            reversed_value: 41.0,
            blocked_value: 100.0,
            angular_value: 50.0,
            wrapping_value: 180.0,
            selected_stop: None,
            stop_summary: String::new(),
            stops_interaction_hint: multi_stop_interaction_hint(stops_policy),
            _subscriptions: Vec::new(),
        };
        preview.sync_stop_summary(cx);
        preview.subscribe(cx);
        preview
    }

    pub(in crate::studio::style::style_guide) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.reversed_slider.update(cx, |slider, cx| {
            slider.set_template(look.slider_template(), cx);
        });
        self.blocked_slider.update(cx, |slider, cx| {
            slider.set_template(look.slider_template(), cx);
        });
        self.stops_slider.update(cx, |slider, cx| {
            slider.set_template(look.slider_template(), cx);
        });
        self.angular_slider.update(cx, |slider, cx| {
            slider.set_template(look.slider_angular_template(), cx);
        });
        self.wrapping_slider.update(cx, |slider, cx| {
            slider.set_template(look.slider_circular_ring_template(), cx);
        });
        cx.notify();
    }

    fn subscribe(&mut self, cx: &mut Context<Self>) {
        self._subscriptions.push(cx.subscribe(&self.reversed_slider, |this, _, event: &SliderEvent, cx| {
            if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
                this.reversed_value = *value;
                cx.notify();
            }
        }));
        self._subscriptions.push(cx.subscribe(&self.blocked_slider, |this, _, event: &SliderEvent, cx| {
            if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
                this.blocked_value = *value;
                cx.notify();
            }
        }));
        self._subscriptions.push(cx.subscribe(&self.angular_slider, |this, _, event: &SliderEvent, cx| {
            if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
                this.angular_value = *value;
                cx.notify();
            }
        }));
        self._subscriptions.push(cx.subscribe(&self.wrapping_slider, |this, _, event: &SliderEvent, cx| {
            if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
                this.wrapping_value = *value;
                cx.notify();
            }
        }));
        self._subscriptions.push(cx.subscribe(&self.stops_slider, |this, _, event: &SliderEvent, cx| {
            if let SliderEvent::ThumbSelected { thumb_id } = event {
                this.selected_stop = Some(*thumb_id);
            }
            this.sync_stop_summary(cx);
            cx.notify();
        }));
    }

    fn sync_stop_summary(&mut self, cx: &mut Context<Self>) {
        let slider = self.stops_slider.read(cx);
        let mut stops = slider.thumbs().iter().map(|thumb| slider.range().value_at(thumb.position)).collect::<Vec<_>>();
        stops.sort_by(|left, right| left.total_cmp(right));

        self.selected_stop = slider.active_thumb_id();
        let selected_value = self
            .selected_stop
            .and_then(|id| slider.thumb_value(id))
            .unwrap_or(stops.first().copied().unwrap_or(0.0));

        self.stop_summary = format!(
            "{} stops [{}] · selected {:.0}",
            stops.len(),
            stops.iter().map(|value| format!("{value:.0}")).collect::<Vec<_>>().join(", "),
            selected_value
        );
    }
}

impl Render for SliderCustomizationPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();

        div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .gap_6()
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
            .child(section_label("Multi-stop", chrome.muted_text))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .when_some(self.stops_interaction_hint.as_ref(), |this, hint| {
                        this.child(
                            div()
                                .text_size(px(11.0))
                                .line_height(px(14.0))
                                .text_color(chrome.muted_text)
                                .child(hint.clone()),
                        )
                    })
                    .child(div().w(px(320.0)).child(self.stops_slider.clone()))
                    .child(stops_label(&self.stop_summary, chrome.body_text)),
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
    }
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

fn stops_label(summary: &str, color: gpui::Hsla) -> impl IntoElement {
    div().text_size(px(12.0)).line_height(px(16.0)).text_color(color).child(summary.to_string())
}

fn multi_stop_interaction_hint(policy: SliderThumbPolicy) -> Option<String> {
    let mut parts = Vec::new();
    if policy.supports_click_to_add() {
        parts.push("Click empty track to add");
    }
    if policy.supports_delete_to_remove() {
        parts.push("Delete/Backspace to remove selected");
    }

    (!parts.is_empty()).then(|| parts.join(" · "))
}

pub(in crate::studio::style::style_guide) fn render_slider_customization_body(
    preview: Entity<SliderCustomizationPreview>,
) -> AnyElement {
    preview.into_any_element()
}
