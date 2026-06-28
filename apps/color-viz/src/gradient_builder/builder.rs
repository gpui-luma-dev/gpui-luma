use std::collections::HashMap;
use std::sync::Arc;

use gpui::{
    Bounds, ClickEvent, Context, Corner, Corners, Entity, MouseDownEvent, Pixels, Render, Subscription, Window,
    anchored, canvas, deferred, div, point, prelude::*, px,
};
use gpui_luma::controls::color::color_field::ColorFieldEvent;
use gpui_luma::controls::color::color_ring::primary_slider_value;
use gpui_luma::controls::color::style::{ElementExt, StyledExt};
use gpui_luma::controls::color::color_slider::{
    ColorSliderBuilder, ColorSliderDomainRenderer, ColorSliderTrackContext, GradientDelegate, GradientStop,
    refresh_color_slider, update_domain_delegate,
};
use gpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem};
use gpui_luma::controls::slider::{SliderControl, SliderEvent, ThumbId};
use gpui_luma::controls::textfield::{TextField, TextFieldEvent};
use gpui_luma::{form_field, grid_layout, hstack, vstack, GridTrack};
use gpui_luma::theme::{ControlSize, ThemeMode};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnLookControlExt, ShadcnTextSize};

use super::color::{format_css_linear_gradient, format_hex_color, format_percent, parse_hex_color, parse_percent};
use super::paint::{GradientType, paint_linear_gradient_preview, sorted_stops};
use super::sv_triangle_picker::SvTrianglePicker;

const SHELL_RADIUS: f32 = 12.0;
const SHELL_BORDER: f32 = 1.0;

fn shell_inner_corner_radius() -> Pixels {
    px(SHELL_RADIUS - SHELL_BORDER)
}

fn controls_panel_corner_radii() -> Corners<Pixels> {
    let inner = shell_inner_corner_radius();
    Corners { top_left: inner, top_right: px(0.0), bottom_left: inner, bottom_right: px(0.0) }
}

fn preview_panel_corner_radii() -> Corners<Pixels> {
    let inner = shell_inner_corner_radius();
    Corners { top_left: px(0.0), top_right: inner, bottom_left: px(0.0), bottom_right: inner }
}

pub struct GradientBuilder {
    look: Arc<ShadcnLook>,
    gradient_stops: Entity<SliderControl>,
    stop_colors: HashMap<ThumbId, gpui::Hsla>,
    selected_stop: Option<ThumbId>,
    domain_renderer: Arc<ColorSliderDomainRenderer>,
    track_context: ColorSliderTrackContext,
    color_field: TextField,
    position_field: TextField,
    rotation_selector: Entity<Selector>,
    type_selector: Entity<Selector>,
    rotation_deg: f32,
    gradient_type: GradientType,
    syncing_fields: bool,
    color_picker: Entity<SvTrianglePicker>,
    color_picker_open: bool,
    swatch_bounds: Option<Bounds<Pixels>>,
    _subscriptions: Vec<Subscription>,
}

impl GradientBuilder {
    pub fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let theme_is_dark = matches!(look.mode(), ThemeMode::Dark);
        let start = parse_hex_color("#864B6A").expect("seed color");
        let end = parse_hex_color("#C48770").expect("seed color");

        let slider_builder = ColorSliderBuilder::gradient("color-viz-gradient-stops", 0.22, vec![start, end])
            .dual_stop()
            .thumb_values([(0.22, Some(start)), (1.0, Some(end))])
            .size(ControlSize::Md)
            .thumb_medium()
            //.edge_to_edge()
            .rounded(px(8.0))
            .theme_is_dark(theme_is_dark);
        let domain_renderer = slider_builder.domain_renderer();
        let track_context = slider_builder.track_context();
        let gradient_stops = slider_builder.spawn(cx);

        let mut stop_colors = HashMap::new();
        for thumb in gradient_stops.read(cx).thumbs() {
            stop_colors.insert(thumb.id, thumb.preview.unwrap_or(start));
        }
        let selected_stop = gradient_stops.read(cx).active_thumb_id();

        let color_field =
            look.textfield("color-viz-gradient-color").value(format_hex_color(start)).full_width(true).spawn(cx);
        let position_field =
            look.textfield("color-viz-gradient-position").value(format_percent(0.22)).full_width(true).spawn(cx);
        let rotation_selector =
            look.selector("color-viz-gradient-rotation").items(rotation_items()).selected_id("90").spawn(cx);
        let type_selector =
            look.selector("color-viz-gradient-type").items(type_items()).selected_id("linear").spawn(cx);
        let color_picker = cx.new(|cx| SvTrianglePicker::new(start, cx));

        let mut builder = Self {
            look: look.clone(),
            gradient_stops: gradient_stops.clone(),
            stop_colors,
            selected_stop,
            domain_renderer,
            track_context,
            color_field: color_field.clone(),
            position_field: position_field.clone(),
            rotation_selector: rotation_selector.clone(),
            type_selector: type_selector.clone(),
            rotation_deg: 90.0,
            gradient_type: GradientType::Linear,
            syncing_fields: false,
            color_picker: color_picker.clone(),
            color_picker_open: false,
            swatch_bounds: None,
            _subscriptions: Vec::new(),
        };

        builder.wire_subscriptions(
            cx,
            gradient_stops,
            color_field,
            position_field,
            rotation_selector,
            type_selector,
            color_picker,
        );
        builder.sync_fields_from_selection(cx);
        builder.rebuild_stops(cx);
        builder
    }

    fn wire_subscriptions(
        &mut self,
        cx: &mut Context<Self>,
        gradient_stops: Entity<SliderControl>,
        color_field: TextField,
        position_field: TextField,
        rotation_selector: Entity<Selector>,
        type_selector: Entity<Selector>,
        color_picker: Entity<SvTrianglePicker>,
    ) {
        self._subscriptions.push(cx.subscribe(&gradient_stops, |this, _, event, cx| {
            this.handle_stops_event(event, cx);
        }));
        self._subscriptions.push(cx.subscribe(&color_field, |this, _, event, cx| {
            this.handle_color_field_event(event, cx);
        }));
        self._subscriptions.push(cx.subscribe(&position_field, |this, _, event, cx| {
            this.handle_position_field_event(event, cx);
        }));
        self._subscriptions.push(cx.subscribe(&rotation_selector, |this, _, event, cx| {
            this.handle_rotation_event(event, cx);
        }));
        self._subscriptions.push(cx.subscribe(&type_selector, |this, _, event, cx| {
            this.handle_type_event(event, cx);
        }));
        self._subscriptions.push(cx.subscribe(&color_picker.read(cx).ring(), |this, _, event, cx| {
            if primary_slider_value(event).is_some() {
                this.apply_picker_color(cx);
            }
        }));
        self._subscriptions.push(cx.subscribe(&color_picker.read(cx).triangle(), |this, _, event, cx| {
            if matches!(event, ColorFieldEvent::Change(_) | ColorFieldEvent::Release(_)) {
                this.apply_picker_color(cx);
            }
        }));
    }

    fn handle_stops_event(&mut self, event: &SliderEvent, cx: &mut Context<Self>) {
        match event {
            SliderEvent::ThumbSelected { thumb_id } => {
                self.selected_stop = Some(*thumb_id);
                self.sync_fields_from_selection(cx);
                if self.color_picker_open {
                    let color = self.selected_color(cx);
                    self.color_picker.update(cx, |picker, cx| picker.set_color(color, cx));
                }
                cx.notify();
            }
            SliderEvent::Change { .. } | SliderEvent::Release { .. } => {
                self.rebuild_stops(cx);
                if self.selected_stop.is_some() {
                    self.sync_fields_from_selection(cx);
                }
            }
            SliderEvent::ThumbAdded { .. } | SliderEvent::ThumbRemoved { .. } => {}
        }
    }

    fn handle_color_field_event(&mut self, event: &TextFieldEvent, cx: &mut Context<Self>) {
        if self.syncing_fields {
            return;
        }
        let (TextFieldEvent::Change { value } | TextFieldEvent::Submit { value }) = event else {
            return;
        };
        let Some(color) = parse_hex_color(value) else {
            return;
        };
        let Some(thumb_id) = self.selected_stop else {
            return;
        };
        self.stop_colors.insert(thumb_id, color);
        self.rebuild_stops(cx);
        cx.notify();
    }

    fn handle_position_field_event(&mut self, event: &TextFieldEvent, cx: &mut Context<Self>) {
        if self.syncing_fields {
            return;
        }
        let (TextFieldEvent::Change { value } | TextFieldEvent::Submit { value }) = event else {
            return;
        };
        let Some(position) = parse_percent(value) else {
            return;
        };
        let Some(thumb_id) = self.selected_stop else {
            return;
        };
        self.gradient_stops.update(cx, |slider, cx| {
            slider.set_thumb_position(thumb_id, position, cx);
        });
        self.rebuild_stops(cx);
        cx.notify();
    }

    fn handle_rotation_event(&mut self, event: &SelectorEvent, cx: &mut Context<Self>) {
        let SelectorEvent::Change { item_id, .. } = event;
        if let Ok(rotation) = item_id.parse::<f32>() {
            self.rotation_deg = rotation;
            cx.notify();
        }
    }

    fn handle_type_event(&mut self, event: &SelectorEvent, cx: &mut Context<Self>) {
        let SelectorEvent::Change { item_id, .. } = event;
        if item_id.as_ref() == "linear" {
            self.gradient_type = GradientType::Linear;
            cx.notify();
        }
    }

    fn apply_picker_color(&mut self, cx: &mut Context<Self>) {
        if !self.color_picker_open {
            return;
        }
        let color = self.color_picker.read(cx).color();
        let Some(thumb_id) = self.selected_stop else {
            return;
        };
        self.stop_colors.insert(thumb_id, color);
        self.syncing_fields = true;
        self.color_field.update(cx, |field, cx| field.set_value(format_hex_color(color), cx));
        self.syncing_fields = false;
        self.rebuild_stops(cx);
        cx.notify();
    }

    fn toggle_color_picker(&mut self, cx: &mut Context<Self>) {
        if self.color_picker_open {
            self.color_picker_open = false;
        } else {
            self.color_picker_open = true;
            let color = self.selected_color(cx);
            self.color_picker.update(cx, |picker, cx| picker.set_color(color, cx));
        }
        cx.notify();
    }

    fn close_color_picker(&mut self, cx: &mut Context<Self>) {
        if self.color_picker_open {
            self.color_picker_open = false;
            cx.notify();
        }
    }

    fn handle_swatch_bounds(&mut self, bounds: &Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.swatch_bounds = Some(*bounds);
    }

    fn rebuild_stops(&mut self, cx: &mut Context<Self>) {
        let slider = self.gradient_stops.read(cx);
        let stops = sorted_stops(
            &slider
                .thumbs()
                .iter()
                .map(|thumb| {
                    let color = self
                        .stop_colors
                        .get(&thumb.id)
                        .copied()
                        .or(thumb.preview)
                        .unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.5, 1.0));
                    (thumb.position, color)
                })
                .collect::<Vec<_>>(),
        );
        let stops = stops.into_iter().map(|(position, color)| GradientStop { position, color }).collect();
        let mut track_context = self.track_context.clone();
        track_context.theme_is_dark = matches!(self.look.mode(), ThemeMode::Dark);
        update_domain_delegate(&self.domain_renderer, Arc::new(GradientDelegate { stops }), track_context);
        refresh_color_slider(&self.gradient_stops, cx);
        cx.notify();
    }

    fn sync_fields_from_selection(&mut self, cx: &mut Context<Self>) {
        let Some(thumb_id) = self.selected_stop else {
            return;
        };
        let slider = self.gradient_stops.read(cx);
        let Some(thumb) = slider.thumbs().iter().find(|thumb| thumb.id == thumb_id) else {
            return;
        };
        let position = thumb.position;
        let color = self
            .stop_colors
            .get(&thumb_id)
            .copied()
            .or(thumb.preview)
            .unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.5, 1.0));

        self.syncing_fields = true;
        self.color_field.update(cx, |field, cx| field.set_value(format_hex_color(color), cx));
        self.position_field.update(cx, |field, cx| field.set_value(format_percent(position), cx));
        self.syncing_fields = false;
    }

    fn selected_color(&self, cx: &Context<Self>) -> gpui::Hsla {
        let Some(thumb_id) = self.selected_stop else {
            return gpui::hsla(0.0, 0.0, 0.5, 1.0);
        };
        self.stop_colors
            .get(&thumb_id)
            .copied()
            .or_else(|| {
                self.gradient_stops
                    .read(cx)
                    .thumbs()
                    .iter()
                    .find(|thumb| thumb.id == thumb_id)
                    .and_then(|thumb| thumb.preview)
            })
            .unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.5, 1.0))
    }

    fn preview_stops(&self, cx: &Context<Self>) -> Vec<(f32, gpui::Hsla)> {
        let slider = self.gradient_stops.read(cx);
        let stops = slider
            .thumbs()
            .iter()
            .map(|thumb| {
                let color = self
                    .stop_colors
                    .get(&thumb.id)
                    .copied()
                    .or(thumb.preview)
                    .unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.5, 1.0));
                (thumb.position, color)
            })
            .collect::<Vec<_>>();
        sorted_stops(&stops)
    }

    fn gradient_spec(&self, cx: &Context<Self>) -> String {
        format_css_linear_gradient(&self.preview_stops(cx), self.rotation_deg)
    }
}

impl Render for GradientBuilder {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let stops = self.preview_stops(cx);
        let rotation_deg = self.rotation_deg;
        let selected_color = self.selected_color(cx);
        let gradient_spec = self.gradient_spec(cx);
        let code_style = self.look.typography_scale(ShadcnTextSize::Xs);
        let gradient_stops = self.gradient_stops.clone();
        let color_field = self.color_field.clone();
        let position_field = self.position_field.clone();
        let rotation_selector = self.rotation_selector.clone();
        let type_selector = self.type_selector.clone();
        let color_picker = self.color_picker.clone();
        let color_picker_open = self.color_picker_open;
        let swatch_bounds = self.swatch_bounds;
        let card_bg = self.look.token_color("card").unwrap_or(chrome.panel_background);
        let preview_corner_radii = preview_panel_corner_radii();

        let controls = vstack! {
            gap=16;
            gradient_stops,
            grid_layout! {
                rows: 2,
                columns: [GridTrack::Star(1.0), GridTrack::Star(1.0)],
                gap: 12.0;
                [0, 0] => form_field!(
                    "Color",
                    chrome;
                    render_color_field(
                        color_field,
                        selected_color,
                        chrome,
                        cx.entity(),
                        self.look.clone(),
                        color_picker,
                        color_picker_open,
                        swatch_bounds,
                        window,
                        cx,
                    )
                ),
                [0, 1] => form_field!("Position", chrome; position_field),
                [1, 0] => form_field!("Rotation", chrome; rotation_selector),
                [1, 1] => form_field!("Type", chrome; type_selector),
            },
            div()
                .id("color-viz-gradient-spec")
                .typography_style(code_style)
                .text_color(chrome.muted_text)
                .font_family("Monaco")
                .whitespace_normal()
                .child(gradient_spec),
        }
        .id("color-viz-gradient-controls")
        .w(px(360.0))
        .flex_shrink_0()
        .h_full()
        .corner_radii(controls_panel_corner_radii())
        .overflow_hidden()
        .p(px(16.0))
        .bg(card_bg);

        let preview = div()
            .id("color-viz-gradient-preview")
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .corner_radii(preview_corner_radii)
            .overflow_hidden()
            .child({
                let stops = stops.clone();
                canvas(
                    move |_, _, _| {},
                    move |bounds, _, window, _cx| {
                        paint_linear_gradient_preview(window, bounds, &stops, rotation_deg, preview_corner_radii);
                    },
                )
                .size_full()
            });

        let shell = hstack! {
            gap=0;
            controls,
            div().w(px(1.0)).flex_shrink_0().bg(chrome.border),
            preview,
        }
        .items_stretch()
        .id("color-viz-gradient-shell")
        .w_full()
        .max_w(px(960.0))
        .h(px(360.0))
        .rounded(px(SHELL_RADIUS))
        .overflow_hidden()
        .border_1()
        .border_color(chrome.border);

        div()
            .id("color-viz-gradient-builder")
            .size_full()
            .min_h_0()
            .flex()
            .items_center()
            .justify_center()
            .p(px(32.0))
            .child(shell)
    }
}

fn render_color_field(
    color_field: TextField,
    selected_color: gpui::Hsla,
    chrome: gpui_luma::theme::LumaChrome,
    builder: Entity<GradientBuilder>,
    look: Arc<ShadcnLook>,
    color_picker: Entity<SvTrianglePicker>,
    color_picker_open: bool,
    swatch_bounds: Option<Bounds<Pixels>>,
    window: &mut Window,
    cx: &mut Context<GradientBuilder>,
) -> impl IntoElement {
    let bounds_builder = builder.clone();
    let click_builder = builder.clone();

    let mut field = div().relative().w_full().child(
        div().relative().w_full().child(color_field).child(
            div()
                .id("color-viz-gradient-swatch")
                .absolute()
                .top(px(7.0))
                .right(px(8.0))
                .size(px(20.0))
                .rounded(px(4.0))
                .bg(selected_color)
                .border_1()
                .border_color(chrome.border)
                .cursor_pointer()
                .on_click(move |event: &ClickEvent, _, cx| {
                    if event.is_keyboard() {
                        return;
                    }
                    click_builder.update(cx, |this, cx| {
                        if !this.color_picker_open {
                            this.toggle_color_picker(cx);
                        }
                    });
                })
                .on_prepaint(move |bounds, window, cx| {
                    bounds_builder.update(cx, |this, cx| {
                        this.handle_swatch_bounds(&bounds, window, cx);
                    });
                }),
        ),
    );

    if color_picker_open {
        if let Some(bounds) = swatch_bounds {
            field = field.child(
                deferred(
                    anchored()
                        .snap_to_window_with_margin(px(8.0))
                        .anchor(Corner::TopLeft)
                        .position(point(bounds.left(), bounds.bottom()))
                        .offset(point(px(0.0), px(4.0)))
                        .child(
                            look.card("color-viz-color-picker-popup")
                                .elevated(true)
                                .child(color_picker)
                                .render(window, cx)
                                .occlude()
                                .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                    this.close_color_picker(cx);
                                })),
                        ),
                )
                .with_priority(1),
            );
        }
    }

    field
}

fn rotation_items() -> Vec<SelectorItem> {
    vec![
        SelectorItem::new("0").label("0°"),
        SelectorItem::new("90").label("90°"),
        SelectorItem::new("180").label("180°"),
        SelectorItem::new("270").label("270°"),
    ]
}

fn type_items() -> Vec<SelectorItem> {
    vec![SelectorItem::new("linear").label("Linear")]
}
