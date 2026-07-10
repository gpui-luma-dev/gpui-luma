use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use gpui::{
    Bounds, ClickEvent, Context, Corner, Corners, Entity, HitboxBehavior, ImageSource, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Render, RenderImage, SharedString, Size, Subscription, Window, anchored,
    canvas, deferred, div, img, point, prelude::*, px, relative, size,
};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::color::composition::CompositionSize;
use gpui_luma::controls::color::style::{ElementExt, StyledExt};
use gpui_luma::controls::color::color_slider::{
    ColorSliderBuilder, ColorSliderDomainRenderer, ColorSliderTrackContext, GradientDelegate, GradientStop,
    refresh_color_slider, update_domain_delegate,
};
use gpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem};
use gpui_luma::controls::slider::{SliderControl, SliderEvent, SliderThumbPolicy, ThumbId};
use gpui_luma::controls::tabs_navigation::{
    TabsNavigation, TabsNavigationEvent, TabsNavigationItem, TabsNavigationWidthMode,
};
use gpui_luma::{form_field, hstack, vstack};
use gpui_luma::theme::{ControlSize, LumaTextStyle, ThemeMode};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnLookControlExt, ShadcnTextSize};
use lucide_icons::Icon as LucideIcon;

use super::color::{format_css_gradient, format_hex_color, format_percent};
use super::paint::{
    GradientType, MeshPoint, PreviewRenderer, MESH_COLS, MESH_ROWS, color_at_position, paint_gradient_preview,
    rasterize_gradient_preview, rasterize_mesh_gradient_preview, sorted_stops,
};
use super::sv_triangle_picker::SvTrianglePicker;

const SHELL_RADIUS: f32 = 12.0;
const SHELL_BORDER: f32 = 1.0;
const MESH_HANDLE_SIZE: f32 = 22.0;
const MESH_POINT_GAP: f32 = 0.08;

fn shell_inner_corner_radius() -> Pixels {
    px(SHELL_RADIUS - SHELL_BORDER)
}

fn controls_panel_corner_radii() -> Corners<Pixels> {
    let inner = shell_inner_corner_radius();
    Corners { top_left: px(0.0), top_right: px(0.0), bottom_left: inner, bottom_right: px(0.0) }
}

fn preview_panel_corner_radii() -> Corners<Pixels> {
    let inner = shell_inner_corner_radius();
    Corners { top_left: px(0.0), top_right: px(0.0), bottom_left: px(0.0), bottom_right: inner }
}

pub struct GradientBuilder {
    look: Arc<ShadcnLook>,
    top_tabs: Entity<TabsNavigation>,
    selected_tab: BuilderTab,
    gradient_stops: Entity<SliderControl>,
    stop_colors: HashMap<ThumbId, gpui::Hsla>,
    selected_stop: Option<ThumbId>,
    domain_renderer: Arc<ColorSliderDomainRenderer>,
    track_context: ColorSliderTrackContext,
    add_stop_button: Entity<Button>,
    mesh_reset_button: Entity<Button>,
    stop_delete_buttons: HashMap<ThumbId, Entity<Button>>,
    rotation_slider: Entity<SliderControl>,
    type_selector: Entity<Selector>,
    renderer_selector: Entity<Selector>,
    rotation_deg: f32,
    gradient_type: GradientType,
    preview_renderer: PreviewRenderer,
    mesh_points: Vec<MeshPoint>,
    mesh_background: gpui::Hsla,
    selected_mesh_point: Option<usize>,
    mesh_color_target: MeshColorTarget,
    active_mesh_drag: Option<usize>,
    preview_size: Size<Pixels>,
    preview_image_cache: Option<(PreviewImageCacheKey, Arc<RenderImage>)>,
    active_preview_strategy: &'static str,
    last_render_ms: Option<f32>,
    color_picker: Entity<SvTrianglePicker>,
    color_picker_open: bool,
    stop_swatch_bounds: HashMap<ThumbId, Bounds<Pixels>>,
    mesh_swatch_bounds: HashMap<usize, Bounds<Pixels>>,
    mesh_background_swatch_bounds: Option<Bounds<Pixels>>,
    mesh_preview_bounds: Option<Bounds<Pixels>>,
    render_task: Option<gpui::Task<()>>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum BuilderTab {
    #[default]
    Gradients,
    Mesh,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MeshColorTarget {
    Point(usize),
    Background,
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
struct PreviewMeshPointKey {
    row: u8,
    col: u8,
    u_millis: u16,
    v_millis: u16,
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PreviewImageCacheKey {
    Gradient {
        gradient_type: GradientType,
        width_px: u16,
        height_px: u16,
        rotation_tenths: u16,
        stops: Vec<PreviewStopKey>,
    },
    Mesh {
        width_px: u16,
        height_px: u16,
        background_red: u8,
        background_green: u8,
        background_blue: u8,
        background_alpha: u8,
        points: Vec<PreviewMeshPointKey>,
    },
}

impl GradientBuilder {
    pub fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let theme_is_dark = matches!(look.mode(), ThemeMode::Dark);
        let start = gpui::hsla(198.0 / 360.0, 1.0, 0.24, 1.0);
        let middle = gpui::hsla(313.0 / 360.0, 0.48, 0.45, 1.0);
        let end = gpui::hsla(33.0 / 360.0, 1.0, 0.46, 1.0);
        let mesh_background = default_mesh_background();

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
        let mesh_reset_button = look.outline_icon_button("color-viz-mesh-reset", LucideIcon::RotateCcw).spawn(cx);

        let rotation_slider =
            look.slider("color-viz-gradient-rotation").range(0.0..360.0).step(1.0).value(90.0).spawn(cx);
        let top_tabs = look
            .tabs_navigation("color-viz-builder-tabs")
            .items(builder_tab_items())
            .active("gradients")
            .width_mode(TabsNavigationWidthMode::Uniform)
            .spawn(cx);
        let type_selector =
            look.selector("color-viz-gradient-type").items(type_items()).selected_id("linear").spawn(cx);
        let renderer_selector =
            look.selector("color-viz-gradient-renderer").items(renderer_items()).selected_id("quads").spawn(cx);
        let color_picker = cx.new(|cx| SvTrianglePicker::with_size(start, CompositionSize::Md, cx));

        let mut builder = Self {
            look: look.clone(),
            top_tabs: top_tabs.clone(),
            selected_tab: BuilderTab::Gradients,
            gradient_stops: gradient_stops.clone(),
            stop_colors,
            selected_stop,
            domain_renderer,
            track_context,
            add_stop_button: add_stop_button.clone(),
            mesh_reset_button: mesh_reset_button.clone(),
            stop_delete_buttons: HashMap::new(),
            rotation_slider: rotation_slider.clone(),
            type_selector: type_selector.clone(),
            renderer_selector: renderer_selector.clone(),
            rotation_deg: 90.0,
            gradient_type: GradientType::Linear,
            preview_renderer: PreviewRenderer::Quads,
            mesh_points: default_mesh_points(),
            mesh_background,
            selected_mesh_point: Some(mesh_point_index(1, 1)),
            mesh_color_target: MeshColorTarget::Point(mesh_point_index(1, 1)),
            active_mesh_drag: None,
            preview_size: size(px(0.0), px(0.0)),
            preview_image_cache: None,
            active_preview_strategy: "quads",
            last_render_ms: None,
            color_picker: color_picker.clone(),
            color_picker_open: false,
            stop_swatch_bounds: HashMap::new(),
            mesh_swatch_bounds: HashMap::new(),
            mesh_background_swatch_bounds: None,
            mesh_preview_bounds: None,
            render_task: None,
            _subscriptions: Vec::new(),
        };

        builder.wire_subscriptions(
            cx,
            top_tabs,
            gradient_stops,
            add_stop_button,
            mesh_reset_button,
            rotation_slider,
            type_selector,
            renderer_selector,
            color_picker,
        );
        builder.sync_stop_buttons(cx);
        builder.rebuild_stops(cx);
        builder
    }

    #[allow(clippy::too_many_arguments)]
    fn wire_subscriptions(
        &mut self,
        cx: &mut Context<Self>,
        top_tabs: Entity<TabsNavigation>,
        gradient_stops: Entity<SliderControl>,
        add_stop_button: Entity<Button>,
        mesh_reset_button: Entity<Button>,
        rotation_slider: Entity<SliderControl>,
        type_selector: Entity<Selector>,
        renderer_selector: Entity<Selector>,
        color_picker: Entity<SvTrianglePicker>,
    ) {
        self._subscriptions.push(cx.subscribe(&gradient_stops, |this, _, event, cx| {
            this.handle_stops_event(event, cx);
        }));
        self._subscriptions.push(cx.subscribe(&top_tabs, |this, _, event: &TabsNavigationEvent, cx| {
            this.handle_top_tabs_event(event, cx);
        }));
        self._subscriptions.push(cx.subscribe(&add_stop_button, |this, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                this.handle_add_stop(cx);
            }
        }));
        self._subscriptions.push(cx.subscribe(&mesh_reset_button, |this, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                this.reset_mesh_state(cx);
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
        self._subscriptions.push(cx.observe(&color_picker, |this, _, cx| {
            this.apply_picker_color(cx);
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
            let _ = self.ensure_preview_image_cache(self.preview_size, cx);
            cx.notify();
        }
    }

    fn handle_top_tabs_event(&mut self, event: &TabsNavigationEvent, cx: &mut Context<Self>) {
        let TabsNavigationEvent::Activate { tab_id, .. } = event;
        let next = match tab_id.as_ref() {
            "mesh" => BuilderTab::Mesh,
            _ => BuilderTab::Gradients,
        };
        if self.selected_tab != next {
            self.selected_tab = next;
            self.active_mesh_drag = None;
            self.color_picker_open = false;
            let _ = self.ensure_preview_image_cache(self.preview_size, cx);
            cx.notify();
        }
    }

    fn handle_type_event(&mut self, event: &SelectorEvent, cx: &mut Context<Self>) {
        let SelectorEvent::Change { item_id, .. } = event;
        let next = match item_id.as_ref() {
            "radial" => GradientType::Radial,
            "angular" => GradientType::Angular,
            _ => GradientType::Linear,
        };
        if self.gradient_type != next {
            self.gradient_type = next;
            let _ = self.ensure_preview_image_cache(self.preview_size, cx);
            cx.notify();
        }
    }

    fn handle_renderer_event(&mut self, event: &SelectorEvent, cx: &mut Context<Self>) {
        let SelectorEvent::Change { item_id, .. } = event;
        let next = match item_id.as_ref() {
            "render" => PreviewRenderer::RenderImageSync,
            "render_async" => PreviewRenderer::RenderImageAsync,
            _ => PreviewRenderer::Quads,
        };
        if self.preview_renderer != next {
            self.preview_renderer = next;
            let _ = self.ensure_preview_image_cache(self.preview_size, cx);
            cx.notify();
        }
    }

    fn apply_picker_color(&mut self, cx: &mut Context<Self>) {
        if !self.color_picker_open {
            return;
        }
        let color = self.color_picker.read(cx).color();
        match self.selected_tab {
            BuilderTab::Gradients => {
                let Some(thumb_id) = self.selected_stop else {
                    return;
                };
                self.stop_colors.insert(thumb_id, color);
                self.rebuild_stops(cx);
            }
            BuilderTab::Mesh => match self.mesh_color_target {
                MeshColorTarget::Point(point_index) => {
                    if let Some(point) = self.mesh_points.get_mut(point_index) {
                        point.color = color;
                        let _ = self.ensure_preview_image_cache(self.preview_size, cx);
                        cx.notify();
                    }
                }
                MeshColorTarget::Background => {
                    self.mesh_background = color;
                    let _ = self.ensure_preview_image_cache(self.preview_size, cx);
                    cx.notify();
                }
            },
        }
    }

    fn open_color_picker_for_stop(&mut self, thumb_id: ThumbId, cx: &mut Context<Self>) {
        self.selected_stop = Some(thumb_id);
        let color = self.selected_color(cx);
        self.color_picker.update(cx, |picker, cx| picker.set_color(color, cx));
        self.color_picker_open = true;
        cx.notify();
    }

    fn open_color_picker_for_mesh_point(&mut self, point_index: usize, cx: &mut Context<Self>) {
        self.selected_mesh_point = Some(point_index);
        self.mesh_color_target = MeshColorTarget::Point(point_index);
        let color = self.selected_mesh_color();
        self.color_picker.update(cx, |picker, cx| picker.set_color(color, cx));
        self.color_picker_open = true;
        cx.notify();
    }

    fn open_color_picker_for_mesh_background(&mut self, cx: &mut Context<Self>) {
        self.mesh_color_target = MeshColorTarget::Background;
        self.color_picker.update(cx, |picker, cx| picker.set_color(self.mesh_background, cx));
        self.color_picker_open = true;
        cx.notify();
    }

    fn reset_mesh_state(&mut self, cx: &mut Context<Self>) {
        self.mesh_points = default_mesh_points();
        self.mesh_background = default_mesh_background();
        self.selected_mesh_point = Some(mesh_point_index(1, 1));
        self.mesh_color_target = MeshColorTarget::Point(mesh_point_index(1, 1));
        self.active_mesh_drag = None;
        self.color_picker_open = false;
        let _ = self.ensure_preview_image_cache(self.preview_size, cx);
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

    fn selected_mesh_color(&self) -> gpui::Hsla {
        match self.mesh_color_target {
            MeshColorTarget::Point(point_index) => self
                .mesh_points
                .get(point_index)
                .map(|point| point.color)
                .unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.5, 1.0)),
            MeshColorTarget::Background => self.mesh_background,
        }
    }

    fn handle_stop_swatch_bounds(&mut self, thumb_id: ThumbId, bounds: &Bounds<Pixels>) {
        self.stop_swatch_bounds.insert(thumb_id, *bounds);
    }

    fn handle_mesh_swatch_bounds(&mut self, point_index: usize, bounds: &Bounds<Pixels>) {
        self.mesh_swatch_bounds.insert(point_index, *bounds);
    }

    fn handle_mesh_background_swatch_bounds(&mut self, bounds: &Bounds<Pixels>) {
        self.mesh_background_swatch_bounds = Some(*bounds);
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
        format_css_gradient(self.gradient_type, &self.preview_stops(cx), self.rotation_deg)
    }

    fn effective_preview_renderer(&self) -> PreviewRenderer {
        match (self.gradient_type, self.preview_renderer) {
            (GradientType::Linear, renderer) => renderer,
            (_, PreviewRenderer::RenderImageAsync) => PreviewRenderer::RenderImageAsync,
            _ => PreviewRenderer::RenderImageSync,
        }
    }

    fn uses_render_preview(&self, cx: &Context<Self>) -> bool {
        if self.selected_tab == BuilderTab::Mesh {
            return true;
        }
        let stop_count = self.preview_stops(cx).len();
        self.gradient_type != GradientType::Linear
            || (matches!(
                self.effective_preview_renderer(),
                PreviewRenderer::RenderImageSync | PreviewRenderer::RenderImageAsync
            ) && stop_count > 1)
    }

    fn sync_preview_strategy(&mut self, stop_count: usize) {
        if self.selected_tab == BuilderTab::Mesh {
            self.active_preview_strategy = "render (sync)";
            return;
        }
        self.active_preview_strategy = match (self.gradient_type, stop_count, self.effective_preview_renderer()) {
            (_, 0, _) => "empty",
            (_, 1, _) => "solid",
            (GradientType::Linear, 2, PreviewRenderer::Quads) => "quads",
            (GradientType::Linear, _, PreviewRenderer::Quads) => "quads",
            (_, _, PreviewRenderer::RenderImageAsync) => "render (async)",
            _ => "render (sync)",
        };
        if !self.active_preview_strategy.starts_with("render") {
            self.last_render_ms = None;
        }
    }

    fn handle_preview_bounds(&mut self, bounds: Bounds<Pixels>, cx: &mut Context<Self>) -> bool {
        let mut refreshed = false;
        if self.preview_size != bounds.size {
            self.preview_size = bounds.size;
        }
        if self.selected_tab == BuilderTab::Mesh {
            self.mesh_preview_bounds = Some(bounds);
        }

        if self.uses_render_preview(cx) && self.ensure_preview_image_cache(bounds.size, cx) {
            refreshed = true;
            cx.notify();
        }
        refreshed
    }

    fn ensure_preview_image_cache(&mut self, preview_size: Size<Pixels>, cx: &mut Context<Self>) -> bool {
        if !self.uses_render_preview(cx) || preview_size.width <= px(0.0) || preview_size.height <= px(0.0) {
            return false;
        }

        let key = match self.selected_tab {
            BuilderTab::Gradients => {
                let stops = self.preview_stops(cx);
                preview_gradient_cache_key(self.gradient_type, preview_size, self.rotation_deg, &stops)
            }
            BuilderTab::Mesh => preview_mesh_cache_key(preview_size, &self.mesh_points, self.mesh_background),
        };
        if let Some((cached_key, _)) = &self.preview_image_cache
            && *cached_key == key
        {
            return false;
        }

        self.render_task = None;

        if self.selected_tab == BuilderTab::Mesh
            || self.effective_preview_renderer() == PreviewRenderer::RenderImageSync
        {
            let started = Instant::now();
            let image = match self.selected_tab {
                BuilderTab::Gradients => {
                    let stops = self.preview_stops(cx);
                    rasterize_gradient_preview(self.gradient_type, preview_size, &stops, self.rotation_deg)
                }
                BuilderTab::Mesh => {
                    rasterize_mesh_gradient_preview(preview_size, &self.mesh_points, self.mesh_background)
                }
            };
            let Some(image) = image else {
                return false;
            };
            self.preview_image_cache = Some((key, image));
            self.last_render_ms = Some(started.elapsed().as_secs_f32() * 1000.0);
            true
        } else {
            let gradient_type = self.gradient_type;
            let rotation_deg = self.rotation_deg;
            let stops = self.preview_stops(cx);
            self.render_task = Some(cx.spawn(async move |this, cx| {
                let started = Instant::now();
                let image = cx
                    .background_executor()
                    .spawn(async move { rasterize_gradient_preview(gradient_type, preview_size, &stops, rotation_deg) })
                    .await;

                let _ = this.update(cx, |this, cx| {
                    if let Some(image) = image {
                        this.preview_image_cache = Some((key, image));
                        this.last_render_ms = Some(started.elapsed().as_secs_f32() * 1000.0);
                        cx.notify();
                    }
                });
            }));
            false
        }
    }

    fn cached_preview_image(&self) -> Option<Arc<RenderImage>> {
        self.preview_image_cache.as_ref().map(|(_, image)| image.clone())
    }

    fn begin_mesh_drag(&mut self, point_index: usize, cx: &mut Context<Self>) {
        self.selected_mesh_point = Some(point_index);
        self.mesh_color_target = MeshColorTarget::Point(point_index);
        self.active_mesh_drag = Some(point_index);
        self.color_picker_open = false;
        cx.notify();
    }

    fn handle_mesh_drag_move(&mut self, position: gpui::Point<Pixels>, cx: &mut Context<Self>) {
        let Some(point_index) = self.active_mesh_drag else {
            return;
        };
        let Some(bounds) = self.mesh_preview_bounds else {
            return;
        };
        let width = bounds.size.width.as_f32();
        let height = bounds.size.height.as_f32();
        if width <= 0.0 || height <= 0.0 {
            return;
        }

        let u = ((position.x - bounds.origin.x).as_f32() / width).clamp(0.0, 1.0);
        let v = ((position.y - bounds.origin.y).as_f32() / height).clamp(0.0, 1.0);
        let (u, v) = self.constrained_mesh_position(point_index, u, v);
        let Some(point) = self.mesh_points.get_mut(point_index) else {
            return;
        };
        if (point.u - u).abs() <= f32::EPSILON && (point.v - v).abs() <= f32::EPSILON {
            return;
        }
        point.u = u;
        point.v = v;
        let _ = self.ensure_preview_image_cache(self.preview_size, cx);
        cx.notify();
    }

    fn finish_mesh_drag(&mut self, cx: &mut Context<Self>) {
        if self.active_mesh_drag.take().is_some() {
            cx.notify();
        }
    }

    fn constrained_mesh_position(&self, point_index: usize, proposed_u: f32, proposed_v: f32) -> (f32, f32) {
        let Some(point) = self.mesh_points.get(point_index).copied() else {
            return (proposed_u.clamp(0.0, 1.0), proposed_v.clamp(0.0, 1.0));
        };

        let row = point.row as usize;
        let col = point.col as usize;
        let mut u = proposed_u.clamp(0.0, 1.0);
        let mut v = proposed_v.clamp(0.0, 1.0);

        if col == 0 {
            let right = self.mesh_points[mesh_point_index(row, col + 1)].u - MESH_POINT_GAP;
            u = u.clamp(0.0, right.max(0.0));
        } else if col + 1 == MESH_COLS {
            let left = self.mesh_points[mesh_point_index(row, col - 1)].u + MESH_POINT_GAP;
            u = u.clamp(left.min(1.0), 1.0);
        } else {
            let left = self.mesh_points[mesh_point_index(row, col - 1)].u + MESH_POINT_GAP;
            let right = self.mesh_points[mesh_point_index(row, col + 1)].u - MESH_POINT_GAP;
            u = u.clamp(left.min(right), left.max(right));
        }

        if row == 0 {
            let bottom = self.mesh_points[mesh_point_index(row + 1, col)].v - MESH_POINT_GAP;
            v = v.clamp(0.0, bottom.max(0.0));
        } else if row + 1 == MESH_ROWS {
            let top = self.mesh_points[mesh_point_index(row - 1, col)].v + MESH_POINT_GAP;
            v = v.clamp(top.min(1.0), 1.0);
        } else {
            let top = self.mesh_points[mesh_point_index(row - 1, col)].v + MESH_POINT_GAP;
            let bottom = self.mesh_points[mesh_point_index(row + 1, col)].v - MESH_POINT_GAP;
            v = v.clamp(top.min(bottom), top.max(bottom));
        }

        (u, v)
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
        let top_tabs = self.top_tabs.clone();
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
        let preview_uses_render_image = self.uses_render_preview(cx);
        let allow_native_two_stop = self.gradient_type == GradientType::Linear
            && stops.len() == 2
            && self.effective_preview_renderer() == PreviewRenderer::Quads;
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
                    PreviewRenderer::RenderImageSync => "render (sync)",
                    PreviewRenderer::RenderImageAsync => "render (async)",
                },
                info_label_style,
                info_value_style,
                chrome,
            ))
            .child(render_info_row("Type", self.gradient_type.label(), info_label_style, info_value_style, chrome))
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
            .child(form_field!("Type", chrome; type_selector))
            .child(form_field!("Renderer", chrome; renderer_selector))
            .when(self.gradient_type != GradientType::Radial, |this| {
                this.child(form_field!("Angle", chrome;
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
            })
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
        let picker_bounds = match self.selected_tab {
            BuilderTab::Gradients => {
                self.selected_stop.and_then(|thumb_id| self.stop_swatch_bounds.get(&thumb_id).copied())
            }
            BuilderTab::Mesh => match self.mesh_color_target {
                MeshColorTarget::Point(point_index) => self.mesh_swatch_bounds.get(&point_index).copied(),
                MeshColorTarget::Background => self.mesh_background_swatch_bounds,
            },
        };
        let content = match self.selected_tab {
            BuilderTab::Gradients => {
                let controls = if color_picker_open {
                    if let Some(bounds) = picker_bounds {
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
                                            .child(color_picker.clone())
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
                            let gradient_type = self.gradient_type;
                            canvas(
                                move |_, _, _| {},
                                move |bounds, _, window, _cx| {
                                    paint_gradient_preview(
                                        gradient_type,
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

                hstack! {
                    gap=0;
                    controls,
                    div().w(px(1.0)).flex_shrink_0().bg(chrome.border),
                    preview,
                }
                .items_stretch()
                .w_full()
                .flex_1()
                .min_h_0()
                .into_any_element()
            }
            BuilderTab::Mesh => {
                let mesh_controls = div().child(render_mesh_controls(
                    self.mesh_points.clone(),
                    self.mesh_background,
                    self.selected_mesh_point,
                    self.mesh_reset_button.clone(),
                    chrome,
                    card_bg,
                    info_label_style,
                    info_value_style,
                    self.preview_size,
                    self.last_render_ms,
                    cx.entity(),
                ));
                let mesh_controls = if color_picker_open {
                    if let Some(bounds) = picker_bounds {
                        mesh_controls.child(
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
                                            .child(color_picker.clone())
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
                        mesh_controls
                    }
                } else {
                    mesh_controls
                };
                let preview_builder = cx.entity();
                let preview_surface = div()
                    .id("color-viz-mesh-preview")
                    .relative()
                    .size_full()
                    .rounded(px(0.0))
                    .on_prepaint(move |bounds, window, cx| {
                        preview_builder.update(cx, |this, cx| {
                            if this.handle_preview_bounds(bounds, cx) {
                                window.refresh();
                            }
                        });
                    })
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .overflow_hidden()
                            .when_some(self.cached_preview_image(), |this, image| {
                                this.child(img(ImageSource::Render(image)).size_full().absolute().top_0().left_0())
                            })
                            .child(
                                canvas(|bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal), {
                                    let builder = cx.entity();
                                    move |bounds, _hitbox, window, cx| {
                                        builder.update(cx, |this, cx| {
                                            this.mesh_preview_bounds = Some(bounds);
                                            if this.handle_preview_bounds(bounds, cx) {
                                                window.refresh();
                                            }
                                        });

                                        window.on_mouse_event({
                                            let builder = builder.clone();
                                            move |event: &MouseMoveEvent, phase, _window, cx| {
                                                if !phase.bubble() && !phase.capture() {
                                                    return;
                                                }
                                                builder.update(cx, |this, cx| {
                                                    this.handle_mesh_drag_move(event.position, cx);
                                                });
                                            }
                                        });

                                        window.on_mouse_event({
                                            let builder = builder.clone();
                                            move |_event: &MouseUpEvent, _phase, _window, cx| {
                                                builder.update(cx, |this, cx| {
                                                    this.finish_mesh_drag(cx);
                                                });
                                            }
                                        });
                                    }
                                })
                                .absolute()
                                .inset_0(),
                            ),
                    )
                    .children(self.mesh_points.iter().enumerate().map(|(point_index, point)| {
                        render_mesh_handle(
                            point_index,
                            *point,
                            self.selected_mesh_point == Some(point_index),
                            cx.entity(),
                        )
                    }));
                let preview = div()
                    .flex_1()
                    .min_w(px(0.0))
                    .h_full()
                    .corner_radii(preview_corner_radii)
                    .p(px(18.0))
                    .child(preview_surface);

                hstack! {
                    gap=0;
                    mesh_controls,
                    div().w(px(1.0)).flex_shrink_0().bg(chrome.border),
                    preview,
                }
                .items_stretch()
                .w_full()
                .flex_1()
                .min_h_0()
                .into_any_element()
            }
        };

        let shell = div()
            .id("color-viz-gradient-shell")
            .w_full()
            .h_full()
            .min_h_0()
            .flex()
            .flex_col()
            .rounded(px(SHELL_RADIUS))
            .overflow_hidden()
            .border_1()
            .border_color(chrome.border)
            .child(div().w_full().flex_shrink_0().p(px(10.0)).bg(card_bg).child(top_tabs))
            .child(div().h(px(1.0)).w_full().flex_shrink_0().bg(chrome.border))
            .child(content);

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

#[allow(clippy::too_many_arguments)]
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
                        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("HEX"))
                        .child(
                            div()
                                .typography_style(value_style)
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
    vec![
        SelectorItem::new("linear").label("Linear"),
        SelectorItem::new("radial").label("Radial"),
        SelectorItem::new("angular").label("Angular"),
    ]
}

fn renderer_items() -> Vec<SelectorItem> {
    vec![
        SelectorItem::new("quads").label("Quads"),
        SelectorItem::new("render").label("Render (Sync)"),
        SelectorItem::new("render_async").label("Render (Async)"),
    ]
}

fn preview_gradient_cache_key(
    gradient_type: GradientType,
    preview_size: Size<Pixels>,
    rotation_deg: f32,
    stops: &[(f32, gpui::Hsla)],
) -> PreviewImageCacheKey {
    PreviewImageCacheKey::Gradient {
        gradient_type,
        width_px: preview_size.width.as_f32().round().clamp(0.0, u16::MAX as f32) as u16,
        height_px: preview_size.height.as_f32().round().clamp(0.0, u16::MAX as f32) as u16,
        rotation_tenths: (rotation_deg.rem_euclid(360.0) * 10.0).round().clamp(0.0, u16::MAX as f32) as u16,
        stops: stops.iter().map(|(position, color)| preview_stop_key(*position, *color)).collect(),
    }
}

fn preview_mesh_cache_key(
    preview_size: Size<Pixels>,
    points: &[MeshPoint],
    background: gpui::Hsla,
) -> PreviewImageCacheKey {
    let background_rgb = background.to_rgb();
    PreviewImageCacheKey::Mesh {
        width_px: preview_size.width.as_f32().round().clamp(0.0, u16::MAX as f32) as u16,
        height_px: preview_size.height.as_f32().round().clamp(0.0, u16::MAX as f32) as u16,
        background_red: (background_rgb.r.clamp(0.0, 1.0) * 255.0).round() as u8,
        background_green: (background_rgb.g.clamp(0.0, 1.0) * 255.0).round() as u8,
        background_blue: (background_rgb.b.clamp(0.0, 1.0) * 255.0).round() as u8,
        background_alpha: (background.a.clamp(0.0, 1.0) * 255.0).round() as u8,
        points: points
            .iter()
            .map(|point| {
                let rgb = point.color.to_rgb();
                PreviewMeshPointKey {
                    row: point.row,
                    col: point.col,
                    u_millis: (point.u.clamp(0.0, 1.0) * 1000.0).round() as u16,
                    v_millis: (point.v.clamp(0.0, 1.0) * 1000.0).round() as u16,
                    red: (rgb.r.clamp(0.0, 1.0) * 255.0).round() as u8,
                    green: (rgb.g.clamp(0.0, 1.0) * 255.0).round() as u8,
                    blue: (rgb.b.clamp(0.0, 1.0) * 255.0).round() as u8,
                    alpha: (point.color.a.clamp(0.0, 1.0) * 255.0).round() as u8,
                }
            })
            .collect(),
    }
}

fn builder_tab_items() -> Vec<TabsNavigationItem> {
    vec![
        TabsNavigationItem::new("gradients").label("Gradients"),
        TabsNavigationItem::new("mesh").label("Mesh"),
    ]
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

fn mesh_point_index(row: usize, col: usize) -> usize {
    row * MESH_COLS + col
}

fn default_mesh_points() -> Vec<MeshPoint> {
    let top = gpui::hsla(46.0 / 360.0, 1.0, 0.51, 1.0);
    let middle = gpui::hsla(349.0 / 360.0, 1.0, 0.58, 1.0);
    let bottom = gpui::hsla(212.0 / 360.0, 0.86, 0.49, 1.0);

    [
        (0, 0, 0.0, 0.0, top),
        (0, 1, 1.0 / 3.0, 0.0, top),
        (0, 2, 2.0 / 3.0, 0.0, top),
        (0, 3, 1.0, 0.0, top),
        (1, 0, 0.0, 0.5, middle),
        (1, 1, 1.0 / 3.0, 0.5, middle),
        (1, 2, 2.0 / 3.0, 0.5, middle),
        (1, 3, 1.0, 0.5, middle),
        (2, 0, 0.0, 1.0, bottom),
        (2, 1, 1.0 / 3.0, 1.0, bottom),
        (2, 2, 2.0 / 3.0, 1.0, bottom),
        (2, 3, 1.0, 1.0, bottom),
    ]
    .into_iter()
    .map(|(row, col, u, v, color)| MeshPoint { row, col, u, v, color })
    .collect()
}

fn default_mesh_background() -> gpui::Hsla {
    gpui::hsla(222.0 / 360.0, 0.22, 0.12, 1.0)
}

#[allow(clippy::too_many_arguments)]
fn render_mesh_controls(
    mesh_points: Vec<MeshPoint>,
    mesh_background: gpui::Hsla,
    selected_mesh_point: Option<usize>,
    mesh_reset_button: Entity<Button>,
    chrome: gpui_luma::theme::LumaChrome,
    card_bg: gpui::Hsla,
    info_label_style: LumaTextStyle,
    info_value_style: LumaTextStyle,
    preview_size: Size<Pixels>,
    last_render_ms: Option<f32>,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let selected_label = selected_mesh_point
        .and_then(|index| mesh_points.get(index))
        .map(|point| format!("P{}{}", point.row, point.col))
        .unwrap_or_else(|| "None".to_string());

    vstack! {
        gap=14;
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .rounded(px(12.0))
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.panel_background)
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(div().typography_style(info_label_style).text_color(chrome.muted_text).child("Mesh"))
                    .child(mesh_reset_button),
            )
            .child(render_info_row("Grid", "3 x 4", info_label_style, info_value_style, chrome))
            .child(render_info_row("Selected", selected_label, info_label_style, info_value_style, chrome))
            .child(render_mesh_background_row(mesh_background, info_label_style, chrome, builder.clone())),
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
            .children(mesh_points.into_iter().enumerate().map(|(point_index, point)| {
                render_mesh_point_row(
                    point_index,
                    point,
                    selected_mesh_point == Some(point_index),
                    info_label_style,
                    chrome,
                    builder.clone(),
                )
            })),
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .rounded(px(12.0))
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.panel_background)
            .child(render_info_row(
                "Preview",
                format!(
                    "{} x {}",
                    preview_size.width.as_f32().round() as i32,
                    preview_size.height.as_f32().round() as i32
                ),
                info_label_style,
                info_value_style,
                chrome,
            ))
            .child(render_info_row(
                "Raster",
                last_render_ms.map(|ms| format!("{ms:.2} ms")).unwrap_or_else(|| "n/a".to_string()),
                info_label_style,
                info_value_style,
                chrome,
            )),
    }
    .id("color-viz-mesh-controls")
    .w(px(432.0))
    .flex_shrink_0()
    .h_full()
    .min_h_0()
    .corner_radii(controls_panel_corner_radii())
    .overflow_y_scroll()
    .p(px(16.0))
    .bg(card_bg)
}

fn render_mesh_background_row(
    background: gpui::Hsla,
    label_style: LumaTextStyle,
    chrome: gpui_luma::theme::LumaChrome,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let bounds_builder = builder.clone();
    let row_builder = builder.clone();

    div()
        .id("color-viz-mesh-background-row")
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .p_3()
        .rounded(px(10.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.content_background)
        .cursor_pointer()
        .on_click(move |event: &ClickEvent, _, cx| {
            if event.is_keyboard() {
                return;
            }
            row_builder.update(cx, |this, cx| {
                this.open_color_picker_for_mesh_background(cx);
            });
        })
        .child(div().typography_style(label_style).text_color(chrome.body_text).child("Background"))
        .child(
            div()
                .id("color-viz-mesh-background-swatch")
                .size(px(28.0))
                .rounded(px(999.0))
                .bg(background)
                .border_2()
                .border_color(chrome.border)
                .cursor_pointer()
                .on_prepaint(move |bounds, _, cx| {
                    bounds_builder.update(cx, |this, _| {
                        this.handle_mesh_background_swatch_bounds(&bounds);
                    });
                })
                .on_click(move |event: &ClickEvent, _, cx| {
                    if event.is_keyboard() {
                        return;
                    }
                    cx.stop_propagation();
                    builder.update(cx, |this, cx| {
                        this.open_color_picker_for_mesh_background(cx);
                    });
                }),
        )
}

fn render_mesh_point_row(
    point_index: usize,
    point: MeshPoint,
    selected: bool,
    label_style: LumaTextStyle,
    chrome: gpui_luma::theme::LumaChrome,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let bounds_builder = builder.clone();
    let swatch_builder = builder.clone();
    let background = if selected {
        chrome.content_background
    } else {
        chrome.panel_background
    };

    div()
        .id(format!("color-viz-mesh-point-row-{point_index}"))
        .w_full()
        .flex()
        .items_center()
        .gap_3()
        .p_3()
        .rounded(px(10.0))
        .border_1()
        .border_color(chrome.border)
        .bg(background)
        .cursor_pointer()
        .on_click(move |event: &ClickEvent, _, cx| {
            if event.is_keyboard() {
                return;
            }
            builder.update(cx, |this, cx| {
                if event.click_count() >= 2 {
                    this.open_color_picker_for_mesh_point(point_index, cx);
                } else {
                    this.selected_mesh_point = Some(point_index);
                    this.mesh_color_target = MeshColorTarget::Point(point_index);
                    this.color_picker_open = false;
                    cx.notify();
                }
            });
        })
        .child(
            div()
                .id(format!("color-viz-mesh-point-swatch-{point_index}"))
                .size(px(28.0))
                .rounded(px(999.0))
                .bg(point.color)
                .border_2()
                .border_color(chrome.border)
                .cursor_pointer()
                .on_prepaint(move |bounds, _, cx| {
                    bounds_builder.update(cx, |this, _| {
                        this.handle_mesh_swatch_bounds(point_index, &bounds);
                    });
                })
                .on_click(move |event: &ClickEvent, _, cx| {
                    if event.is_keyboard() {
                        return;
                    }
                    cx.stop_propagation();
                    swatch_builder.update(cx, |this, cx| {
                        this.open_color_picker_for_mesh_point(point_index, cx);
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
                        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("POINT"))
                        .child(
                            div()
                                .typography_style(label_style)
                                .text_color(chrome.body_text)
                                .child(format!("P{}{}", point.row, point.col)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("X"))
                        .child(
                            div()
                                .typography_style(label_style)
                                .text_color(chrome.body_text)
                                .child(format_percent(point.u)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("Y"))
                        .child(
                            div()
                                .typography_style(label_style)
                                .text_color(chrome.body_text)
                                .child(format_percent(point.v)),
                        ),
                ),
        )
}

fn render_mesh_handle(
    point_index: usize,
    point: MeshPoint,
    selected: bool,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let border = if selected {
        gpui::hsla(0.0, 0.0, 1.0, 1.0)
    } else {
        gpui::hsla(0.0, 0.0, 1.0, 0.82)
    };
    let drag_builder = builder.clone();
    let release_builder = builder.clone();

    div()
        .id(format!("color-viz-mesh-handle-{point_index}"))
        .absolute()
        .left(relative(point.u))
        .top(relative(point.v))
        .ml(px(-MESH_HANDLE_SIZE * 0.5))
        .mt(px(-MESH_HANDLE_SIZE * 0.5))
        .size(px(MESH_HANDLE_SIZE))
        .rounded(px(999.0))
        .bg(point.color)
        .border_2()
        .border_color(border)
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            drag_builder.update(cx, |this, cx| {
                this.begin_mesh_drag(point_index, cx);
            });
        })
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            release_builder.update(cx, |this, cx| {
                this.finish_mesh_drag(cx);
            });
        })
        .on_click(move |event: &ClickEvent, _, cx| {
            if event.is_keyboard() {
                return;
            }
            cx.stop_propagation();
            builder.update(cx, |this, cx| {
                this.selected_mesh_point = Some(point_index);
                this.mesh_color_target = MeshColorTarget::Point(point_index);
                this.color_picker_open = false;
                cx.notify();
            });
        })
}
