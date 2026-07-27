//! Color picker control exposition — Photoshop-style SV field, hue, and alpha composition.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::color::ColorSwatch;
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::color_slider::{
    AlphaDelegate, ColorSliderBuilder, ColorSliderDomainRenderer, primary_slider_value, sizing,
};
use gpui_luma::controls::color::composition::{ColorCompositionSync, CompositionSize};
use gpui_luma::controls::slider::{SliderControl, SliderEvent};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnRadius};

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::color_exposition_common::{
    composition_caption_text_size, composition_card_width, composition_inset_radius, composition_size_label,
    composition_title_text_size, detail_row_sized, format_color_field_event, format_compact_hsla, format_hex_color,
    format_slider_event, render_demo_section, render_labeled_demo_card_with_padding,
};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const PICKER_HORIZONTAL_PADDING: f32 = 25.0;
const PICKER_VERTICAL_PADDING: f32 = 18.0;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "ColorFieldEvent::Change / Release(Hsv)",
        trigger: "Drag inside the saturation/value field",
        notes: "Updates hue, saturation, and value on the shared HSV model.",
    },
    EventReferenceSpec {
        event: "SliderEvent::Change / Release { value }",
        trigger: "Drag hue or alpha sliders",
        notes: "Hue and alpha sliders stay linked through ColorCompositionSync.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "ColorFieldState + ColorSliderBuilder",
        surface: "Composition",
        notes: "Gallery Photoshop picker — SV field with hue and alpha sliders.",
    },
    PublicInterfaceSpec {
        symbol: "ColorCompositionSync",
        surface: "Sync",
        notes: "Prevents feedback loops while syncing linked field and slider values.",
    },
    PublicInterfaceSpec {
        symbol: "CompositionSize",
        surface: "Layout",
        notes: "Sm, Md, and Lg resolve control width and swatch height scaling.",
    },
    PublicInterfaceSpec {
        symbol: "ColorSwatch",
        surface: "Readout",
        notes: "Checkerboard-backed preview swatch below the controls.",
    },
];

pub struct ColorPickerControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    state_sm: Entity<ColorPickerDemo>,
    state_md: Entity<ColorPickerDemo>,
    state_lg: Entity<ColorPickerDemo>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ColorPickerControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("color-picker").expect("color-picker catalog entry");

        let state_sm = cx.new(|cx| ColorPickerDemo::with_size(look.clone(), CompositionSize::Sm, cx));
        let state_md = cx.new(|cx| ColorPickerDemo::with_size(look.clone(), CompositionSize::Md, cx));
        let state_lg = cx.new(|cx| ColorPickerDemo::with_size(look.clone(), CompositionSize::Lg, cx));

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-color-picker-event-log",
                "Edit the medium picker; ColorFieldEvent and SliderEvent variants appear below.",
            )
        });

        let subscriptions = wire_picker_events(&state_md, &event_stream, cx);

        Self { look, entry, state_sm, state_md, state_lg, event_stream, _subscriptions: subscriptions }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for state in [&self.state_sm, &self.state_md, &self.state_lg] {
            state.update(cx, |demo, cx| demo.sync_look(look.clone(), cx));
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(self.look.clone(), cx));
        cx.notify();
    }
}

impl Render for ColorPickerControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.0))
                .child(render_demo_section(
                    look,
                    "Photoshop-style Picker",
                    "A composed picker built from the migrated field and slider primitives.",
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(px(16.0))
                        .child(render_labeled_demo_card_with_padding(
                            look,
                            "Sm",
                            "Compact composition metrics.",
                            ColorPickerDemo::card_width_for(CompositionSize::Sm, PICKER_HORIZONTAL_PADDING),
                            PICKER_HORIZONTAL_PADDING,
                            PICKER_VERTICAL_PADDING,
                            composition_title_text_size(CompositionSize::Sm),
                            composition_caption_text_size(CompositionSize::Sm),
                            self.state_sm.clone(),
                        ))
                        .child(render_labeled_demo_card_with_padding(
                            look,
                            "Md",
                            "Default composition metrics.",
                            ColorPickerDemo::card_width_for(CompositionSize::Md, PICKER_HORIZONTAL_PADDING),
                            PICKER_HORIZONTAL_PADDING,
                            PICKER_VERTICAL_PADDING,
                            composition_title_text_size(CompositionSize::Md),
                            composition_caption_text_size(CompositionSize::Md),
                            self.state_md.clone(),
                        ))
                        .child(render_labeled_demo_card_with_padding(
                            look,
                            "Lg",
                            "Expanded composition metrics.",
                            ColorPickerDemo::card_width_for(CompositionSize::Lg, PICKER_HORIZONTAL_PADDING),
                            PICKER_HORIZONTAL_PADDING,
                            PICKER_VERTICAL_PADDING,
                            composition_title_text_size(CompositionSize::Lg),
                            composition_caption_text_size(CompositionSize::Lg),
                            self.state_lg.clone(),
                        ))
                        .into_any_element(),
                ))
                .child(self.event_stream.clone());

            render_control_exposition_card(
                look,
                self.entry,
                preview.into_any_element(),
                Some(render_exposition_doc_sections(look, EVENT_SPECS, PUBLIC_INTERFACE_SPECS)),
                ControlExpositionLayout::BORDERLESS,
            )
        })
    }
}

struct ColorPickerDemo {
    look: Arc<ShadcnLook>,
    composition_size: CompositionSize,
    metrics: ColorPickerMetrics,
    sync: ColorCompositionSync,
    field: Entity<ColorFieldState>,
    hue_slider: Entity<SliderControl>,
    alpha_slider: Entity<SliderControl>,
    alpha_domain: Arc<ColorSliderDomainRenderer>,
    hsv: Hsv,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy)]
struct ColorPickerMetrics {
    control_width: f32,
    swatch_height: f32,
}

impl ColorPickerMetrics {
    fn resolve(size: CompositionSize) -> Self {
        let control_width = size.resolve_primary(220.0, 260.0, 320.0);
        let scale = control_width / 260.0;
        Self { control_width, swatch_height: (44.0 * scale).max(32.0) }
    }

    fn card_width(self, horizontal_padding: f32) -> f32 {
        composition_card_width(self.control_width, horizontal_padding)
    }
}

impl ColorPickerDemo {
    pub fn card_width_for(size: CompositionSize, horizontal_padding: f32) -> f32 {
        ColorPickerMetrics::resolve(size).card_width(horizontal_padding)
    }

    fn with_size(look: Arc<ShadcnLook>, size: CompositionSize, cx: &mut Context<Self>) -> Self {
        let hsv = Hsv { h: 12.0, s: 0.78, v: 0.86, a: 0.92 };
        let metrics = ColorPickerMetrics::resolve(size);

        let size_label = composition_size_label(size);
        let field_radius = px(look.radius(ShadcnRadius::Sm));

        let field = cx.new(move |_| {
            ColorFieldState::saturation_value(
                format!("controls-doc-color-picker-field-{size_label}"),
                hsv,
                sizing::THUMB_SIZE_MEDIUM,
            )
            .rounded(field_radius)
            .vector()
        });
        let hue_slider = ColorSliderBuilder::hue(format!("controls-doc-color-picker-hue-{size_label}"), hsv.h)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge()
            .spawn(cx);
        let alpha_builder =
            ColorSliderBuilder::alpha(format!("controls-doc-color-picker-alpha-{size_label}"), hsv.a, hsv)
                .size(ControlSize::Sm)
                .thumb_medium()
                .edge_to_edge();
        let alpha_domain = alpha_builder.domain_renderer();
        let alpha_slider = alpha_builder.spawn(cx);

        let subscriptions = vec![
            cx.subscribe(&field, |this, _, event: &ColorFieldEvent, cx| {
                let hsv = match event {
                    ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv) => *hsv,
                    _ => return,
                };
                let Some(_sync_guard) = this.sync.begin_guard() else {
                    return;
                };
                this.hsv.h = hsv.h;
                this.hsv.s = hsv.s;
                this.hsv.v = hsv.v;
                this.sync_controls(cx, false);
                cx.notify();
            }),
            cx.subscribe(&hue_slider, |this, _, event, cx| {
                let Some(hue) = primary_slider_value(event) else {
                    return;
                };
                let Some(_sync_guard) = this.sync.begin_guard() else {
                    return;
                };
                this.hsv.h = hue;
                this.sync_controls(cx, true);
                cx.notify();
            }),
            cx.subscribe(&alpha_slider, |this, _, event, cx| {
                let Some(alpha) = primary_slider_value(event) else {
                    return;
                };
                let Some(_sync_guard) = this.sync.begin_guard() else {
                    return;
                };
                this.hsv.a = alpha;
                this.sync_controls(cx, true);
                cx.notify();
            }),
        ];

        Self {
            look,
            composition_size: size,
            metrics,
            field,
            hue_slider,
            alpha_slider,
            alpha_domain,
            sync: ColorCompositionSync::new(),
            hsv,
            _subscriptions: subscriptions,
        }
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        self.field.update(cx, |_, cx| cx.notify());
        self.hue_slider.update(cx, |_, cx| cx.notify());
        self.alpha_slider.update(cx, |_, cx| cx.notify());
        cx.notify();
    }

    fn sync_controls(&self, cx: &mut Context<Self>, sync_field: bool) {
        let hsv = self.hsv;

        if sync_field {
            self.field.update(cx, |field, cx| {
                field.set_hsv(hsv, cx);
            });
        }

        self.sync.sync_slider_value(&self.hue_slider, hsv.h, cx);
        self.sync.sync_color_slider(
            &self.alpha_slider,
            &self.alpha_domain,
            Arc::new(AlphaDelegate { spec: hsv }),
            self.alpha_domain.context(),
            hsv.a,
            cx,
        );
    }
}

impl Render for ColorPickerDemo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let selected = self.hsv.to_hsla_ext();
        let control_width = self.metrics.control_width;
        let swatch_height = self.metrics.swatch_height;
        let look = &self.look;
        let text_size = composition_title_text_size(self.composition_size);

        div()
            .w_full()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(14.0))
            .child(div().size(px(control_width)).child(self.field.clone()))
            .child(div().w(px(control_width)).child(self.hue_slider.clone()))
            .child(div().w(px(control_width)).child(self.alpha_slider.clone()))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .child(
                        ColorSwatch::new(selected)
                            .checkerboard(true)
                            .height(px(swatch_height))
                            .rounded(px(composition_inset_radius(look))),
                    )
                    .child(detail_row_sized(look, "Hex", format_hex_color(selected), text_size))
                    .child(detail_row_sized(look, "HSLA", format_compact_hsla(selected), text_size)),
            )
    }
}

fn wire_picker_events(
    state: &Entity<ColorPickerDemo>,
    event_stream: &Entity<ControlEventStream>,
    cx: &mut Context<ColorPickerControlExposition>,
) -> Vec<Subscription> {
    let field = state.read(cx).field.clone();
    let hue_slider = state.read(cx).hue_slider.clone();
    let alpha_slider = state.read(cx).alpha_slider.clone();
    let event_stream = event_stream.clone();

    vec![
        cx.subscribe(&field, {
            let event_stream = event_stream.clone();
            move |_, _, event: &ColorFieldEvent, cx| {
                if let Some(line) = format_color_field_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        }),
        cx.subscribe(&hue_slider, {
            let event_stream = event_stream.clone();
            move |_, _, event: &SliderEvent, cx| {
                if let Some(line) = format_slider_event("Hue", event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        }),
        cx.subscribe(&alpha_slider, move |_, _, event: &SliderEvent, cx| {
            if let Some(line) = format_slider_event("Alpha", event) {
                event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
            }
        }),
    ]
}
