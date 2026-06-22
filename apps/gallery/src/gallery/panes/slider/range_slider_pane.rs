use std::ops::RangeInclusive;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Background, BorderStyle, Bounds, Context, Corners, Div, DragMoveEvent, Edges, Entity, IntoElement,
    MouseButton, MouseDownEvent, MouseUpEvent, PaintQuad, Pixels, Render, SharedString, Stateful, Subscription, Window,
    canvas, div, hsla, linear_color_stop, linear_gradient, point, prelude::*, px, relative, size, transparent_black,
};
use gpui_luma::controls::color::style::StyledExt;
use gpui_luma::controls::range_slider::{
    RangeSlider, RangeSliderBoundsHandler, RangeSliderDrag, RangeSliderDragMoveHandler, RangeSliderEvent,
    RangeSliderHoverHandler, RangeSliderMouseDownHandler, RangeSliderMouseUpHandler, RangeSliderRenderModel,
    RangeSliderSegmentKind, RangeSliderTemplate, RangeSliderTemplateHandlers, RangeSliderTrackSegment,
    ThemedRangeSliderTemplate,
};
use gpui_luma::controls::slider::{Slider, SliderEvent, SliderLook, SliderOrientation, SliderTheme, SliderThumbSize};
use gpui_luma::controls::value::ControlRange;
use gpui_luma::theme::{ControlSize, InteractionState};
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn::prelude::*;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::notify_entity;

#[derive(Clone)]
pub(in crate::gallery) struct RangeSliderPane {
    default_slider: RangeSlider,
    size_sm_slider: RangeSlider,
    size_md_slider: RangeSlider,
    size_lg_slider: RangeSlider,
    large_thumb_slider: RangeSlider,
    slider: RangeSlider,
    state_preview: Entity<RangeSliderStatePreview>,
    segment_one_end_slider: Slider,
    segment_two_start_slider: Slider,
    segment_two_end_slider: Slider,
    segment_three_start_slider: Slider,
    value: f32,
    segment_one_end: f32,
    segment_two_start: f32,
    segment_two_end: f32,
    segment_three_start: f32,
}

impl RangeSliderPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let segment_one_end = 120.0;
        let segment_two_start = 180.0;
        let segment_two_end = 240.0;
        let segment_three_start = 300.0;
        let default_slider = look
            .range_slider("range-slider-default")
            .range(0.0..=360.0)
            .step(10.0)
            .value(210.0)
            .allowed_intervals(vec![0.0..=360.0])
            .spawn(cx);

        let size_sm_slider = look
            .range_slider("range-slider-size-sm")
            .size(ControlSize::Sm)
            .range(0.0..=360.0)
            .step(10.0)
            .value(140.0)
            .allowed_intervals(vec![0.0..=360.0])
            .spawn(cx);
        let size_md_slider = look
            .range_slider("range-slider-size-md")
            .size(ControlSize::Md)
            .range(0.0..=360.0)
            .step(10.0)
            .value(210.0)
            .allowed_intervals(vec![0.0..=360.0])
            .spawn(cx);
        let size_lg_slider = look
            .range_slider("range-slider-size-lg")
            .size(ControlSize::Lg)
            .range(0.0..=360.0)
            .step(10.0)
            .value(280.0)
            .allowed_intervals(vec![0.0..=360.0])
            .spawn(cx);
        let large_thumb_slider = look
            .range_slider("range-slider-thumb-lg")
            .size(ControlSize::Md)
            .thumb_size(SliderThumbSize::Lg)
            .range(0.0..=360.0)
            .step(10.0)
            .value(240.0)
            .allowed_intervals(vec![0.0..=360.0])
            .spawn(cx);
        let slider = look
            .range_slider("range-slider-demo")
            .range(0.0..=360.0)
            .step(10.0)
            .value(210.0)
            .allowed_intervals(Self::intervals_from_values(
                segment_one_end,
                segment_two_start,
                segment_two_end,
                segment_three_start,
            ))
            .template(Arc::new(HueRangeSliderTemplate::new(look.slider_theme())))
            .spawn(cx);

        let state_preview = cx.new(|_| {
            RangeSliderStatePreview::new(look.clone(), Arc::new(ThemedRangeSliderTemplate::new(look.slider_theme())))
        });

        Self {
            default_slider,
            size_sm_slider,
            size_md_slider,
            size_lg_slider,
            large_thumb_slider,
            slider,
            state_preview,
            segment_one_end_slider: look
                .slider("range-slider-segment-one-end")
                .range(0.0..=360.0)
                .step(10.0)
                .value(segment_one_end)
                .spawn(cx),
            segment_two_start_slider: look
                .slider("range-slider-segment-two-start")
                .range(0.0..=360.0)
                .step(10.0)
                .value(segment_two_start)
                .spawn(cx),
            segment_two_end_slider: look
                .slider("range-slider-segment-two-end")
                .range(0.0..=360.0)
                .step(10.0)
                .value(segment_two_end)
                .spawn(cx),
            segment_three_start_slider: look
                .slider("range-slider-segment-three-start")
                .range(0.0..=360.0)
                .step(10.0)
                .value(segment_three_start)
                .spawn(cx),
            value: 210.0,
            segment_one_end,
            segment_two_start,
            segment_two_end,
            segment_three_start,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.slider, |app, _, event: &RangeSliderEvent, cx| {
            app.panes.range_slider.handle_range_slider_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.segment_one_end_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.range_slider.handle_segment_one_end_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.segment_two_start_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.range_slider.handle_segment_two_start_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.segment_two_end_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.range_slider.handle_segment_two_end_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.segment_three_start_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.range_slider.handle_segment_three_start_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();

        scrolling_gallery_pane(
            "Range Slider",
            "A hue-oriented range-constrained slider that greys out unreachable gamut gaps, snaps pointer input to the nearest valid edge, and skips gaps during keyboard stepping.",
            div()
                .w_full()
                .max_w(px(760.0))
                .pl(px(50.0))
                .flex()
                .flex_col()
                .gap(px(24.0))
                .child(section_label("Baseline range slider", chrome.muted_text))
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(value_row(
                            "Default",
                            "Full-range allowed interval [0..=360]".to_string(),
                            chrome.body_text,
                            chrome.muted_text,
                        ))
                        .child(div().w(px(420.0)).max_w_full().child(self.default_slider.clone())),
                )
                .child(section_label("Range slider sizes", chrome.muted_text))
                .child(labeled_demo(
                    "Sm",
                    div().w(px(420.0)).max_w_full().child(self.size_sm_slider.clone()),
                    chrome.muted_text,
                ))
                .child(labeled_demo(
                    "Md",
                    div().w(px(420.0)).max_w_full().child(self.size_md_slider.clone()),
                    chrome.muted_text,
                ))
                .child(labeled_demo(
                    "Lg",
                    div().w(px(420.0)).max_w_full().child(self.size_lg_slider.clone()),
                    chrome.muted_text,
                ))
                .child(section_label("Thumb sizing", chrome.muted_text))
                .child(labeled_demo(
                    "Md control · Lg thumb",
                    div().w(px(420.0)).max_w_full().child(self.large_thumb_slider.clone()),
                    chrome.muted_text,
                ))
                .child(section_label("Mock gamut hue slider", chrome.muted_text))
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(div().w(px(420.0)).max_w_full().child(self.slider.clone()))
                        .child(value_row(
                            "Value",
                            format!("{:.0} deg", self.value),
                            chrome.body_text,
                            chrome.muted_text,
                        ))
                        .child(value_row("Allowed", self.allowed_intervals_text(), chrome.body_text, chrome.muted_text))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.muted_text)
                                .child("Try dragging through the grey zones or focus the slider and use arrow keys."),
                        ),
                )
                .child(section_label("Interactive segment customizer", chrome.muted_text))
                .child(setting_row(
                    "Segment 1 end",
                    self.segment_one_end_slider.clone(),
                    self.segment_one_end,
                    chrome.body_text,
                    chrome.muted_text,
                ))
                .child(setting_row(
                    "Segment 2 start",
                    self.segment_two_start_slider.clone(),
                    self.segment_two_start,
                    chrome.body_text,
                    chrome.muted_text,
                ))
                .child(setting_row(
                    "Segment 2 end",
                    self.segment_two_end_slider.clone(),
                    self.segment_two_end,
                    chrome.body_text,
                    chrome.muted_text,
                ))
                .child(setting_row(
                    "Segment 3 start",
                    self.segment_three_start_slider.clone(),
                    self.segment_three_start,
                    chrome.body_text,
                    chrome.muted_text,
                ))
                .child(self.state_preview.clone())
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.default_slider, cx);
        notify_entity(&self.size_sm_slider, cx);
        notify_entity(&self.size_md_slider, cx);
        notify_entity(&self.size_lg_slider, cx);
        notify_entity(&self.large_thumb_slider, cx);
        notify_entity(&self.slider, cx);
        notify_entity(&self.segment_one_end_slider, cx);
        notify_entity(&self.segment_two_start_slider, cx);
        notify_entity(&self.segment_two_end_slider, cx);
        notify_entity(&self.segment_three_start_slider, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_range_slider_event(&mut self, event: &RangeSliderEvent, cx: &mut Context<GalleryApp>) {
        match event {
            RangeSliderEvent::Change { value } | RangeSliderEvent::Release { value } => {
                self.value = *value;
                cx.notify();
            }
        }
    }

    fn handle_segment_one_end_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        self.segment_one_end = slider_event_value(event);
        self.normalize_segments(cx);
    }

    fn handle_segment_two_start_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        self.segment_two_start = slider_event_value(event);
        self.normalize_segments(cx);
    }

    fn handle_segment_two_end_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        self.segment_two_end = slider_event_value(event);
        self.normalize_segments(cx);
    }

    fn handle_segment_three_start_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        self.segment_three_start = slider_event_value(event);
        self.normalize_segments(cx);
    }

    fn normalize_segments(&mut self, cx: &mut Context<GalleryApp>) {
        let segment_one_end = self.segment_one_end.clamp(0.0, 360.0);
        let segment_two_start = self.segment_two_start.clamp(segment_one_end, 360.0);
        let segment_two_end = self.segment_two_end.clamp(segment_two_start, 360.0);
        let segment_three_start = self.segment_three_start.clamp(segment_two_end, 360.0);

        self.segment_one_end = segment_one_end;
        self.segment_two_start = segment_two_start;
        self.segment_two_end = segment_two_end;
        self.segment_three_start = segment_three_start;

        self.segment_one_end_slider.update(cx, |slider, cx| slider.set_value(segment_one_end, cx));
        self.segment_two_start_slider.update(cx, |slider, cx| slider.set_value(segment_two_start, cx));
        self.segment_two_end_slider.update(cx, |slider, cx| slider.set_value(segment_two_end, cx));
        self.segment_three_start_slider.update(cx, |slider, cx| slider.set_value(segment_three_start, cx));

        let intervals = Self::intervals_from_values(
            self.segment_one_end,
            self.segment_two_start,
            self.segment_two_end,
            self.segment_three_start,
        );
        self.slider.update(cx, |slider, cx| slider.set_allowed_intervals(intervals, cx));
        self.value = self.slider.read(cx).value();
        cx.notify();
    }

    fn intervals_from_values(
        segment_one_end: f32,
        segment_two_start: f32,
        segment_two_end: f32,
        segment_three_start: f32,
    ) -> Vec<RangeInclusive<f32>> {
        vec![0.0..=segment_one_end, segment_two_start..=segment_two_end, segment_three_start..=360.0]
    }

    fn allowed_intervals_text(&self) -> String {
        Self::intervals_from_values(
            self.segment_one_end,
            self.segment_two_start,
            self.segment_two_end,
            self.segment_three_start,
        )
        .into_iter()
        .map(|interval| format!("[{:.0}..={:.0}]", interval.start(), interval.end()))
        .collect::<Vec<_>>()
        .join(", ")
    }
}

#[derive(Clone)]
struct RangeSliderStatePreview {
    look: Arc<ShadcnLook>,
    template: Arc<dyn RangeSliderTemplate>,
}

#[derive(Clone, Copy)]
struct RangeSliderStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl RangeSliderStatePreview {
    fn new(look: Arc<ShadcnLook>, template: Arc<dyn RangeSliderTemplate>) -> Self {
        Self { look, template }
    }
}

impl Render for RangeSliderStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let standard = RangeSliderStateSample { id: "default", label: "Standard", state: InteractionState::default() };
        let hover = RangeSliderStateSample {
            id: "hover",
            label: "Hover",
            state: InteractionState { hovered: true, ..InteractionState::default() },
        };
        let focus = RangeSliderStateSample {
            id: "focus",
            label: "Focus",
            state: InteractionState { focused: true, ..InteractionState::default() },
        };
        let active = RangeSliderStateSample {
            id: "active",
            label: "Active",
            state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
        };
        let disabled = RangeSliderStateSample {
            id: "disabled",
            label: "Disabled",
            state: InteractionState { disabled: true, ..InteractionState::default() },
        };

        div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(10.0))
            .child(section_label("Template state preview", chrome.muted_text))
            .child(preview_row(vec![
                render_range_slider_state_sample(&self.template, standard, chrome.muted_text, window, cx),
                render_range_slider_state_sample(&self.template, hover, chrome.muted_text, window, cx),
            ]))
            .child(preview_row(vec![
                render_range_slider_state_sample(&self.template, focus, chrome.muted_text, window, cx),
                render_range_slider_state_sample(&self.template, active, chrome.muted_text, window, cx),
            ]))
            .child(preview_row(vec![render_range_slider_state_sample(
                &self.template,
                disabled,
                chrome.muted_text,
                window,
                cx,
            )]))
    }
}

fn render_range_slider_state_sample(
    template: &Arc<dyn RangeSliderTemplate>,
    sample: RangeSliderStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("range-slider-preview-{}", sample.id));
    let range = ControlRange::from(0.0..=360.0);
    let allowed_intervals = vec![0.0..=360.0];
    let model = RangeSliderRenderModel {
        id: &id,
        orientation: SliderOrientation::Horizontal,
        size: ControlSize::Md,
        thumb_size: None,
        range,
        step: 10.0,
        value: 210.0,
        percentage: range.percentage(210.0),
        allowed_intervals: &allowed_intervals,
        track_segments: vec![RangeSliderTrackSegment {
            start_percentage: 0.0,
            end_percentage: 1.0,
            kind: RangeSliderSegmentKind::Allowed,
        }],
        enabled: !sample.state.disabled,
        corner_radius: None,
        state: sample.state,
    };

    div()
        .w(px(320.0))
        .max_w(px(320.0))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(div().w_full().child(template.render(&model, range_slider_preview_handlers(), window, cx)))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

#[derive(Clone)]
struct HueRangeSliderTemplate {
    theme: Arc<dyn SliderTheme>,
}

impl HueRangeSliderTemplate {
    fn new(theme: Arc<dyn SliderTheme>) -> Self {
        Self { theme }
    }
}

impl RangeSliderTemplate for HueRangeSliderTemplate {
    fn render(
        &self,
        model: &RangeSliderRenderModel<'_>,
        handlers: RangeSliderTemplateHandlers,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let RangeSliderTemplateHandlers { track_bounds, hover, mouse_down, mouse_up, mouse_up_out, drag_move } =
            handlers;
        let look = self.theme.resolve(model.size, model.thumb_size, model.state);
        let percentage = model.percentage.clamp(0.0, 1.0);
        let long_axis = look.width;
        let short_axis = look.height;
        let cross_axis = look.track_height;
        let root_height = match model.orientation {
            SliderOrientation::Horizontal => short_axis,
            SliderOrientation::Vertical => long_axis,
        };
        let root_width = short_axis;
        let segments = model.track_segments.clone();
        let range = model.range;

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .when(model.orientation == SliderOrientation::Horizontal, |this| {
                this.w_full().min_w(px(0.0)).h(px(root_height)).px(px(look.thumb_size * 0.5))
            })
            .when(model.orientation == SliderOrientation::Vertical, |this| this.w(px(root_width)).h(px(root_height)))
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out)
            .on_drag(RangeSliderDrag::new(model.id.clone()), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(drag_move);

        root = match model.orientation {
            SliderOrientation::Horizontal => {
                let track_top = (short_axis - cross_axis) * 0.5;
                let thumb_top = (short_axis - look.thumb_size) * 0.5 - thumb_focus_offset();
                let thumb_center_offset = -(look.thumb_size * 0.5 + thumb_focus_offset());
                let track_radius =
                    model.corner_radius.map(|radius| radius.to_pixels(window.rem_size())).unwrap_or(px(look.radius));

                let track_look = look.clone();
                let track = div()
                    .id(format!("{}-track", model.id))
                    .absolute()
                    .left(px(0.0))
                    .right(px(0.0))
                    .top(px(track_top))
                    .h(px(cross_axis))
                    .bg(look.track_background)
                    .rounded(track_radius)
                    .child(
                        canvas(
                            move |bounds, window, cx| track_bounds(&bounds, window, cx),
                            move |bounds, _, window, _| paint_hue_track(bounds, range, &segments, &track_look, window),
                        )
                        .absolute()
                        .size_full(),
                    );

                let thumb = default_thumb(model.id, &look, percentage, thumb_top, thumb_center_offset);
                root.child(track).child(thumb)
            }
            SliderOrientation::Vertical => {
                let track_left = (short_axis - cross_axis) * 0.5;
                let thumb_left = (short_axis - look.thumb_size) * 0.5;
                let thumb_top = (long_axis - look.thumb_size).max(0.0) * (1.0 - percentage);
                let track_radius =
                    model.corner_radius.map(|radius| radius.to_pixels(window.rem_size())).unwrap_or(px(look.radius));

                let track = div()
                    .id(format!("{}-track", model.id))
                    .absolute()
                    .left(px(track_left))
                    .top(px(0.0))
                    .w(px(cross_axis))
                    .h(px(long_axis))
                    .bg(look.track_background)
                    .rounded(track_radius)
                    .children(model.track_segments.iter().map(|segment| {
                        let start = segment.start_percentage.clamp(0.0, 1.0);
                        let end = segment.end_percentage.clamp(0.0, 1.0);
                        let height = (end - start).clamp(0.0, 1.0);
                        let background = match segment.kind {
                            RangeSliderSegmentKind::Allowed => look.fill_background,
                            RangeSliderSegmentKind::Gap => look.track_background,
                        };

                        div()
                            .absolute()
                            .left(px(0.0))
                            .bottom(relative(start))
                            .w_full()
                            .h(relative(height))
                            .bg(background)
                            .corner_radii(segment_corner_radii(start, end, model.orientation, track_radius))
                    }))
                    .child(
                        canvas(move |bounds, window, cx| track_bounds(&bounds, window, cx), |_, _, _, _| {})
                            .absolute()
                            .size_full(),
                    );

                let thumb = div()
                    .id(format!("{}-thumb", model.id))
                    .absolute()
                    .left(px(thumb_left - thumb_focus_offset()))
                    .top(px(thumb_top - thumb_focus_offset()))
                    .flex()
                    .items_center()
                    .justify_center()
                    .p(px(THUMB_FOCUS_GAP))
                    .border(px(THUMB_FOCUS_WIDTH))
                    .border_color(focus_ring_color(look.focus_ring))
                    .rounded(px(look.radius + thumb_focus_offset()))
                    .child(
                        div()
                            .size(px(look.thumb_size))
                            .bg(look.thumb_background)
                            .border_1()
                            .border_color(look.thumb_border)
                            .rounded(px(look.radius))
                            .shadow(look.thumb_shadow.clone()),
                    );

                root.child(track).child(thumb)
            }
        };

        if model.enabled {
            root = root.cursor_pointer();
        } else {
            root = root.opacity(0.56);
        }

        root
    }
}

fn default_thumb(
    id: &SharedString,
    look: &SliderLook,
    percentage: f32,
    thumb_top: f32,
    thumb_center_offset: f32,
) -> Stateful<Div> {
    div()
        .id(format!("{}-thumb", id))
        .absolute()
        .left(relative(percentage))
        .top(px(thumb_top))
        .ml(px(thumb_center_offset))
        .flex()
        .items_center()
        .justify_center()
        .p(px(THUMB_FOCUS_GAP))
        .border(px(THUMB_FOCUS_WIDTH))
        .border_color(focus_ring_color(look.focus_ring))
        .rounded(px(look.radius + thumb_focus_offset()))
        .child(
            div()
                .size(px(look.thumb_size))
                .bg(look.thumb_background)
                .border_1()
                .border_color(look.thumb_border)
                .rounded(px(look.radius))
                .shadow(look.thumb_shadow.clone()),
        )
}

fn paint_hue_track(
    bounds: Bounds<Pixels>,
    range: gpui_luma::controls::value::ControlRange,
    segments: &[gpui_luma::controls::range_slider::RangeSliderTrackSegment],
    look: &SliderLook,
    window: &mut Window,
) {
    if bounds.size.width <= px(0.0) || bounds.size.height <= px(0.0) {
        return;
    }

    for segment in segments {
        let start = segment.start_percentage.clamp(0.0, 1.0);
        let end = segment.end_percentage.clamp(0.0, 1.0);
        let segment_bounds = Bounds {
            origin: point(bounds.left() + bounds.size.width * start, bounds.top()),
            size: size((bounds.size.width * (end - start)).max(px(0.0)), bounds.size.height),
        };
        let corners = horizontal_segment_corner_radii(start, end, bounds.size.height / 2.0);

        match segment.kind {
            RangeSliderSegmentKind::Allowed => {
                let start_hue = range.value_at(start);
                let end_hue = range.value_at(end);
                paint_hue_segment(window, segment_bounds, start_hue, end_hue, corners);
            }
            RangeSliderSegmentKind::Gap => {
                paint_quad_fill(window, segment_bounds, look.track_background.into(), corners);
            }
        }
    }
}

fn paint_hue_segment(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    start_hue: f32,
    end_hue: f32,
    corners: Corners<Pixels>,
) {
    if bounds.size.width <= px(0.0) || bounds.size.height <= px(0.0) {
        return;
    }

    let band_count = (((end_hue - start_hue).abs() / 12.0).ceil() as usize).max(1);
    let band_size = bounds.size.width / band_count as f32;

    for index in 0..band_count {
        let t0 = index as f32 / band_count as f32;
        let t1 = (index + 1) as f32 / band_count as f32;
        let band_start_hue = start_hue + (end_hue - start_hue) * t0;
        let band_end_hue = start_hue + (end_hue - start_hue) * t1;
        let (start_offset, current_band_size) =
            calculate_overlapping_segment(index, band_count, band_size, bounds.size.width);
        let band_bounds = Bounds {
            origin: point(bounds.origin.x + start_offset, bounds.origin.y),
            size: size(current_band_size, bounds.size.height),
        };

        let band_corners = if band_count == 1 {
            corners
        } else if index == 0 {
            Corners {
                top_left: corners.top_left,
                top_right: px(0.0),
                bottom_left: corners.bottom_left,
                bottom_right: px(0.0),
            }
        } else if index == band_count - 1 {
            Corners {
                top_left: px(0.0),
                top_right: corners.top_right,
                bottom_left: px(0.0),
                bottom_right: corners.bottom_right,
            }
        } else {
            Corners::default()
        };

        window.paint_quad(PaintQuad {
            bounds: band_bounds,
            corner_radii: band_corners,
            background: linear_gradient(
                90.0,
                linear_color_stop(hsla((band_start_hue / 360.0).rem_euclid(1.0), 1.0, 0.5, 1.0), 0.0),
                linear_color_stop(hsla((band_end_hue / 360.0).rem_euclid(1.0), 1.0, 0.5, 1.0), 1.0),
            ),
            border_widths: Edges::default(),
            border_color: transparent_black(),
            border_style: BorderStyle::default(),
        });
    }
}

fn calculate_overlapping_segment(
    index: usize,
    count: usize,
    item_size: Pixels,
    total_size: Pixels,
) -> (Pixels, Pixels) {
    let start_offset = index as f32 * item_size;
    let end_offset = if index == count - 1 {
        total_size
    } else {
        (index + 1) as f32 * item_size + px(2.0)
    };
    (start_offset, end_offset - start_offset)
}

fn horizontal_segment_corner_radii(start: f32, end: f32, radius: Pixels) -> Corners<Pixels> {
    let mut corners = Corners::default();
    if start <= 0.0001 {
        corners.top_left = radius;
        corners.bottom_left = radius;
    }
    if end >= 0.9999 {
        corners.top_right = radius;
        corners.bottom_right = radius;
    }
    corners
}

fn segment_corner_radii(start: f32, end: f32, orientation: SliderOrientation, radius: Pixels) -> Corners<Pixels> {
    match orientation {
        SliderOrientation::Horizontal => horizontal_segment_corner_radii(start, end, radius),
        SliderOrientation::Vertical => {
            let mut corners = Corners::default();
            if start <= 0.0001 {
                corners.bottom_left = radius;
                corners.bottom_right = radius;
            }
            if end >= 0.9999 {
                corners.top_left = radius;
                corners.top_right = radius;
            }
            corners
        }
    }
}

fn paint_quad_fill(window: &mut Window, bounds: Bounds<Pixels>, background: Background, corner_radii: Corners<Pixels>) {
    window.paint_quad(PaintQuad {
        bounds,
        corner_radii,
        background,
        border_widths: Edges::default(),
        border_color: transparent_black(),
        border_style: BorderStyle::default(),
    });
}

fn slider_event_value(event: &SliderEvent) -> f32 {
    match event {
        SliderEvent::Change { value } | SliderEvent::Release { value } => *value,
    }
}

fn scrolling_gallery_pane(
    title: &'static str,
    description: &'static str,
    content: AnyElement,
    look: &ShadcnLook,
) -> AnyElement {
    let chrome = look.chrome();

    div()
        .size_full()
        .overflow_hidden()
        .bg(chrome.content_background)
        .p(px(28.0))
        .child(
            div()
                .id("range-slider-pane")
                .size_full()
                .flex()
                .flex_col()
                .gap(px(18.0))
                .overflow_y_scroll()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_size(px(30.0))
                                .line_height(px(36.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(chrome.title_text)
                                .child(title),
                        )
                        .child(
                            div()
                                .max_w(px(760.0))
                                .text_size(px(14.0))
                                .line_height(px(20.0))
                                .text_color(chrome.muted_text)
                                .child(description),
                        ),
                )
                .child(content),
        )
        .into_any_element()
}

fn preview_row(children: Vec<AnyElement>) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_wrap()
        .items_start()
        .justify_center()
        .gap(px(12.0))
        .children(children)
        .into_any_element()
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

fn range_slider_preview_handlers() -> RangeSliderTemplateHandlers {
    RangeSliderTemplateHandlers {
        track_bounds: Box::new(noop_bounds) as RangeSliderBoundsHandler,
        hover: Box::new(noop_hover) as RangeSliderHoverHandler,
        mouse_down: Box::new(noop_mouse_down) as RangeSliderMouseDownHandler,
        mouse_up: Box::new(noop_mouse_up) as RangeSliderMouseUpHandler,
        mouse_up_out: Box::new(noop_mouse_up) as RangeSliderMouseUpHandler,
        drag_move: Box::new(noop_drag_move) as RangeSliderDragMoveHandler,
    }
}

fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

fn noop_drag_move(_: &DragMoveEvent<RangeSliderDrag>, _: &mut Window, _: &mut App) {}

fn setting_row(
    label: &str,
    slider: Slider,
    value: f32,
    value_color: gpui::Hsla,
    label_color: gpui::Hsla,
) -> AnyElement {
    let label = label.to_string();

    div()
        .w_full()
        .max_w(px(520.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(value_row(&label, format!("{value:.0} deg"), value_color, label_color))
        .child(div().w_full().child(slider))
        .into_any_element()
}

fn value_row(label: &str, value: String, value_color: gpui::Hsla, label_color: gpui::Hsla) -> AnyElement {
    let label = label.to_string();

    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(16.0))
        .child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(label_color).child(label))
        .child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(value_color).child(value))
        .into_any_element()
}

fn section_label(label: &str, color: gpui::Hsla) -> AnyElement {
    let label = label.to_string();

    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(color)
        .child(label)
        .into_any_element()
}

const THUMB_FOCUS_GAP: f32 = 0.0;
const THUMB_FOCUS_WIDTH: f32 = 2.0;

fn thumb_focus_offset() -> f32 {
    THUMB_FOCUS_GAP + THUMB_FOCUS_WIDTH
}

fn focus_ring_color(focus_ring: Option<gpui::Hsla>) -> gpui::Hsla {
    focus_ring.unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 0.0))
}
