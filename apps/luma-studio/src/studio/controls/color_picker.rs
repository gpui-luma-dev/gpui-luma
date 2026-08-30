use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, Context, Corners, Div, Edges, Entity, EventEmitter, Hsla, IntoElement, PaintQuad, Pixels,
    Render, Stateful, Subscription, Window, canvas, div, point, prelude::*, px, size, transparent_black,
};
use gpui_luma::controls::anchored_panel::{AnchoredPanel, AnchoredPanelDismissPolicy, AnchoredPanelPlacement};
use gpui_luma::controls::color::chrome_tokens::swatch_checkerboard_colors;
use gpui_luma::controls::command::button::{Button, ButtonEvent, ButtonRenderModel, ButtonTemplate};
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::color_slider::{
    AlphaDelegate, ColorSliderBuilder, ColorSliderDomainRenderer, primary_slider_value, sizing,
};
use gpui_luma::controls::color::composition::ColorCompositionSync;
use gpui_luma::controls::color::style::{ActiveTheme, ElementExt};
use gpui_luma::controls::slider::{SliderControl, SliderEvent};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;

pub(crate) struct ColorSwatchButtonTemplate {
    pub size: f32,
    pub radius: f32,
}

const CHECKERBOARD_SQUARE_SIZE: Pixels = px(8.0);

fn paint_swatch_checkerboard(window: &mut Window, bounds: Bounds<Pixels>, radius: Pixels, color: Hsla) {
    let rows = (bounds.size.height / CHECKERBOARD_SQUARE_SIZE).ceil() as i32;
    let cols = (bounds.size.width / CHECKERBOARD_SQUARE_SIZE).ceil() as i32;
    for row in 0..rows {
        for col in 0..cols {
            if (row + col) % 2 == 0 {
                continue;
            }
            let origin =
                bounds.origin + point(CHECKERBOARD_SQUARE_SIZE * col as f32, CHECKERBOARD_SQUARE_SIZE * row as f32);
            let square = Bounds {
                origin,
                size: size(
                    CHECKERBOARD_SQUARE_SIZE.min(bounds.size.width - CHECKERBOARD_SQUARE_SIZE * col as f32),
                    CHECKERBOARD_SQUARE_SIZE.min(bounds.size.height - CHECKERBOARD_SQUARE_SIZE * row as f32),
                ),
            };
            if !square_outside_rounded_rect(square, bounds, radius) {
                window.paint_quad(PaintQuad {
                    bounds: square,
                    corner_radii: Corners::default(),
                    background: color.into(),
                    border_widths: Edges::default(),
                    border_color: transparent_black(),
                    border_style: gpui::BorderStyle::default(),
                });
            }
        }
    }
}

fn square_outside_rounded_rect(square: Bounds<Pixels>, rect: Bounds<Pixels>, radius: Pixels) -> bool {
    let radius = radius.as_f32();
    if radius <= 0.0 {
        return false;
    }
    let left = rect.origin.x.as_f32();
    let top = rect.origin.y.as_f32();
    let right = (rect.origin.x + rect.size.width).as_f32();
    let bottom = (rect.origin.y + rect.size.height).as_f32();
    let points = [
        (square.origin.x.as_f32(), square.origin.y.as_f32()),
        ((square.origin.x + square.size.width).as_f32(), square.origin.y.as_f32()),
        (square.origin.x.as_f32(), (square.origin.y + square.size.height).as_f32()),
        ((square.origin.x + square.size.width).as_f32(), (square.origin.y + square.size.height).as_f32()),
    ];
    let corners = [
        (left + radius, top + radius, 0_u8),
        (right - radius, top + radius, 1_u8),
        (left + radius, bottom - radius, 2_u8),
        (right - radius, bottom - radius, 3_u8),
    ];
    corners.iter().any(|(cx, cy, corner)| {
        points.iter().any(|(x, y)| {
            let in_corner = match corner {
                0 => *x < left + radius && *y < top + radius,
                1 => *x > right - radius && *y < top + radius,
                2 => *x < left + radius && *y > bottom - radius,
                _ => *x > right - radius && *y > bottom - radius,
            };
            in_corner && (x - cx).powi(2) + (y - cy).powi(2) > radius.powi(2)
        })
    })
}

impl ButtonTemplate<Hsla> for ColorSwatchButtonTemplate {
    fn render(&self, model: &ButtonRenderModel<Hsla>, _window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let size = px(self.size);
        let radius = px(self.radius);
        let color = model.data;
        let is_dark = cx.theme().is_dark();
        let border_color = cx.theme().border;
        let is_hovered = model.state.hovered && !model.state.disabled;
        let is_pressed = model.state.pressed && !model.state.disabled;
        let is_focused = model.state.focused && !model.state.disabled;

        let mut root = div().id(model.id.clone()).size(size).relative().cursor_pointer().child(
            canvas(
                move |_, _, _| (),
                move |bounds, _, window, _| {
                    let border_width = px(1.0);
                    let inner_bounds = bounds.inset(border_width);
                    let inner_radius = (radius - border_width).max(px(0.0));

                    if color.a < 0.999 && inner_bounds.size.width.as_f32() > 0.0 {
                        let (checker_light, checker_dark) = swatch_checkerboard_colors(is_dark);
                        window.paint_quad(PaintQuad {
                            bounds: inner_bounds,
                            corner_radii: Corners::all(inner_radius),
                            background: checker_light.into(),
                            border_widths: Edges::default(),
                            border_color: transparent_black(),
                            border_style: gpui::BorderStyle::default(),
                        });
                        paint_swatch_checkerboard(window, inner_bounds, inner_radius, checker_dark);
                    }

                    window.paint_quad(PaintQuad {
                        bounds,
                        corner_radii: Corners::all(radius),
                        background: color.into(),
                        border_widths: Edges::all(border_width),
                        border_color,
                        border_style: gpui::BorderStyle::default(),
                    });
                },
            )
            .absolute()
            .size_full(),
        );

        if is_pressed {
            root = root.opacity(0.75);
        } else if is_hovered {
            root = root.opacity(0.90);
        }
        if is_focused {
            root = root.border_2().border_color(border_color).rounded(radius);
        }
        root
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ColorPickerEvent {
    Change(Hsla),
}

pub(crate) struct ColorPickerPopover {
    look: Arc<ShadcnLook>,
    color: Hsla,
    hsv: Hsv,
    sync: ColorCompositionSync,
    field: Entity<ColorFieldState>,
    hue_slider: Entity<SliderControl>,
    alpha_slider: Entity<SliderControl>,
    alpha_domain: Arc<ColorSliderDomainRenderer>,
    panel: Entity<AnchoredPanel>,
    button: Entity<Button<Hsla>>,
    swatch_size: f32,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ColorPickerEvent> for ColorPickerPopover {}

impl ColorPickerPopover {
    pub(crate) fn new(
        look: Arc<ShadcnLook>,
        color: Hsla,
        instance_id: impl Into<String>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new_with_size(look, color, instance_id.into(), 30.0, cx)
    }

    fn new_with_size(
        look: Arc<ShadcnLook>,
        color: Hsla,
        instance_id: String,
        swatch_size: f32,
        cx: &mut Context<Self>,
    ) -> Self {
        let hsv = Hsv::from_hsla_ext(color);
        let field = cx.new(|_| {
            ColorFieldState::saturation_value(format!("{instance_id}-field"), hsv, sizing::THUMB_SIZE_MEDIUM)
                .vector()
                .no_border()
                .rounded(px(0.0))
        });
        let hue_slider = ColorSliderBuilder::hue(format!("{instance_id}-hue"), hsv.h)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge()
            .spawn(cx);
        let alpha_builder = ColorSliderBuilder::alpha(format!("{instance_id}-alpha"), hsv.a, hsv)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge();
        let alpha_domain = alpha_builder.domain_renderer();
        let alpha_slider = alpha_builder.spawn(cx);

        let entity = cx.entity().clone();
        let panel = AnchoredPanel::new(format!("{instance_id}-popup"))
            .content(move |_, _, cx| entity.update(cx, |picker, _| picker.render_popup()))
            .placement(AnchoredPanelPlacement::SmartStart)
            .dismiss_policy(AnchoredPanelDismissPolicy::CloseOnClickAway)
            .spawn(cx);

        let button = Button::new(format!("{instance_id}-trigger"))
            .typed(color)
            .template(Arc::new(ColorSwatchButtonTemplate { size: swatch_size, radius: 4.0 }))
            .spawn(cx);
        let panel_for_button = panel.clone();

        let subscriptions = vec![
            cx.subscribe(&button, move |_, _, event: &ButtonEvent, cx| {
                if matches!(event, ButtonEvent::Click) {
                    panel_for_button.update(cx, |panel, cx| panel.toggle_guarded_from(None, cx));
                }
            }),
            cx.subscribe(&field, |picker, _, event: &ColorFieldEvent, cx| {
                let (ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv)) = event else {
                    return;
                };
                picker.hsv.a = picker.color.a;
                picker.hsv.s = hsv.s;
                picker.hsv.v = hsv.v;
                picker.sync_controls(cx, false);
                picker.emit_change(cx);
            }),
            cx.subscribe(&hue_slider, |picker, _, event: &SliderEvent, cx| {
                let Some(value) = primary_slider_value(event) else {
                    return;
                };
                picker.hsv.h = value;
                picker.sync_controls(cx, true);
                picker.emit_change(cx);
            }),
            cx.subscribe(&alpha_slider, |picker, _, event: &SliderEvent, cx| {
                let Some(value) = primary_slider_value(event) else {
                    return;
                };
                picker.hsv.a = value;
                picker.sync_controls(cx, true);
                picker.emit_change(cx);
            }),
        ];

        Self {
            look,
            color,
            hsv,
            sync: ColorCompositionSync::new(),
            field,
            hue_slider,
            alpha_slider,
            alpha_domain,
            panel,
            button,
            swatch_size,
            _subscriptions: subscriptions,
        }
    }

    pub(crate) fn new_palette(
        look: Arc<ShadcnLook>,
        color: Hsla,
        instance_id: impl Into<String>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new_with_size(look, color, instance_id.into(), 55.0, cx)
    }

    pub(crate) fn set_color(&mut self, color: Hsla, cx: &mut Context<Self>) {
        self.color = color;
        self.hsv = Hsv::from_hsla_ext(color);
        self.button.update(cx, |button, cx| button.set_data(color, cx));
        self.sync_controls(cx, true);
        cx.notify();
    }

    fn sync_controls(&self, cx: &mut Context<Self>, sync_field: bool) {
        if sync_field {
            self.field.update(cx, |field, cx| field.set_hsv(self.hsv, cx));
        }
        self.sync.sync_slider_value(&self.hue_slider, self.hsv.h, cx);
        self.sync.sync_color_slider(
            &self.alpha_slider,
            &self.alpha_domain,
            Arc::new(AlphaDelegate { spec: self.hsv }),
            self.alpha_domain.context(),
            self.hsv.a,
            cx,
        );
    }

    fn emit_change(&mut self, cx: &mut Context<Self>) {
        self.color = self.hsv.to_hsla_ext();
        cx.emit(ColorPickerEvent::Change(self.color));
        cx.notify();
    }

    fn render_popup(&self) -> AnyElement {
        div()
            .w(px(260.0))
            .p(px(12.0))
            .gap(px(10.0))
            .flex()
            .flex_col()
            .bg(self.look.chrome().content_background)
            .border_1()
            .border_color(self.look.chrome().border)
            .rounded(px(6.0))
            .child(div().w(px(236.0)).h(px(180.0)).child(self.field.clone()))
            .child(div().w(px(236.0)).child(self.hue_slider.clone()))
            .child(div().w(px(236.0)).child(self.alpha_slider.clone()))
            .into_any_element()
    }
}

impl Render for ColorPickerPopover {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let button = self.button.clone();
        let panel_for_bounds = self.panel.clone();
        let swatch_size = self.swatch_size;
        let trigger = div().size(px(swatch_size)).child(button).on_prepaint(move |bounds, _, cx| {
            panel_for_bounds.update(cx, |panel, cx| panel.set_anchor_bounds(bounds, cx));
        });
        div().size(px(swatch_size)).child(trigger).child(self.panel.clone())
    }
}
