use std::f32::consts::PI;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, Context, DragMoveEvent, Entity, IntoElement, MouseDownEvent, MouseUpEvent, Pixels, Render,
    SharedString, Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::color::color_slider::ColorSliderBuilder;
use gpui_luma::controls::slider::{
    Slider, SliderInputStrategy, SliderThumbPolicy, SliderBoundsHandler, SliderDrag, SliderEvent, SliderHoverHandler,
    SliderMouseDownHandler, SliderMouseUpHandler, SliderRenderModel, SliderTemplate, SliderTemplateHandlers,
    SliderThumbRole, SliderThumbSize, SliderThumbValue, ThumbId, TrackPresentation, build_track_segments,
};
use gpui_luma::controls::value::ControlRange;
use gpui_luma::theme::{ControlSize, InteractionState};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_slider_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_scrollable_with_inspector, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct SliderPane {
    fill_slider: Slider,
    vertical_slider: Slider,
    vertical_reversed_slider: Slider,
    blocked_slider: Slider,
    reversed_slider: Slider,
    angular_slider: Slider,
    wrapping_slider: Slider,
    stops_slider: Slider,
    slider_sm: Slider,
    slider_md: Slider,
    slider_lg: Slider,
    slider_large_thumb: Slider,
    slider_sharp: Slider,
    color_hue_slider: Slider,
    state_preview: Entity<SliderStatePreview>,
    inspector: Entity<ColorInspectorShell>,
    fill_value: f32,
    vertical_value: f32,
    vertical_reversed_value: f32,
    blocked_value: f32,
    reversed_value: f32,
    angular_value: f32,
    wrapping_value: f32,
    selected_stop: Option<ThumbId>,
    stop_summary: String,
    stops_interaction_hint: Option<String>,
    color_hue_value: f32,
}

impl SliderPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let slider_template = look.slider_template();
        let tree = spawn_color_inspector_tree("slider-inspector-tree", look.clone(), build_slider_inspect_tree, cx);
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "slider-inspector",
                "slider-inspector-split",
                "slider-inspector-detail",
                build_slider_inspect_tree,
                cx,
            )
        });
        let stops_policy = SliderThumbPolicy::multi_stop();

        Self {
            fill_slider: look.slider("slider-fill").range(1..100).step(1).value(41).spawn(cx),
            vertical_slider: look.slider("slider-vertical").vertical().range(1..100).step(1).value(62).spawn(cx),
            vertical_reversed_slider: look
                .slider("slider-vertical-reversed")
                .vertical()
                .reversed(true)
                .range(1..100)
                .step(1)
                .value(62)
                .spawn(cx),
            blocked_slider: look
                .slider("slider-blocked")
                .range(0.0..360.0)
                .step(10.0)
                .value(100.0)
                .allowed_intervals(vec![0.0..=120.0, 180.0..=240.0, 300.0..=360.0])
                .spawn(cx),
            reversed_slider: look.slider("slider-reversed").reversed(true).range(1..100).step(1).value(41).spawn(cx),
            angular_slider: look
                .slider("slider-angular")
                .angular(-1.25 * PI, 0.25 * PI)
                .template(look.slider_angular_template())
                .range(0..100)
                .step(1)
                .value(50)
                .spawn(cx),
            wrapping_slider: look
                .slider("slider-wrapping")
                .angular(0.0, 2.0 * PI)
                .wrapping(true)
                .domain()
                .template(look.slider_circular_ring_template())
                .range(0.0..360.0)
                .step(1.0)
                .value(180.0)
                .spawn(cx),
            stops_slider: look
                .slider("slider-stops")
                .multi_stop()
                .range(0.0..360.0)
                .step(1.0)
                .thumb_values([(0.0, None), (180.0, None), (300.0, None)])
                .spawn(cx),
            slider_sm: look.slider("slider-size-sm").size(ControlSize::Sm).range(1..100).step(10).value(28).spawn(cx),
            slider_md: look.slider("slider-size-md").size(ControlSize::Md).range(1..100).step(10).value(41).spawn(cx),
            slider_lg: look.slider("slider-size-lg").size(ControlSize::Lg).range(1..100).step(10).value(62).spawn(cx),
            slider_large_thumb: look
                .slider("slider-thumb-lg")
                .size(ControlSize::Md)
                .thumb_size(SliderThumbSize::Lg)
                .range(1..100)
                .step(10)
                .value(54)
                .spawn(cx),
            slider_sharp: look
                .slider("slider-sharp-preview")
                .size(ControlSize::Md)
                .range(1..100)
                .step(10)
                .value(54)
                .corner_radius(px(0.0).into())
                .spawn(cx),
            color_hue_slider: ColorSliderBuilder::hue("slider-color-hue", 180.0).spawn(cx),
            state_preview: cx.new(|_| SliderStatePreview::new(look.clone(), slider_template)),
            inspector,
            fill_value: 41.0,
            vertical_value: 62.0,
            vertical_reversed_value: 62.0,
            blocked_value: 100.0,
            reversed_value: 41.0,
            angular_value: 50.0,
            wrapping_value: 180.0,
            selected_stop: None,
            stop_summary: String::new(),
            stops_interaction_hint: multi_stop_interaction_hint(stops_policy),
            color_hue_value: 180.0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.fill_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.slider.handle_fill_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.vertical_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.slider.handle_vertical_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.vertical_reversed_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.slider.handle_vertical_reversed_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.blocked_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.slider.handle_blocked_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.reversed_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.slider.handle_reversed_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.angular_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.slider.handle_angular_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.wrapping_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.slider.handle_wrapping_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.stops_slider, |app, _, event: &SliderEvent, cx| {
            let slider = app.panes.slider.stops_slider.clone();
            app.panes.slider.handle_stops_event(&slider, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.color_hue_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.slider.handle_color_hue_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();

        gallery_pane_scrollable_with_inspector(
            "Slider",
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
                .child(section_label("Color spectrum (ColorSliderBuilder)", chrome.muted_text))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(320.0)).child(self.color_hue_slider.clone()))
                        .child(value_label(self.color_hue_value, chrome.body_text)),
                )
                .child(section_label("Slider sizes", chrome.muted_text))
                .child(labeled_demo("Sm", div().w(px(320.0)).child(self.slider_sm.clone()), chrome.muted_text))
                .child(labeled_demo("Md", div().w(px(320.0)).child(self.slider_md.clone()), chrome.muted_text))
                .child(labeled_demo("Lg", div().w(px(320.0)).child(self.slider_lg.clone()), chrome.muted_text))
                .child(section_label("Thumb sizing", chrome.muted_text))
                .child(labeled_demo(
                    "Md control · Lg thumb",
                    div().w(px(320.0)).child(self.slider_large_thumb.clone()),
                    chrome.muted_text,
                ))
                .child(section_label("Corner radius overrides", chrome.muted_text))
                .child(labeled_demo(
                    "Slider · square track",
                    div().w(px(320.0)).child(self.slider_sharp.clone()),
                    chrome.muted_text,
                ))
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
                .child(self.state_preview.clone())
                .into_any_element(),
            self.inspector.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.fill_slider, cx);
        notify_entity(&self.vertical_slider, cx);
        notify_entity(&self.vertical_reversed_slider, cx);
        notify_entity(&self.blocked_slider, cx);
        notify_entity(&self.reversed_slider, cx);
        notify_entity(&self.angular_slider, cx);
        notify_entity(&self.wrapping_slider, cx);
        notify_entity(&self.stops_slider, cx);
        notify_entity(&self.slider_sm, cx);
        notify_entity(&self.slider_md, cx);
        notify_entity(&self.slider_lg, cx);
        notify_entity(&self.slider_large_thumb, cx);
        notify_entity(&self.slider_sharp, cx);
        notify_entity(&self.state_preview, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn handle_fill_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
            self.fill_value = *value;
            cx.notify();
        }
    }

    fn handle_color_hue_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
            self.color_hue_value = *value;
            cx.notify();
        }
    }

    fn handle_vertical_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
            self.vertical_value = *value;
            cx.notify();
        }
    }

    fn handle_vertical_reversed_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
            self.vertical_reversed_value = *value;
            cx.notify();
        }
    }

    fn handle_reversed_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
            self.reversed_value = *value;
            cx.notify();
        }
    }

    fn handle_blocked_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
            self.blocked_value = *value;
            cx.notify();
        }
    }

    fn handle_angular_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
            self.angular_value = *value;
            cx.notify();
        }
    }

    fn handle_wrapping_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
            self.wrapping_value = *value;
            cx.notify();
        }
    }

    fn handle_stops_event(&mut self, slider: &Slider, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, SliderEvent::ThumbSelected { .. }) {
            if let SliderEvent::ThumbSelected { thumb_id } = event {
                self.selected_stop = Some(*thumb_id);
            }
        }

        self.sync_stop_summary(slider, cx);
        cx.notify();
    }

    fn sync_stop_summary(&mut self, slider: &Slider, cx: &mut Context<GalleryApp>) {
        let slider = slider.read(cx);
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

#[derive(Clone)]
struct SliderStatePreview {
    look: Arc<ShadcnLook>,
    template: Arc<dyn SliderTemplate>,
}

struct SliderStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl SliderStatePreview {
    fn new(look: Arc<ShadcnLook>, template: Arc<dyn SliderTemplate>) -> Self {
        Self { look, template }
    }
}

impl Render for SliderStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let samples = [
            SliderStateSample { id: "default", label: "Standard", state: InteractionState::default() },
            SliderStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            SliderStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            SliderStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            SliderStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(10.0))
            .child(section_label("Template state preview", chrome.muted_text))
            .child(
                div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                    samples
                        .into_iter()
                        .map(|sample| render_state_sample(&self.template, sample, chrome.muted_text, window, cx)),
                ),
            )
    }
}

fn render_state_sample(
    template: &Arc<dyn SliderTemplate>,
    sample: SliderStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("slider-preview-{}", sample.id));
    let range = ControlRange::from(1..100);
    let value = 41.0;
    let position = range.percentage(value);
    let thumb_id = ThumbId::next();
    let thumbs = [SliderThumbValue { id: thumb_id, position, preview: None, role: SliderThumbRole::Value }];
    let track_segments = build_track_segments(TrackPresentation::Fill, position, &[], range);
    let model = SliderRenderModel {
        id: &id,
        strategy: SliderInputStrategy::Horizontal,
        orientation: SliderInputStrategy::Horizontal.orientation(),
        presentation: TrackPresentation::Fill,
        size: ControlSize::Md,
        thumb_size: None,
        range,
        step: 10.0,
        thumbs: &thumbs,
        track_segments,
        reversed: false,
        wrapping: false,
        enabled: !sample.state.disabled,
        corner_radius: None,
        thumb_policy: SliderThumbPolicy::default(),
        active_thumb_id: Some(thumb_id),
        state: sample.state,
        domain_track: None,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(div().w(px(320.0)).max_w(px(360.0)).child(template.render(
            &model,
            slider_preview_handlers(),
            thumb_id,
            window,
            cx,
        )))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn slider_preview_handlers() -> SliderTemplateHandlers {
    SliderTemplateHandlers {
        track_bounds: Box::new(noop_bounds) as SliderBoundsHandler,
        hover: Box::new(noop_hover) as SliderHoverHandler,
        mouse_down: Box::new(noop_mouse_down) as SliderMouseDownHandler,
        mouse_up: Box::new(noop_mouse_up) as SliderMouseUpHandler,
        mouse_up_out: Box::new(noop_mouse_up) as SliderMouseUpHandler,
        drag_move: Arc::new(noop_drag_move),
        thumb_mouse_down: Arc::new(noop_thumb_mouse_down),
    }
}

fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

fn noop_drag_move(_: &DragMoveEvent<SliderDrag>, _: &mut Window, _: &mut App) {}

fn noop_thumb_mouse_down(_: &ThumbId, _: &MouseDownEvent, _: &mut Window, _: &mut App) {}

fn vertical_demo(label: &'static str, slider: Slider, value: f32, look: &ShadcnLook) -> impl IntoElement {
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

fn stops_label(summary: &str, color: gpui::Hsla) -> impl IntoElement {
    div().text_size(px(12.0)).line_height(px(16.0)).text_color(color).child(summary.to_string())
}

fn labeled_demo(label: &str, element: impl IntoElement, label_color: gpui::Hsla) -> AnyElement {
    let label = label.to_string();

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(element)
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(label))
        .into_any_element()
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
