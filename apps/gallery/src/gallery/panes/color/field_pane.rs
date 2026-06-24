use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma::controls::color::color_slider::{color_spec::Hsv, sizing};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, notify_entity};

use super::common::{color_gallery_pane, demo_card, demo_section, detail_row};

const FIELD_SIZE: f32 = 220.0;

#[derive(Clone)]
pub(in crate::gallery) struct ColorFieldPane {
    rect_field: Entity<ColorFieldState>,
    raster_field: Entity<ColorFieldState>,
    triangle_field: Entity<ColorFieldState>,
    wheel_field: Entity<ColorFieldState>,
    hue_value_field: Entity<ColorFieldState>,
    hsv: Hsv,
}

impl ColorFieldPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, _look: Arc<ShadcnLook>) -> Self {
        let hsv = Hsv { h: 320.0, s: 0.72, v: 0.84, a: 1.0 };

        let rect_field = cx.new(|_| {
            ColorFieldState::saturation_value_rect("color-field-rect", hsv, sizing::THUMB_SIZE_MEDIUM)
                .vector()
                .rounded(px(12.0))
        });
        let raster_field = cx.new(|_| {
            ColorFieldState::saturation_value_rect("color-field-raster", hsv, sizing::THUMB_SIZE_MEDIUM)
                .rounded(px(12.0))
                .raster_image_prewarmed_square(FIELD_SIZE)
        });
        let triangle_field = cx.new(|_| {
            ColorFieldState::saturation_value_triangle("color-field-triangle", hsv, sizing::THUMB_SIZE_MEDIUM).vector()
        });
        let wheel_field = cx.new(|_| {
            ColorFieldState::hue_saturation_wheel("color-field-wheel", hsv, sizing::THUMB_SIZE_MEDIUM)
                //.vector() TODO add sample to this file for vector vs raster
                .inside_field()
        });
        let hue_value_field = cx.new(|_| {
            ColorFieldState::hue_value("color-field-hue-value", hsv, sizing::THUMB_SIZE_MEDIUM)
                .rounded(px(12.0))
                .vector()
        });

        Self { rect_field, raster_field, triangle_field, wheel_field, hue_value_field, hsv }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.rect_field, |app, _, event: &ColorFieldEvent, cx| {
            app.panes.color_field.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let selected = self.hsv.to_hsla_ext();

        color_gallery_pane(
            "Color Field",
            "Original Opal field demos split out as a dedicated gallery page: rectangular planes, alternate domains, and vector or raster rendering.",
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(28.0))
                .child(demo_section(
                    "Planes",
                    "Representative field models from the original gallery inventory.",
                    vec![
                        demo_card(
                            "Saturation / Value",
                            "Primary rectangular HSV field.",
                            320.0,
                            div().size(px(FIELD_SIZE)).child(self.rect_field.clone()),
                            look,
                        ),
                        demo_card(
                            "Hue / Value",
                            "Alternate rectangular domain at fixed saturation.",
                            320.0,
                            div().size(px(FIELD_SIZE)).child(self.hue_value_field.clone()),
                            look,
                        ),
                        demo_card(
                            "Triangle Domain",
                            "Triangle selection surface using the same color model.",
                            320.0,
                            div().size(px(FIELD_SIZE)).child(self.triangle_field.clone()),
                            look,
                        ),
                        demo_card(
                            "Hue / Saturation Wheel",
                            "Circular field variant from the upstream page set.",
                            320.0,
                            div().size(px(FIELD_SIZE)).child(self.wheel_field.clone()),
                            look,
                        ),
                    ],
                    look,
                ))
                .child(demo_section(
                    "Renderer Compare",
                    "The upstream page contrasts vector and raster surfaces. This keeps the interactive vector field next to a prewarmed raster image field.",
                    vec![
                        demo_card(
                            "Raster Image",
                            "Raster-backed field using the same HSV model.",
                            320.0,
                            div().size(px(FIELD_SIZE)).child(self.raster_field.clone()),
                            look,
                        ),
                        demo_card(
                            "Selection Readout",
                            "Readout from the primary saturation/value field.",
                            320.0,
                            div()
                                .w_full()
                                .flex()
                                .flex_col()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .h(px(44.0))
                                        .rounded(px(12.0))
                                        .border_1()
                                        .border_color(look.chrome().border)
                                        .bg(selected),
                                )
                                .child(detail_row("Hex", format_hex_color(selected), look))
                                .child(detail_row("HSLA", format_compact_hsla(selected), look))
                                .child(detail_row("Hue", format!("{:.1} deg", self.hsv.h), look))
                                .child(detail_row("Saturation", format!("{:.3}", self.hsv.s), look))
                                .child(detail_row("Value", format!("{:.3}", self.hsv.v), look)),
                            look,
                        ),
                    ],
                    look,
                ))
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.rect_field, cx);
        notify_entity(&self.raster_field, cx);
        notify_entity(&self.triangle_field, cx);
        notify_entity(&self.wheel_field, cx);
        notify_entity(&self.hue_value_field, cx);
    }

    fn handle_event(&mut self, event: &ColorFieldEvent, cx: &mut Context<GalleryApp>) {
        self.hsv = match event {
            ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv) => *hsv,
        };
        cx.notify();
    }
}
