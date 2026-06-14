use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Subscription, div, prelude::*, px};
use gpui_luma::controls::color::color_slider::color_spec::{Hsl, RgbaSpec};
use gpui_luma::controls::color::color_slider::{
    AlphaDelegate, ChannelDelegate, ColorInterpolation, ColorSliderState, ThumbShape,
};
use gpui_luma::controls::color::style::{Size, Sizable};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::notify_entity;

use super::common::{color_gallery_pane, control_label, demo_card, demo_section};

const CARD_WIDTH: f32 = 360.0;

#[derive(Clone)]
pub(in crate::gallery) struct ColorSliderRevealedPane {
    size_xsmall: Entity<ColorSliderState>,
    size_small: Entity<ColorSliderState>,
    size_medium: Entity<ColorSliderState>,
    size_large: Entity<ColorSliderState>,
    rounded_full: Entity<ColorSliderState>,
    rounded_8: Entity<ColorSliderState>,
    rounded_square: Entity<ColorSliderState>,
    thumb_default: Entity<ColorSliderState>,
    thumb_square: Entity<ColorSliderState>,
    thumb_bar: Entity<ColorSliderState>,
    edge_default: Entity<ColorSliderState>,
    edge_square: Entity<ColorSliderState>,
    edge_large: Entity<ColorSliderState>,
    interp_rgb: Entity<ColorSliderState>,
    interp_hsl: Entity<ColorSliderState>,
    interp_lab: Entity<ColorSliderState>,
    delegate_hue: Entity<ColorSliderState>,
    delegate_alpha: Entity<ColorSliderState>,
    delegate_red: Entity<ColorSliderState>,
    delegate_saturation: Entity<ColorSliderState>,
}

impl ColorSliderRevealedPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, _look: Arc<ShadcnLook>) -> Self {
        let size_xsmall =
            cx.new(|cx| ColorSliderState::hue("color-slider-revealed-size-xsmall", 180.0, cx).with_size(Size::XSmall));
        let size_small =
            cx.new(|cx| ColorSliderState::hue("color-slider-revealed-size-small", 180.0, cx).with_size(Size::Small));
        let size_medium = cx.new(|cx| ColorSliderState::hue("color-slider-revealed-size-medium", 180.0, cx));
        let size_large =
            cx.new(|cx| ColorSliderState::hue("color-slider-revealed-size-large", 180.0, cx).with_size(Size::Large));

        let rounded_full = cx.new(|cx| ColorSliderState::hue("color-slider-revealed-rounded-full", 180.0, cx));
        let rounded_8 =
            cx.new(|cx| ColorSliderState::hue("color-slider-revealed-rounded-8", 180.0, cx).rounded(px(8.0)));
        let rounded_square =
            cx.new(|cx| ColorSliderState::hue("color-slider-revealed-rounded-square", 180.0, cx).rounded(px(0.0)));

        let thumb_default = cx.new(|cx| ColorSliderState::hue("color-slider-revealed-thumb-default", 180.0, cx));
        let thumb_square =
            cx.new(|cx| ColorSliderState::hue("color-slider-revealed-thumb-square", 180.0, cx).thumb_square());
        let thumb_bar = cx.new(|cx| {
            let mut slider = ColorSliderState::hue("color-slider-revealed-thumb-bar", 180.0, cx);
            slider.thumb.shape = ThumbShape::Bar;
            slider
        });

        let edge_default =
            cx.new(|cx| ColorSliderState::hue("color-slider-revealed-edge-default", 180.0, cx).edge_to_edge());
        let edge_square = cx.new(|cx| {
            ColorSliderState::hue("color-slider-revealed-edge-square", 180.0, cx)
                .rounded(px(0.0))
                .thumb_square()
                .edge_to_edge()
        });
        let edge_large = cx.new(|cx| {
            ColorSliderState::hue("color-slider-revealed-edge-large", 180.0, cx)
                .with_size(Size::Large)
                .edge_to_edge()
        });

        let red = gpui::hsla(0.0, 1.0, 0.5, 1.0);
        let blue = gpui::hsla(240.0 / 360.0, 1.0, 0.5, 1.0);
        let interp_rgb = cx.new(|cx| {
            ColorSliderState::gradient("color-slider-revealed-interp-rgb", 0.5, vec![red, blue], cx)
                .interpolation(ColorInterpolation::Rgb)
        });
        let interp_hsl = cx.new(|cx| {
            ColorSliderState::gradient("color-slider-revealed-interp-hsl", 0.5, vec![red, blue], cx)
                .interpolation(ColorInterpolation::Hsl)
        });
        let interp_lab = cx.new(|cx| {
            ColorSliderState::gradient("color-slider-revealed-interp-lab", 0.5, vec![red, blue], cx)
                .interpolation(ColorInterpolation::Lab)
        });

        let delegate_hue =
            cx.new(|cx| ColorSliderState::hue("color-slider-revealed-delegate-hue", 180.0, cx).thumb_medium());
        let delegate_alpha = cx.new(|cx| {
            let hsl = Hsl { h: 200.0, s: 0.8, l: 0.5, a: 1.0 };
            ColorSliderState::alpha("color-slider-revealed-delegate-alpha", 0.5, AlphaDelegate { spec: hsl }, cx)
        });
        let delegate_red = cx.new(|cx| {
            let rgba = RgbaSpec { r: 255.0, g: 128.0, b: 0.0, a: 1.0 };
            ColorSliderState::channel(
                "color-slider-revealed-delegate-red",
                rgba.r,
                ChannelDelegate::new(rgba, RgbaSpec::RED.into()).expect("RGBA red delegate should be valid"),
                cx,
            )
            .min(0.0)
            .max(255.0)
        });
        let delegate_saturation = cx.new(|cx| {
            let hsl = Hsl { h: 240.0, s: 0.5, l: 0.52, a: 1.0 };
            ColorSliderState::channel(
                "color-slider-revealed-delegate-saturation",
                hsl.s,
                ChannelDelegate::new(hsl, Hsl::SATURATION.into()).expect("HSL saturation delegate should be valid"),
                cx,
            )
        });

        Self {
            size_xsmall,
            size_small,
            size_medium,
            size_large,
            rounded_full,
            rounded_8,
            rounded_square,
            thumb_default,
            thumb_square,
            thumb_bar,
            edge_default,
            edge_square,
            edge_large,
            interp_rgb,
            interp_hsl,
            interp_lab,
            delegate_hue,
            delegate_alpha,
            delegate_red,
            delegate_saturation,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        color_gallery_pane(
            "Color Slider Revealed",
            "A wider styling survey for the color-slider primitive, matching the role of the original upstream page.",
            div()
                .w_full()
                .max_w(px(1120.0))
                .flex()
                .flex_col()
                .gap(px(28.0))
                .child(demo_section(
                    "Scale",
                    "Track sizes and default density mappings.",
                    vec![demo_card(
                        "Sizes",
                        "XSmall through large.",
                        CARD_WIDTH,
                        slider_stack(
                            [
                                ("XSmall", self.size_xsmall.clone()),
                                ("Small", self.size_small.clone()),
                                ("Medium", self.size_medium.clone()),
                                ("Large", self.size_large.clone()),
                            ],
                            look,
                        ),
                        look,
                    )],
                    look,
                ))
                .child(demo_section(
                    "Corners and Thumb Shapes",
                    "Surface radius and thumb-shape variations.",
                    vec![
                        demo_card(
                            "Corner Radius",
                            "Default rounded, softened, and square corners.",
                            CARD_WIDTH,
                            slider_stack(
                                [
                                    ("Rounded", self.rounded_full.clone()),
                                    ("Rounded 8", self.rounded_8.clone()),
                                    ("Square", self.rounded_square.clone()),
                                ],
                                look,
                            ),
                            look,
                        ),
                        demo_card(
                            "Thumb Shapes",
                            "Default, square, and bar thumb treatments.",
                            CARD_WIDTH,
                            slider_stack(
                                [
                                    ("Default", self.thumb_default.clone()),
                                    ("Square", self.thumb_square.clone()),
                                    ("Bar", self.thumb_bar.clone()),
                                ],
                                look,
                            ),
                            look,
                        ),
                    ],
                    look,
                ))
                .child(demo_section(
                    "Edge Cases",
                    "Edge-to-edge and interpolation variants.",
                    vec![
                        demo_card(
                            "Edge To Edge",
                            "Track ends aligned to the thumb centerline.",
                            CARD_WIDTH,
                            slider_stack(
                                [
                                    ("Default", self.edge_default.clone()),
                                    ("Square", self.edge_square.clone()),
                                    ("Large", self.edge_large.clone()),
                                ],
                                look,
                            ),
                            look,
                        ),
                        demo_card(
                            "Interpolation",
                            "RGB, HSL, and Lab interpolation over the same two-color gradient.",
                            CARD_WIDTH,
                            slider_stack(
                                [
                                    ("RGB", self.interp_rgb.clone()),
                                    ("HSL", self.interp_hsl.clone()),
                                    ("Lab", self.interp_lab.clone()),
                                ],
                                look,
                            ),
                            look,
                        ),
                    ],
                    look,
                ))
                .child(demo_section(
                    "Delegate Samples",
                    "Representative delegate families from the original page.",
                    vec![demo_card(
                        "Delegates",
                        "Hue, alpha, red-channel, and HSL saturation delegates.",
                        CARD_WIDTH,
                        slider_stack(
                            [
                                ("Hue", self.delegate_hue.clone()),
                                ("Alpha", self.delegate_alpha.clone()),
                                ("Red", self.delegate_red.clone()),
                                ("Saturation", self.delegate_saturation.clone()),
                            ],
                            look,
                        ),
                        look,
                    )],
                    look,
                ))
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        for entity in [
            &self.size_xsmall,
            &self.size_small,
            &self.size_medium,
            &self.size_large,
            &self.rounded_full,
            &self.rounded_8,
            &self.rounded_square,
            &self.thumb_default,
            &self.thumb_square,
            &self.thumb_bar,
            &self.edge_default,
            &self.edge_square,
            &self.edge_large,
            &self.interp_rgb,
            &self.interp_hsl,
            &self.interp_lab,
            &self.delegate_hue,
            &self.delegate_alpha,
            &self.delegate_red,
            &self.delegate_saturation,
        ] {
            notify_entity(entity, cx);
        }
    }
}

fn slider_stack<const N: usize>(rows: [(&'static str, Entity<ColorSliderState>); N], look: &ShadcnLook) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .children(rows.into_iter().map(|(label, slider)| {
            div()
                .w_full()
                .flex()
                .items_center()
                .gap(px(12.0))
                .child(div().w(px(74.0)).child(control_label(label, look)))
                .child(div().flex_1().child(slider))
                .into_any_element()
        }))
        .into_any_element()
}
