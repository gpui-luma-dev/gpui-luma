#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{
    AnyElement, AppContext, Context, Entity, FontWeight, Hsla, IntoElement, ParentElement, Rgba, Styled, Subscription,
    div, hsla, px, rgb,
};
use gpui_luma::{hstack, vstack};
use gpui_luma::controls::color::color_field::{CircleDomain, ColorFieldEvent, ColorFieldState, WhiteMixHueWheelModel};
use gpui_luma::controls::color::color_ring::{ColorRingEvent, ColorRingState, LightnessRingDelegate, ring::sizing};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::style::Size;
use gpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem};
use gpui_luma::controls::textfield::{TextField, TextFieldEvent};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_hex_color, notify_entity};

use super::common::{color_gallery_pane, notify_control};

const RING_SIZE: f32 = 300.0;
const RING_CANVAS_PADDING: f32 = 20.0;
const RING_TO_WHEEL_INNER_GAP: f32 = 12.0;
const WHEEL_SIZE: f32 = (RING_SIZE - 2.0 * sizing::RING_THICKNESS_MEDIUM - RING_TO_WHEEL_INNER_GAP).max(40.0);
const WHEEL_HALF: f32 = WHEEL_SIZE * 0.5;
const CARD_WIDTH: f32 = 520.0;
const COMPONENT_GAP_PX: f32 = 28.0;
const ROW_GAP_PX: f32 = 16.0;
const COLOR_SWATCH_SIZE: f32 = 48.0;
const RESULT_SWATCH_HEIGHT: f32 = 112.0;

#[derive(Clone)]
pub(in crate::gallery) struct ColorCombinationsPane {
    state: Entity<ColorCombinationsState>,
}

impl ColorCombinationsPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let state = cx.new(|cx| ColorCombinationsState::new(look, cx));
        Self { state }
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        color_gallery_pane(
            "Combinations",
            "The original harmony-combination composition page: a base color wheel plus generated harmony palettes.",
            self.state.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.state.update(cx, |state, cx| state.notify_controls(cx));
        notify_entity(&self.state, cx);
    }
}

struct ColorCombinationsState {
    look: Arc<ShadcnLook>,
    wheel: Entity<ColorFieldState>,
    lightness_ring: Entity<ColorRingState>,
    harmony_menu: Entity<Selector>,
    color_input: TextField,
    color_input_programmatic_update: bool,
    color: Hsla,
    selected_combination: ColorCombination,
    _subscriptions: Vec<Subscription>,
}

impl ColorCombinationsState {
    fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let color: Hsla = rgb(0x47c424).into();
        let wheel = cx.new(|_| {
            ColorFieldState::new(
                "color-combinations-wheel",
                hsla_to_wheel_hsv(color),
                Arc::new(CircleDomain),
                Arc::new(WhiteMixHueWheelModel),
            )
            .thumb_size((WHEEL_SIZE * 0.07).max(10.0))
            .inside_field()
            .raster_image_prewarmed_square(WHEEL_SIZE)
        });
        let lightness_ring = cx.new(|cx| {
            ColorRingState::lightness(
                "color-combinations-lightness-ring",
                color.l,
                LightnessRingDelegate { hue: color.h * 360.0, saturation: color.s },
                cx,
            )
            .size(Size::Size(px(RING_SIZE)))
            .ring_thickness_size(Size::Medium)
            .thumb_size(14.0)
            .rotation_degrees(180.0)
        });
        let harmony_menu = look
            .selector("color-combinations-harmony")
            .label("Combination")
            .items(color_combination_items())
            .selected_id(ColorCombination::Tetradic.id())
            .spawn(cx);
        let color_input = look
            .textfield("color-combinations-input")
            .value(format_rgb_hex(color))
            .placeholder("#RRGGBB")
            .full_width(true)
            .spawn(cx);

        let subscriptions = vec![
            cx.subscribe(&wheel, |this, _, event: &ColorFieldEvent, cx| {
                let hsv = match event {
                    ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv) => *hsv,
                };
                this.color.h = (hsv.h / 360.0).rem_euclid(1.0);
                this.color.s = hsv.s.clamp(0.0, 1.0);
                this.sync_ring(cx);
                cx.notify();
            }),
            cx.subscribe(&lightness_ring, |this, _, event: &ColorRingEvent, cx| {
                let lightness = match event {
                    ColorRingEvent::Change(value) | ColorRingEvent::Release(value) => *value,
                };
                this.color.l = lightness.clamp(0.0, 1.0);
                this.sync_wheel(cx);
                cx.notify();
            }),
            cx.subscribe(&harmony_menu, |this, _, event: &SelectorEvent, cx| {
                let SelectorEvent::Change { item_id, .. } = event;
                this.selected_combination = ColorCombination::from_id(item_id.as_ref());
                cx.notify();
            }),
            cx.subscribe(&color_input, |this, _, event: &TextFieldEvent, cx| {
                if this.color_input_programmatic_update {
                    return;
                }

                if let TextFieldEvent::Change { value } | TextFieldEvent::Submit { value } = event
                    && let Some(color) = parse_rgb_hex(value)
                {
                    this.color = color;
                    this.sync_wheel(cx);
                    this.sync_ring(cx);
                    cx.notify();
                }
            }),
        ];

        Self {
            look,
            wheel,
            lightness_ring,
            harmony_menu,
            color_input,
            color_input_programmatic_update: false,
            color,
            selected_combination: ColorCombination::Tetradic,
            _subscriptions: subscriptions,
        }
    }

    fn notify_controls(&self, cx: &mut Context<Self>) {
        notify_control(&self.wheel, cx);
        notify_control(&self.lightness_ring, cx);
        notify_control(&self.harmony_menu, cx);
        notify_control(&self.color_input, cx);
    }

    fn sync_ring(&mut self, cx: &mut Context<Self>) {
        let color = self.color;
        self.lightness_ring.update(cx, |ring, cx| {
            ring.set_value(color.l, cx);
            ring.set_delegate(Box::new(LightnessRingDelegate { hue: color.h * 360.0, saturation: color.s }), cx);
        });
        self.sync_color_input(cx);
    }

    fn sync_wheel(&mut self, cx: &mut Context<Self>) {
        let color = self.color;
        self.wheel.update(cx, |wheel, cx| {
            let hsv = hsla_to_wheel_hsv(color);
            wheel.set_hsv_components(hsv.h, hsv.s, hsv.v, cx);
        });
        self.sync_color_input(cx);
    }

    fn sync_color_input(&mut self, cx: &mut Context<Self>) {
        let value = format_rgb_hex(self.color);
        if self.color_input.read(cx).value() == value {
            return;
        }

        self.color_input_programmatic_update = true;
        self.color_input.update(cx, |input, cx| input.set_value(value, cx));
        self.color_input_programmatic_update = false;
    }
}

impl gpui::Render for ColorCombinationsState {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let color = self.color;
        let swatches = self.selected_combination.palette(color);
        let harmony_points = harmony_points_from_swatches(&swatches);
        let title_text = self.look.chrome().title_text;
        let body_text = self.look.chrome().body_text;
        let border = self.look.chrome().border;
        let look = self.look.clone();
        let lightness_ring = self.lightness_ring.clone();
        let wheel = self.wheel.clone();
        let color_input = self.color_input.clone();
        let harmony_menu = self.harmony_menu.clone();

        vstack! {
            align=center;
            look
                .card("color-combinations-card")
                .elevated(false)
                .child_render(move |_, _| {
                    render_combinations_card_body(
                        title_text,
                        body_text,
                        border,
                        lightness_ring.clone(),
                        wheel.clone(),
                        color_input.clone(),
                        harmony_menu.clone(),
                        color,
                        harmony_points.clone(),
                        swatches.clone(),
                    )
                })
                .render(window, cx)
                .w(px(CARD_WIDTH))
                .max_w_full(),
        }
        .w_full()
    }
}

fn hsla_to_wheel_hsv(color: Hsla) -> Hsv {
    Hsv {
        h: (color.h * 360.0).rem_euclid(360.0),
        s: color.s.clamp(0.0, 1.0),
        // The white-mix wheel carries HSL lightness through `v`.
        v: color.l.clamp(0.0, 1.0),
        a: color.a.clamp(0.0, 1.0),
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ColorCombination {
    Monochromatic,
    Complementary,
    Analogous,
    Triadic,
    #[default]
    Tetradic,
    Pentadic,
    Hexadic,
}

impl ColorCombination {
    fn id(self) -> &'static str {
        match self {
            Self::Monochromatic => "monochromatic",
            Self::Complementary => "complementary",
            Self::Analogous => "analogous",
            Self::Triadic => "triadic",
            Self::Tetradic => "tetradic",
            Self::Pentadic => "pentadic",
            Self::Hexadic => "hexadic",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Monochromatic => "Monochromatic",
            Self::Complementary => "Complementary",
            Self::Analogous => "Analogous",
            Self::Triadic => "Triadic",
            Self::Tetradic => "Tetradic",
            Self::Pentadic => "Pentadic",
            Self::Hexadic => "Hexadic",
        }
    }

    fn from_id(id: &str) -> Self {
        match id {
            "monochromatic" => Self::Monochromatic,
            "complementary" => Self::Complementary,
            "analogous" => Self::Analogous,
            "triadic" => Self::Triadic,
            "tetradic" => Self::Tetradic,
            "pentadic" => Self::Pentadic,
            "hexadic" => Self::Hexadic,
            _ => Self::Tetradic,
        }
    }

    fn palette(self, base: Hsla) -> Vec<CombinationSwatch> {
        match self {
            Self::Monochromatic => monochromatic_palette(base),
            Self::Complementary => complementary_palette(base),
            Self::Analogous => analogous_palette(base),
            Self::Triadic => triadic_palette(base),
            Self::Tetradic => tetradic_palette(base),
            Self::Pentadic => pentadic_palette(base),
            Self::Hexadic => hexadic_palette(base),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CombinationSwatch {
    role: &'static str,
    color: Hsla,
}

fn color_combination_items() -> Vec<SelectorItem> {
    [
        ColorCombination::Monochromatic,
        ColorCombination::Complementary,
        ColorCombination::Analogous,
        ColorCombination::Triadic,
        ColorCombination::Tetradic,
        ColorCombination::Pentadic,
        ColorCombination::Hexadic,
    ]
    .into_iter()
    .map(|combination| SelectorItem::new(combination.id()).label(combination.label()))
    .collect()
}

fn monochromatic_palette(base: Hsla) -> Vec<CombinationSwatch> {
    vec![
        CombinationSwatch { role: "Base", color: base },
        CombinationSwatch {
            role: "Tint",
            color: hsla(base.h, (base.s * 0.78).clamp(0.0, 1.0), (base.l + 0.14).clamp(0.0, 1.0), base.a),
        },
    ]
}

fn complementary_palette(base: Hsla) -> Vec<CombinationSwatch> {
    vec![
        CombinationSwatch { role: "Base", color: base },
        CombinationSwatch { role: "Complement", color: rotate_hue(base, 180.0) },
    ]
}

fn analogous_palette(base: Hsla) -> Vec<CombinationSwatch> {
    vec![
        CombinationSwatch { role: "Left", color: rotate_hue(base, -30.0) },
        CombinationSwatch { role: "Base", color: base },
        CombinationSwatch { role: "Right", color: rotate_hue(base, 30.0) },
    ]
}

fn triadic_palette(base: Hsla) -> Vec<CombinationSwatch> {
    vec![
        CombinationSwatch { role: "Base", color: base },
        CombinationSwatch { role: "Triad B", color: rotate_hue(base, 120.0) },
        CombinationSwatch { role: "Triad C", color: rotate_hue(base, 240.0) },
    ]
}

fn tetradic_palette(base: Hsla) -> Vec<CombinationSwatch> {
    vec![
        CombinationSwatch { role: "Base", color: base },
        CombinationSwatch { role: "Tetrad B", color: rotate_hue(base, 90.0) },
        CombinationSwatch { role: "Tetrad C", color: rotate_hue(base, 180.0) },
        CombinationSwatch { role: "Tetrad D", color: rotate_hue(base, 270.0) },
    ]
}

fn pentadic_palette(base: Hsla) -> Vec<CombinationSwatch> {
    vec![
        CombinationSwatch { role: "Base", color: base },
        CombinationSwatch { role: "Pentad B", color: rotate_hue(base, 72.0) },
        CombinationSwatch { role: "Pentad C", color: rotate_hue(base, 144.0) },
        CombinationSwatch { role: "Pentad D", color: rotate_hue(base, 216.0) },
        CombinationSwatch { role: "Pentad E", color: rotate_hue(base, 288.0) },
    ]
}

fn hexadic_palette(base: Hsla) -> Vec<CombinationSwatch> {
    vec![
        CombinationSwatch { role: "Base", color: base },
        CombinationSwatch { role: "Hexad B", color: rotate_hue(base, 60.0) },
        CombinationSwatch { role: "Hexad C", color: rotate_hue(base, 120.0) },
        CombinationSwatch { role: "Hexad D", color: rotate_hue(base, 180.0) },
        CombinationSwatch { role: "Hexad E", color: rotate_hue(base, 240.0) },
        CombinationSwatch { role: "Hexad F", color: rotate_hue(base, 300.0) },
    ]
}

fn rotate_hue(base: Hsla, delta_degrees: f32) -> Hsla {
    hsla((base.h + delta_degrees / 360.0).rem_euclid(1.0), base.s, base.l, base.a)
}

fn harmony_points_from_swatches(swatches: &[CombinationSwatch]) -> Vec<CombinationSwatch> {
    swatches.iter().copied().filter(|swatch| swatch.role != "Base").collect()
}

fn render_combo_wheel_layer(
    wheel: Entity<ColorFieldState>,
    harmony_points: Vec<CombinationSwatch>,
) -> impl IntoElement {
    let center = WHEEL_SIZE * 0.5;
    let marker_size = (((WHEEL_SIZE * 0.07).max(10.0)) * 0.64).max(8.0);
    let marker_half = marker_size * 0.5;
    let marker_radius_max = (center - marker_half).max(0.0);

    div()
        .size(px(WHEEL_SIZE))
        .relative()
        .child(wheel)
        .children(harmony_points.into_iter().map(|swatch| {
            let angle = (swatch.color.h * 360.0).rem_euclid(360.0).to_radians();
            let marker_radius = marker_radius_max * swatch.color.s.clamp(0.0, 1.0);
            let left = center + marker_radius * angle.cos() - marker_half;
            let top = center - marker_radius * angle.sin() - marker_half;

            div()
                .absolute()
                .left(px(left))
                .top(px(top))
                .size(px(marker_size))
                .rounded_full()
                .border_1()
                .border_color(hsla(0.0, 0.0, 0.0, 0.95))
                .bg(hsla(0.0, 0.0, 1.0, 0.96))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .size(px((marker_size - 5.0).max(3.0)))
                        .rounded_full()
                        .bg(swatch.color)
                        .border_1()
                        .border_color(hsla(0.0, 0.0, 1.0, 1.0)),
                )
                .into_any_element()
        }))
}

fn render_combinations_card_body(
    title_text: Hsla,
    body_text: Hsla,
    border: Hsla,
    lightness_ring: Entity<ColorRingState>,
    wheel: Entity<ColorFieldState>,
    color_input: TextField,
    harmony_menu: Entity<Selector>,
    color: Hsla,
    harmony_points: Vec<CombinationSwatch>,
    swatches: Vec<CombinationSwatch>,
) -> AnyElement {
    vstack! {
        gap=COMPONENT_GAP_PX align=center;
        div()
            .size(px(RING_SIZE + RING_CANVAS_PADDING))
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .child(lightness_ring)
            .child(
                div()
                    .absolute()
                    .left_1_2()
                    .top_1_2()
                    .ml(px(-WHEEL_HALF))
                    .mt(px(-WHEEL_HALF))
                    .size(px(WHEEL_SIZE))
                    .rounded_full()
                    .overflow_hidden()
                    .child(render_combo_wheel_layer(wheel, harmony_points)),
            ),
        hstack! {
            gap=ROW_GAP_PX align=center;
            field_label("Color", title_text),
            div().size(px(COLOR_SWATCH_SIZE)).rounded_full().border_1().border_color(border).bg(color),
            div().min_w(px(0.0)).flex_1().child(color_input),
        }
        .w_full(),
        hstack! {
            gap=ROW_GAP_PX align=center;
            field_label("Combination", title_text),
            div().min_w(px(0.0)).flex_1().child(harmony_menu),
        }
        .w_full(),
        div().w_full().flex().gap(px(ROW_GAP_PX)).children(swatches.into_iter().map(|swatch| {
            vstack! {
                gap=10.0;
                div().h(px(RESULT_SWATCH_HEIGHT)).rounded(px(14.0)).bg(swatch.color),
                div()
                    .text_size(px(14.0))
                    .line_height(px(20.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(body_text)
                    .child(format_rgb_hex(swatch.color)),
            }
            .flex_1()
            .min_w(px(0.0))
            .into_any_element()
        })),
    }
    .into_any_element()
}

fn field_label(label: &'static str, color: Hsla) -> gpui::Div {
    div()
        .text_size(px(18.0))
        .line_height(px(24.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color)
        .child(label)
}

fn format_rgb_hex(color: Hsla) -> String {
    format_hex_color(color).to_ascii_uppercase()
}

fn parse_rgb_hex(value: &str) -> Option<Hsla> {
    Rgba::try_from(value.trim()).ok().map(Into::into)
}
