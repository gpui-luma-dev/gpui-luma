use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use gpui::{AppContext, Bounds, Context, Entity, Pixels, RenderImage, Size, Subscription, px, size};
use luma::controls::color::color_slider::{ColorSliderBuilder, ColorSliderDomainRenderer, ColorSliderTrackContext};
use luma::controls::color::composition::CompositionSize;
use luma::controls::button::{Button, ButtonEvent};
use luma::controls::selector::{Selector, SelectorEvent};
use luma::controls::slider::{SliderControl, SliderEvent, SliderThumbPolicy, ThumbId};
use luma::controls::tabs::{Tabs, TabsEvent, TabsWidthMode};
use luma::theme::{ControlSize, ThemeMode};
use luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};
use lucide_svg_static::Icon as LucideIcon;

mod actions;
mod mesh;
mod view;

use super::paint::{GradientType, MeshPoint, PreviewRenderer};
use super::sv_triangle_picker::SvTrianglePicker;
use mesh::{
    BuilderTab, MeshAspectRatioPreset, MeshColorTarget, MeshGridPreset, PreviewImageCacheKey, builder_tab_items,
    default_mesh_background, default_mesh_points, default_mesh_selected_index, mesh_aspect_ratio_items,
    mesh_grid_items, renderer_items, type_items,
};

pub struct GradientBuilder {
    look: Arc<ShadcnLook>,
    top_tabs: Entity<Tabs>,
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
    mesh_grid_selector: Entity<Selector>,
    mesh_aspect_ratio_selector: Entity<Selector>,
    rotation_deg: f32,
    gradient_type: GradientType,
    preview_renderer: PreviewRenderer,
    mesh_grid_preset: MeshGridPreset,
    mesh_aspect_ratio_preset: MeshAspectRatioPreset,
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
    mesh_preview_container_size: Size<Pixels>,
    mesh_preview_bounds: Option<Bounds<Pixels>>,
    render_task: Option<gpui::Task<()>>,
    _subscriptions: Vec<Subscription>,
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
            .tabs("color-viz-builder-tabs")
            .items(builder_tab_items())
            .active("gradients")
            .width_mode(TabsWidthMode::Uniform)
            .spawn(cx);
        let type_selector =
            look.selector("color-viz-gradient-type").items(type_items()).selected_id("linear").spawn(cx);
        let renderer_selector =
            look.selector("color-viz-gradient-renderer").items(renderer_items()).selected_id("quads").spawn(cx);
        let mesh_grid_selector =
            look.selector("color-viz-mesh-grid").items(mesh_grid_items()).selected_id("3x4").spawn(cx);
        let mesh_aspect_ratio_selector =
            look.selector("color-viz-mesh-aspect").items(mesh_aspect_ratio_items()).selected_id("3:4").spawn(cx);
        let color_picker = cx.new(|cx| SvTrianglePicker::with_size(start, CompositionSize::Md, cx));
        let mesh_grid_preset = MeshGridPreset::SampleThreeByFour;
        let mesh_aspect_ratio_preset = MeshAspectRatioPreset::ThreeByFour;

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
            mesh_grid_selector: mesh_grid_selector.clone(),
            mesh_aspect_ratio_selector: mesh_aspect_ratio_selector.clone(),
            rotation_deg: 90.0,
            gradient_type: GradientType::Linear,
            preview_renderer: PreviewRenderer::Quads,
            mesh_grid_preset,
            mesh_aspect_ratio_preset,
            mesh_points: default_mesh_points(mesh_grid_preset),
            mesh_background,
            selected_mesh_point: Some(default_mesh_selected_index(mesh_grid_preset)),
            mesh_color_target: MeshColorTarget::Point(default_mesh_selected_index(mesh_grid_preset)),
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
            mesh_preview_container_size: size(px(0.0), px(0.0)),
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
            mesh_grid_selector,
            mesh_aspect_ratio_selector,
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
        top_tabs: Entity<Tabs>,
        gradient_stops: Entity<SliderControl>,
        add_stop_button: Entity<Button>,
        mesh_reset_button: Entity<Button>,
        rotation_slider: Entity<SliderControl>,
        type_selector: Entity<Selector>,
        renderer_selector: Entity<Selector>,
        mesh_grid_selector: Entity<Selector>,
        mesh_aspect_ratio_selector: Entity<Selector>,
        color_picker: Entity<SvTrianglePicker>,
    ) {
        self._subscriptions.push(cx.subscribe(&gradient_stops, |this, _, event, cx| {
            this.handle_stops_event(event, cx);
        }));
        self._subscriptions.push(cx.subscribe(&top_tabs, |this, _, event: &TabsEvent, cx| {
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
        self._subscriptions.push(cx.subscribe(&mesh_grid_selector, |this, _, event, cx| {
            this.handle_mesh_grid_event(event, cx);
        }));
        self._subscriptions.push(cx.subscribe(&mesh_aspect_ratio_selector, |this, _, event, cx| {
            this.handle_mesh_aspect_ratio_event(event, cx);
        }));
        self._subscriptions.push(cx.observe(&color_picker, |this, _, cx| {
            this.apply_picker_color(cx);
        }));
    }
}
