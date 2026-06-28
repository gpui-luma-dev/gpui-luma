use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use gpui::{
    Bounds, ClickEvent, Context, Corner, Corners, Entity, ImageSource, MouseDownEvent, Pixels, Render, RenderImage,
    SharedString, Size, Subscription, Window, anchored, canvas, deferred, div, img, point, prelude::*, px, size,
};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::color::color_field::ColorFieldEvent;
use gpui_luma::controls::color::color_ring::primary_slider_value;
use gpui_luma::controls::color::style::{ElementExt, StyledExt};
use gpui_luma::controls::color::color_slider::{
    ColorSliderBuilder, ColorSliderDomainRenderer, ColorSliderTrackContext, GradientDelegate, GradientStop,
    refresh_color_slider, update_domain_delegate,
};
use gpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem};
use gpui_luma::controls::slider::{SliderControl, SliderEvent, SliderThumbPolicy, ThumbId};
use gpui_luma::{form_field, hstack, vstack};
use gpui_luma::theme::{ControlSize, LumaTextStyle, ThemeMode};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnLookControlExt, ShadcnTextSize};
use lucide_icons::Icon as LucideIcon;

use super::color::{format_css_linear_gradient, format_hex_color, format_percent};
use super::paint::{
    GradientType, PreviewRenderer, color_at_position, paint_linear_gradient_preview, rasterize_linear_gradient_preview,
    sorted_stops,
};
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
    add_stop_button: Entity<Button>,
    stop_delete_buttons: HashMap<ThumbId, Entity<Button>>,
    rotation_slider: Entity<SliderControl>,
    type_selector: Entity<Selector>,
    renderer_selector: Entity<Selector>,
    rotation_deg: f32,
    gradient_type: GradientType,
    preview_renderer: PreviewRenderer,
    preview_size: Size<Pixels>,
    preview_image_cache: Option<(PreviewImageCacheKey, Arc<RenderImage>)>,
    active_preview_strategy: &'static str,
    last_render_ms: Option<f32>,
    color_picker: Entity<SvTrianglePicker>,
    color_picker_open: bool,
    stop_swatch_bounds: HashMap<ThumbId, Bounds<Pixels>>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PreviewStopKey {
    position_millis: u16,
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PreviewImageCacheKey {
    width_px: u16,
    height_px: u16,
    rotation_tenths: u16,
    stops: Vec<PreviewStopKey>,
}

impl GradientBuilder {
    pub fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let theme_is_dark = matches!(look.mode(), ThemeMode::Dark);
        let start = gpui::hsla(198.0 / 360.0, 1.0, 0.24, 1.0);
        let middle = gpui::hsla(313.0 / 360.0, 0.48, 0.45, 1.0);
        let end = gpui::hsla(33.0 / 360.0, 1.0, 0.46, 1.0);

        let slider_builder = ColorSliderBuilder::gradient("color-viz-gradient-stops", 0.0, vec![start, middle, end])
            .thumb_policy(SliderThumbPolicy {
                min_count: 2,
                max_count: 8,
                min_distance: 0.02,
                allow_insert: true,
                allow_remove: true,
                allow_overlap: false,
            })
            .thumb_values([(0.0, Some(start)), (0.5, Some(middle)), (1.0, Some(end))])
            .size(ControlSize::Sm)
            .thumb_medium()
            .theme_is_dark(theme_is_dark);
        let domain_renderer = slider_builder.domain_renderer();
        let track_context = slider_builder.track_context();
        let gradient_stops = slider_builder.spawn(cx);

        let mut stop_colors = HashMap::new();
        for thumb in gradient_stops.read(cx).thumbs() {
            stop_colors.insert(thumb.id, thumb.preview.unwrap_or(start));
        }
        let selected_stop = gradient_stops.read(cx).active_thumb_id();
        let add_stop_button = look.outline_icon_button("color-viz-gradient-add-stop", LucideIcon::Plus).spawn(cx);

        let rotation_slider =
            look.slider("color-viz-gradient-rotation").range(0.0..360.0).step(1.0).value(90.0).spawn(cx);
        let type_selector =
            look.selector("color-viz-gradient-type").items(type_items()).selected_id("linear").spawn(cx);
        let renderer_selector =
            look.selector("color-viz-gradient-renderer").items(renderer_items()).selected_id("quads").spawn(cx);
        let color_picker = cx.new(|cx| SvTrianglePicker::new(start, cx));

        let mut builder = Self {
            look: look.clone(),
            gradient_stops: gradient_stops.clone(),
            stop_colors,
            selected_stop,
            domain_renderer,
            track_context,
            add_stop_button: add_stop_button.clone(),
            stop_delete_buttons: HashMap::new(),
            rotation_slider: rotation_slider.clone(),
            type_selector: type_selector.clone(),
            renderer_selector: renderer_selector.clone(),
            rotation_deg: 90.0,
            gradient_type: GradientType::Linear,
            preview_renderer: PreviewRenderer::Quads,
            preview_size: size(px(0.0), px(0.0)),
            preview_image_cache: None,
            active_preview_strategy: "quads",
            last_render_ms: None,
            color_picker: color_picker.clone(),
            color_picker_open: false,
            stop_swatch_bounds: HashMap::new(),
            _subscriptions: Vec::new(),
        };

        builder.wire_subscriptions(
            cx,
            gradient_stops,
            add_stop_button,
            rotation_slider,
            type_selector,
            renderer_selector,
            color_picker,
        );
        builder.sync_stop_buttons(cx);
        builder.rebuild_stops(cx);
        builder
    }

    fn wire_subscriptions(
        &mut self,
        cx: &mut Context<Self>,
        gradient_stops: Entity<SliderControl>,
        add_stop_button: Entity<Button>,
        rotation_slider: Entity<SliderControl>,
        type_selector: Entity<Selector>,
        renderer_selector: Entity<Selector>,
        color_picker: Entity<SvTrianglePicker>,
    ) {
        self._subscriptions.push(cx.subscribe(&gradient_stops, |this, _, event, cx| {
            this.handle_stops_event(event, cx);
        }));
        self._subscriptions.push(cx.subscribe(&add_stop_button, |this, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                this.handle_add_stop(cx);
            }
        }));
        self._subscriptions.push(cx.subscribe(&rotation_slider, |this, _, event, cx| {
            this.handle_rotation_event(event, cx);
        }));
        self._subscriptions.push(cx.subscribe(&type_selector, |this, _, event, cx| {
            this.handle_type_event(event, cx);
        }));
        self._subscriptions.push(cx.subscribe(&renderer_selector, |this, _, event, cx| {
            this.handle_renderer_event(event, cx);
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
            SliderEvent::ThumbAdded { thumb_id, .. } => {
                self.hydrate_thumb_color(*thumb_id, cx);
                self.selected_stop = Some(*thumb_id);
                self.sync_stop_buttons(cx);
                self.rebuild_stops(cx);
            }
            SliderEvent::ThumbRemoved { thumb_id } => {
                self.stop_colors.remove(thumb_id);
                self.sync_stop_buttons(cx);
                self.rebuild_stops(cx);
            }
            SliderEvent::ThumbSelected { thumb_id } => {
                self.selected_stop = Some(*thumb_id);
                if self.color_picker_open {
                    let color = self.selected_color(cx);
                    self.color_picker.update(cx, |picker, cx| picker.set_color(color, cx));
                }
                cx.notify();
            }
            SliderEvent::Change { .. } | SliderEvent::Release { .. } => {
                self.rebuild_stops(cx);
            }
        }
    }

    fn handle_add_stop(&mut self, cx: &mut Context<Self>) {
        let position = self.new_stop_position(cx);
        let color = color_at_position(&self.preview_stops(cx), position);
        let mut inserted = None;
        self.gradient_stops.update(cx, |slider, cx| {
            inserted = slider.insert_thumb_at(position, cx);
        });

        if let Some(thumb_id) = inserted {
            self.stop_colors.insert(thumb_id, color);
            self.selected_stop = Some(thumb_id);
            self.sync_stop_buttons(cx);
            self.rebuild_stops(cx);
            cx.notify();
        }
    }

    fn handle_delete_stop(&mut self, thumb_id: ThumbId, cx: &mut Context<Self>) {
        self.stop_colors.remove(&thumb_id);
        self.gradient_stops.update(cx, |slider, cx| {
            slider.remove_thumb_id(thumb_id, cx);
        });
        self.sync_stop_buttons(cx);
        self.rebuild_stops(cx);
        cx.notify();
    }

    fn handle_rotation_event(&mut self, event: &SliderEvent, cx: &mut Context<Self>) {
        if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
            self.rotation_deg = *value;
            self.invalidate_preview_cache();
            let _ = self.ensure_preview_image_cache(self.preview_size, cx);
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

    fn handle_renderer_event(&mut self, event: &SelectorEvent, cx: &mut Context<Self>) {
        let SelectorEvent::Change { item_id, .. } = event;
        let next = if item_id.as_ref() == "render" {
            PreviewRenderer::RenderImage
        } else {
            PreviewRenderer::Quads
        };
        if self.preview_renderer != next {
            self.preview_renderer = next;
            self.invalidate_preview_cache();
            let _ = self.ensure_preview_image_cache(self.preview_size, cx);
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
        self.rebuild_stops(cx);
        cx.notify();
    }

    fn open_color_picker_for_stop(&mut self, thumb_id: ThumbId, cx: &mut Context<Self>) {
        self.selected_stop = Some(thumb_id);
        let color = self.selected_color(cx);
        self.color_picker.update(cx, |picker, cx| picker.set_color(color, cx));
        self.color_picker_open = true;
        cx.notify();
    }

    fn sync_stop_buttons(&mut self, cx: &mut Context<Self>) {
        let thumbs = self.gradient_stops.read(cx).thumbs().to_vec();
        let thumb_ids = thumbs.iter().map(|thumb| thumb.id).collect::<Vec<_>>();
        self.stop_delete_buttons.retain(|thumb_id, _| thumb_ids.contains(thumb_id));
        self.stop_swatch_bounds.retain(|thumb_id, _| thumb_ids.contains(thumb_id));

        let can_remove = thumb_ids.len() > 2;
        for thumb_id in thumb_ids {
            if let Some(button) = self.stop_delete_buttons.get(&thumb_id) {
                button.update(cx, |button, cx| button.set_enabled(can_remove, cx));
                continue;
            }

            let button = self
                .look
                .outline_icon_button(
                    format!("color-viz-gradient-stop-delete-{}", thumb_id.as_u64()),
                    LucideIcon::Trash2,
                )
                .spawn(cx);
            button.update(cx, |button, cx| button.set_enabled(can_remove, cx));
            self._subscriptions.push(cx.subscribe(&button, move |this, _, event: &ButtonEvent, cx| {
                if matches!(event, ButtonEvent::Click) {
                    this.handle_delete_stop(thumb_id, cx);
                }
            }));
            self.stop_delete_buttons.insert(thumb_id, button);
        }
    }

    fn hydrate_thumb_color(&mut self, thumb_id: ThumbId, cx: &Context<Self>) {
        if self.stop_colors.contains_key(&thumb_id) {
            return;
        }

        let slider = self.gradient_stops.read(cx);
        let Some(thumb) = slider.thumbs().iter().find(|thumb| thumb.id == thumb_id) else {
            return;
        };
        let fallback_stops = slider
            .thumbs()
            .iter()
            .filter(|candidate| candidate.id != thumb_id)
            .map(|candidate| {
                let color = self
                    .stop_colors
                    .get(&candidate.id)
                    .copied()
                    .or(candidate.preview)
                    .unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.5, 1.0));
                (candidate.position, color)
            })
            .collect::<Vec<_>>();

        let color = if fallback_stops.is_empty() {
            thumb.preview.unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.5, 1.0))
        } else {
            color_at_position(&sorted_stops(&fallback_stops), thumb.position)
        };
        self.stop_colors.insert(thumb_id, color);
    }

    fn new_stop_position(&self, cx: &Context<Self>) -> f32 {
        let stops = self.ordered_stop_rows(cx);
        if stops.is_empty() {
            return 0.5;
        }

        let Some(selected_stop) = self.selected_stop else {
            return 0.5;
        };
        let Some(index) = stops.iter().position(|(thumb_id, _, _)| *thumb_id == selected_stop) else {
            return 0.5;
        };

        if index + 1 < stops.len() {
            return (stops[index].1 + stops[index + 1].1) * 0.5;
        }
        if index > 0 {
            return (stops[index - 1].1 + stops[index].1) * 0.5;
        }

        (stops[index].1 + 0.5).min(1.0)
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
        self.invalidate_preview_cache();
        let _ = self.ensure_preview_image_cache(self.preview_size, cx);
        refresh_color_slider(&self.gradient_stops, cx);
        cx.notify();
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

    fn handle_stop_swatch_bounds(&mut self, thumb_id: ThumbId, bounds: &Bounds<Pixels>) {
        self.stop_swatch_bounds.insert(thumb_id, *bounds);
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

    fn ordered_stop_rows(&self, cx: &Context<Self>) -> Vec<(ThumbId, f32, gpui::Hsla)> {
        let mut stops = self
            .gradient_stops
            .read(cx)
            .thumbs()
            .iter()
            .map(|thumb| {
                let color = self
                    .stop_colors
                    .get(&thumb.id)
                    .copied()
                    .or(thumb.preview)
                    .unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.5, 1.0));
                (thumb.id, thumb.position, color)
            })
            .collect::<Vec<_>>();
        stops.sort_by(|left, right| left.1.total_cmp(&right.1));
        stops
    }

    fn gradient_spec(&self, cx: &Context<Self>) -> String {
        format_css_linear_gradient(&self.preview_stops(cx), self.rotation_deg)
    }

    fn uses_render_preview(&self, cx: &Context<Self>) -> bool {
        self.preview_renderer == PreviewRenderer::RenderImage && self.preview_stops(cx).len() > 1
    }

    fn sync_preview_strategy(&mut self, stop_count: usize) {
        self.active_preview_strategy = match (stop_count, self.preview_renderer) {
            (0, _) => "empty",
            (1, _) => "solid",
            (2, PreviewRenderer::RenderImage) => "render",
            (2, PreviewRenderer::Quads) => "quads",
            (_, PreviewRenderer::RenderImage) => "render",
            _ => "quads",
        };
        if self.active_preview_strategy != "render" {
            self.last_render_ms = None;
        }
    }

    fn handle_preview_bounds(&mut self, bounds: Bounds<Pixels>, cx: &mut Context<Self>) -> bool {
        let mut refreshed = false;
        if self.preview_size != bounds.size {
            self.preview_size = bounds.size;
            self.invalidate_preview_cache();
        }

        if self.uses_render_preview(cx) && self.ensure_preview_image_cache(bounds.size, cx) {
            refreshed = true;
            cx.notify();
        }
        refreshed
    }

    fn ensure_preview_image_cache(&mut self, preview_size: Size<Pixels>, cx: &Context<Self>) -> bool {
        if !self.uses_render_preview(cx) || preview_size.width <= px(0.0) || preview_size.height <= px(0.0) {
            return false;
        }

        let stops = self.preview_stops(cx);
        let key = preview_image_cache_key(preview_size, self.rotation_deg, &stops);
        if let Some((cached_key, _)) = &self.preview_image_cache
            && *cached_key == key
        {
            return false;
        }

        let started = Instant::now();
        let Some(image) = rasterize_linear_gradient_preview(preview_size, &stops, self.rotation_deg) else {
            return false;
        };

        self.preview_image_cache = Some((key, image));
        self.last_render_ms = Some(started.elapsed().as_secs_f32() * 1000.0);
        true
    }

    fn cached_preview_image(&self) -> Option<Arc<RenderImage>> {
        self.preview_image_cache.as_ref().map(|(_, image)| image.clone())
    }

    fn invalidate_preview_cache(&mut self) {
        self.preview_image_cache = None;
    }
}

impl Render for GradientBuilder {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let stops = self.preview_stops(cx);
        self.sync_preview_strategy(stops.len());
        let rotation_deg = self.rotation_deg;
        let gradient_spec = self.gradient_spec(cx);
        let code_style = self.look.typography_scale(ShadcnTextSize::Xs);
        let info_label_style = self.look.mode_tokens().typography.text.label;
        let info_value_style = self.look.mode_tokens().typography.text.label;
        let gradient_stops = self.gradient_stops.clone();
        let add_stop_button = self.add_stop_button.clone();
        let rotation_slider = self.rotation_slider.clone();
        let type_selector = self.type_selector.clone();
        let renderer_selector = self.renderer_selector.clone();
        let color_picker = self.color_picker.clone();
        let color_picker_open = self.color_picker_open;
        let card_bg = self.look.token_color("card").unwrap_or(chrome.panel_background);
        let stop_rows = self.ordered_stop_rows(cx);
        let preview_corner_radii = preview_panel_corner_radii();
        let preview_surface_radii =
            Corners { top_left: px(0.0), top_right: px(0.0), bottom_left: px(0.0), bottom_right: px(0.0) };
        let preview_uses_render_image = self.preview_renderer == PreviewRenderer::RenderImage && stops.len() > 1;
        let allow_native_two_stop = stops.len() == 2 && self.preview_renderer != PreviewRenderer::Quads;
        let preview_image = if preview_uses_render_image {
            self.cached_preview_image()
        } else {
            None
        };
        let diagnostics_panel = div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .rounded(px(12.0))
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.panel_background)
            .child(render_info_row("Active", self.active_preview_strategy, info_label_style, info_value_style, chrome))
            .child(render_info_row(
                "Requested",
                match self.preview_renderer {
                    PreviewRenderer::Quads => "quads",
                    PreviewRenderer::RenderImage => "render",
                },
                info_label_style,
                info_value_style,
                chrome,
            ))
            .child(render_info_row("Stops", format!("{}", stops.len()), info_label_style, info_value_style, chrome))
            .child(render_info_row(
                "Preview",
                format!(
                    "{} x {}",
                    self.preview_size.width.as_f32().round() as i32,
                    self.preview_size.height.as_f32().round() as i32
                ),
                info_label_style,
                info_value_style,
                chrome,
            ))
            .child(render_info_row(
                "Raster",
                self.last_render_ms.map(|ms| format!("{ms:.2} ms")).unwrap_or_else(|| "n/a".to_string()),
                info_label_style,
                info_value_style,
                chrome,
            ));
        let stop_editor = div()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .rounded(px(12.0))
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.panel_background)
            .child(form_field!("Angle", chrome;
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(rotation_slider)
                    .child(
                        div()
                            .typography_style(code_style)
                            .text_color(chrome.muted_text)
                            .child(format!("{rotation_deg:.0}deg"))
                    )
            ))
            .child(form_field!("Type", chrome; type_selector))
            .child(form_field!("Renderer", chrome; renderer_selector))
            .child(
                div()
                    .id("color-viz-gradient-spec")
                    .typography_style(code_style)
                    .text_color(chrome.muted_text)
                    .font_family("Monaco")
                    .whitespace_normal()
                    .child(gradient_spec),
            );

        let controls = vstack! {
            gap=14;
            div()
                .w_full()
                .flex()
                .items_start()
                .justify_between()
                .gap_4()
                .child(div().flex_1().min_w(px(0.0)).pt_1().child(gradient_stops))
                .child(add_stop_button),
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_2()
                .rounded(px(12.0))
                .border_1()
                .border_color(chrome.border)
                .bg(chrome.panel_background)
                .p_3()
                .children(stop_rows.into_iter().map(|(thumb_id, position, color)| {
                    render_stop_row(
                        thumb_id,
                        position,
                        color,
                        self.selected_stop == Some(thumb_id),
                        self.stop_delete_buttons.get(&thumb_id).cloned(),
                        self.look.mode_tokens().typography.text.label,
                        self.look.mode_tokens().typography.text.label,
                        chrome,
                        cx.entity(),
                    )
                })),
            stop_editor,
            diagnostics_panel,
        }
        .id("color-viz-gradient-controls")
        .w(px(432.0))
        .flex_shrink_0()
        .h_full()
        .min_h_0()
        .corner_radii(controls_panel_corner_radii())
        .overflow_y_scroll()
        .p(px(16.0))
        .bg(card_bg);
        let controls = if color_picker_open {
            if let Some(thumb_id) = self.selected_stop {
                if let Some(bounds) = self.stop_swatch_bounds.get(&thumb_id).copied() {
                    controls.child(
                        deferred(
                            anchored()
                                .snap_to_window_with_margin(px(8.0))
                                .anchor(Corner::TopLeft)
                                .position(point(bounds.left(), bounds.bottom()))
                                .offset(point(px(0.0), px(4.0)))
                                .child(
                                    self.look
                                        .card("color-viz-color-picker-popup")
                                        .elevated(true)
                                        .child(color_picker)
                                        .render(_window, cx)
                                        .occlude()
                                        .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                            this.color_picker_open = false;
                                            cx.notify();
                                        })),
                                ),
                        )
                        .with_priority(1),
                    )
                } else {
                    controls
                }
            } else {
                controls
            }
        } else {
            controls
        };

        let preview_builder = cx.entity();
        let preview_surface = div()
            .id("color-viz-gradient-preview")
            .relative()
            .size_full()
            .rounded(px(0.0))
            .overflow_hidden()
            .on_prepaint(move |bounds, window, cx| {
                preview_builder.update(cx, |this, cx| {
                    if this.handle_preview_bounds(bounds, cx) {
                        window.refresh();
                    }
                });
            })
            .when_some(preview_image, |this, image| {
                this.child(img(ImageSource::Render(image)).size_full().absolute().top_0().left_0())
            })
            .when(!preview_uses_render_image, |this| {
                this.child({
                    let stops = stops.clone();
                    canvas(
                        move |_, _, _| {},
                        move |bounds, _, window, _cx| {
                            paint_linear_gradient_preview(
                                window,
                                bounds,
                                &stops,
                                rotation_deg,
                                preview_surface_radii,
                                allow_native_two_stop,
                            );
                        },
                    )
                    .size_full()
                })
            });
        let preview = div()
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .corner_radii(preview_corner_radii)
            .p(px(10.0))
            .child(preview_surface);

        let shell = hstack! {
            gap=0;
            controls,
            div().w(px(1.0)).flex_shrink_0().bg(chrome.border),
            preview,
        }
        .items_stretch()
        .id("color-viz-gradient-shell")
        .w_full()
        .h_full()
        .min_h_0()
        .rounded(px(SHELL_RADIUS))
        .overflow_hidden()
        .border_1()
        .border_color(chrome.border);

        div()
            .id("color-viz-gradient-builder")
            .size_full()
            .min_h_0()
            .flex()
            .items_stretch()
            .justify_start()
            .p(px(32.0))
            .child(shell)
    }
}

fn render_stop_row(
    thumb_id: ThumbId,
    position: f32,
    color: gpui::Hsla,
    selected: bool,
    delete_button: Option<Entity<Button>>,
    label_style: LumaTextStyle,
    value_style: LumaTextStyle,
    chrome: gpui_luma::theme::LumaChrome,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let border = chrome.border;
    let background = if selected {
        chrome.content_background
    } else {
        chrome.panel_background
    };
    let swatch_builder = builder.clone();
    let bounds_builder = builder.clone();

    div()
        .id(format!("color-viz-gradient-stop-row-{}", thumb_id.as_u64()))
        .w_full()
        .flex()
        .items_center()
        .gap_3()
        .p_3()
        .rounded(px(10.0))
        .border_1()
        .border_color(border)
        .bg(background)
        .cursor_pointer()
        .on_click(move |_, _, cx| {
            builder.update(cx, |this, cx| {
                this.gradient_stops.update(cx, |slider, cx| {
                    slider.select_thumb_id(thumb_id, cx);
                });
            });
        })
        .child(
            div()
                .id(format!("color-viz-gradient-stop-swatch-{}", thumb_id.as_u64()))
                .size(px(28.0))
                .rounded(px(6.0))
                .bg(color)
                .border_1()
                .border_color(chrome.border)
                .cursor_pointer()
                .on_prepaint(move |bounds, _, cx| {
                    bounds_builder.update(cx, |this, _| {
                        this.handle_stop_swatch_bounds(thumb_id, &bounds);
                    });
                })
                .on_click(move |event: &ClickEvent, _, cx| {
                    if event.is_keyboard() {
                        return;
                    }
                    cx.stop_propagation();
                    swatch_builder.update(cx, |this, cx| {
                        this.open_color_picker_for_stop(thumb_id, cx);
                    });
                }),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().typography_style(label_style.clone()).text_color(chrome.muted_text).child("HEX"))
                        .child(
                            div()
                                .typography_style(value_style.clone())
                                .text_color(chrome.body_text)
                                .child(format_hex_color(color)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("POSITION"))
                        .child(
                            div()
                                .typography_style(value_style)
                                .text_color(chrome.body_text)
                                .child(format_percent(position)),
                        ),
                ),
        )
        .when_some(delete_button, |row, button| row.child(button))
}

fn render_info_row(
    label: &'static str,
    value: impl Into<SharedString>,
    label_style: LumaTextStyle,
    value_style: LumaTextStyle,
    chrome: gpui_luma::theme::LumaChrome,
) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .child(div().typography_style(label_style).text_color(chrome.muted_text).child(label))
        .child(div().typography_style(value_style).text_color(chrome.body_text).child(value.into()))
}

fn type_items() -> Vec<SelectorItem> {
    vec![SelectorItem::new("linear").label("Linear")]
}

fn renderer_items() -> Vec<SelectorItem> {
    vec![SelectorItem::new("quads").label("Quads"), SelectorItem::new("render").label("Render")]
}

fn preview_image_cache_key(
    preview_size: Size<Pixels>,
    rotation_deg: f32,
    stops: &[(f32, gpui::Hsla)],
) -> PreviewImageCacheKey {
    PreviewImageCacheKey {
        width_px: preview_size.width.as_f32().round().clamp(0.0, u16::MAX as f32) as u16,
        height_px: preview_size.height.as_f32().round().clamp(0.0, u16::MAX as f32) as u16,
        rotation_tenths: (rotation_deg.rem_euclid(360.0) * 10.0).round().clamp(0.0, u16::MAX as f32) as u16,
        stops: stops.iter().map(|(position, color)| preview_stop_key(*position, *color)).collect(),
    }
}

fn preview_stop_key(position: f32, color: gpui::Hsla) -> PreviewStopKey {
    let rgb = color.to_rgb();
    PreviewStopKey {
        position_millis: (position.clamp(0.0, 1.0) * 1000.0).round() as u16,
        red: (rgb.r.clamp(0.0, 1.0) * 255.0).round() as u8,
        green: (rgb.g.clamp(0.0, 1.0) * 255.0).round() as u8,
        blue: (rgb.b.clamp(0.0, 1.0) * 255.0).round() as u8,
        alpha: (color.a.clamp(0.0, 1.0) * 255.0).round() as u8,
    }
}
