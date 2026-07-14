use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, AppContext, Entity, Hsla, IntoElement, Pixels, SharedString, px};

use super::{
    ResizablePanels, ResizablePanelsTemplate, ResizablePanelsTheme, default_resizable_panels_template,
    default_resizable_panels_theme,
};
use super::template::modified_resizable_panels_template;

pub type PanelRender = Rc<dyn Fn() -> AnyElement>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResizablePanelsOrientation {
    Horizontal,
    Vertical,
}

/// Preset overlay resize handle dimensions (lane, grip, and hit target scale together).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ResizeHandleSize {
    Sm,
    #[default]
    Md,
    Lg,
}

/// Controls when the resize handle lane and grip are visibly rendered.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ResizeHandleVisibility {
    /// Keep the seam visible only as a divider while retaining the resize hit target.
    #[default]
    Hidden,
    /// Reveal the configured handle lane while hovering, focusing, or dragging it.
    Hover,
    /// Always show the configured handle lane.
    Always,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResizeCollapseMode {
    ToMinSize,
    Completely,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PanelHideMode {
    ToMinSize,
    Completely,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResizeCollapseDirection {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResizeCollapseBehavior {
    pub mode: ResizeCollapseMode,
    pub direction: ResizeCollapseDirection,
}

impl ResizeCollapseBehavior {
    pub fn new(mode: ResizeCollapseMode, direction: ResizeCollapseDirection) -> Self {
        Self { mode, direction }
    }
}

/// Resolved pixel metrics for an overlay resize handle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResizeHandleMetrics {
    /// Visible overlay lane width/height on the split seam.
    pub lane_px: f32,
    /// Pointer hit target extent (may extend past the lane).
    pub hit_target_px: f32,
    pub grip_cross_axis_px: f32,
    pub grip_main_axis_px: f32,
}

impl ResizeHandleSize {
    pub fn metrics(self) -> ResizeHandleMetrics {
        match self {
            Self::Sm => ResizeHandleMetrics {
                lane_px: 8.0,
                hit_target_px: 12.0,
                grip_cross_axis_px: 4.0,
                grip_main_axis_px: 36.0,
            },
            Self::Md => ResizeHandleMetrics {
                lane_px: 12.0,
                hit_target_px: 14.0,
                grip_cross_axis_px: 6.0,
                grip_main_axis_px: 40.0,
            },
            Self::Lg => ResizeHandleMetrics {
                lane_px: 18.0,
                hit_target_px: 20.0,
                grip_cross_axis_px: 8.0,
                grip_main_axis_px: 44.0,
            },
        }
    }

    pub fn lane_pixels(self) -> Pixels {
        px(self.metrics().lane_px)
    }

    /// Maps a pixel lane width to the nearest preset.
    pub fn from_lane_px(lane_px: f32) -> Self {
        if lane_px >= 16.0 {
            Self::Lg
        } else if lane_px >= 10.0 {
            Self::Md
        } else {
            Self::Sm
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanelSize {
    /// Exact width or height in pixels on the main axis.
    Absolute(Pixels),
    /// Proportional weight sharing the remainder after absolute panes.
    Weight(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanelLayoutState {
    Absolute(f32),
    Weight(f32),
}

impl PanelLayoutState {
    pub fn absolute_px(self) -> f32 {
        match self {
            Self::Absolute(px) => px,
            Self::Weight(weight) => weight,
        }
    }

    pub fn from_spec(spec: &ResizablePanelSpec) -> Self {
        match spec.size {
            PanelSize::Absolute(pixels) => Self::Absolute(pixels.as_f32()),
            PanelSize::Weight(weight) => Self::Weight(weight.max(0.0)),
        }
    }
}

#[derive(Clone)]
pub struct ResizablePanelSpec {
    pub size: PanelSize,
    pub min_px: Option<f32>,
    pub max_px: Option<f32>,
    /// Panel background used by the shell and the overlay handle halves on this edge.
    pub background: Option<Hsla>,
    pub render: PanelRender,
}

impl ResizablePanelSpec {
    pub fn new_render<E, F>(render: F) -> Self
    where
        E: IntoElement,
        F: Fn() -> E + 'static,
    {
        Self {
            size: PanelSize::Weight(1.0),
            min_px: None,
            max_px: None,
            background: None,
            render: Rc::new(move || render().into_any_element()),
        }
    }

    /// Background color for this pane (required for overlay-handle shells).
    pub fn bg(mut self, background: Hsla) -> Self {
        self.background = Some(background);
        self
    }

    /// Fixed main-axis size in pixels.
    pub fn size(mut self, width: Pixels) -> Self {
        self.size = PanelSize::Absolute(width);
        self
    }

    /// Proportional weight for the post-absolute remainder.
    pub fn weight(mut self, weight: f32) -> Self {
        self.size = PanelSize::Weight(weight.max(0.0));
        self
    }

    /// Minimum main-axis size in pixels.
    pub fn min(mut self, min: Pixels) -> Self {
        self.min_px = Some(min.as_f32());
        self
    }

    /// Maximum main-axis size in pixels.
    pub fn max(mut self, max: Pixels) -> Self {
        self.max_px = Some(max.as_f32());
        self
    }
}

#[derive(Clone)]
pub struct ResizablePanelsModel {
    pub(crate) id: SharedString,
    pub(crate) orientation: ResizablePanelsOrientation,
    pub(crate) frame_width: Option<Pixels>,
    pub(crate) frame_height: Option<Pixels>,
    pub(crate) show_border: bool,
    pub(crate) enabled: bool,
    pub(crate) handle_visibility: ResizeHandleVisibility,
    pub(crate) double_click_collapse: Option<ResizeCollapseBehavior>,
    pub(crate) resize_handle: ResizeHandleSize,
    pub(crate) handle_grip: bool,
    pub(crate) keyboard_step: f32,
    pub(crate) keyboard_shift_step: f32,
    pub(crate) panels: Vec<ResizablePanelSpec>,
    pub(crate) template: Arc<dyn ResizablePanelsTemplate>,
    pub(crate) theme: Arc<dyn ResizablePanelsTheme>,
}

pub struct ResizablePanelsRenderModel<'a> {
    pub id: &'a SharedString,
    pub orientation: ResizablePanelsOrientation,
    pub frame_width: Option<Pixels>,
    pub frame_height: Option<Pixels>,
    pub show_border: bool,
    pub enabled: bool,
    pub handle_visibility: ResizeHandleVisibility,
    pub double_click_collapse: Option<ResizeCollapseBehavior>,
    pub hovered_handle: Option<usize>,
    pub dragging_handle: Option<usize>,
    pub resize_handle: ResizeHandleSize,
    pub handle_grip: bool,
    pub panel_sizes_px: &'a [f32],
    pub panels: &'a [ResizablePanelSpec],
    pub measured_size: Option<gpui::Size<Pixels>>,
}

pub struct ResizablePanelsBuilder {
    pub(crate) model: ResizablePanelsModel,
}

impl ResizablePanelsBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self::with_orientation(id, ResizablePanelsOrientation::Horizontal)
    }

    pub fn horizontal(id: impl Into<SharedString>) -> Self {
        Self::with_orientation(id, ResizablePanelsOrientation::Horizontal)
    }

    pub fn vertical(id: impl Into<SharedString>) -> Self {
        Self::with_orientation(id, ResizablePanelsOrientation::Vertical)
    }

    fn with_orientation(id: impl Into<SharedString>, orientation: ResizablePanelsOrientation) -> Self {
        Self {
            model: ResizablePanelsModel {
                id: id.into(),
                orientation,
                frame_width: None,
                frame_height: None,
                show_border: true,
                enabled: true,
                handle_visibility: ResizeHandleVisibility::Hidden,
                double_click_collapse: None,
                resize_handle: ResizeHandleSize::Sm,
                handle_grip: false,
                keyboard_step: 2.0,
                keyboard_shift_step: 10.0,
                panels: Vec::new(),
                template: default_resizable_panels_template(),
                theme: default_resizable_panels_theme(),
            },
        }
    }

    pub fn orientation(mut self, orientation: ResizablePanelsOrientation) -> Self {
        self.model.orientation = orientation;
        self
    }

    /// Fixed width and height for demos; omit to fill the parent (`size_full`).
    pub fn size(mut self, width: Pixels, height: Pixels) -> Self {
        self.model.frame_width = Some(width);
        self.model.frame_height = Some(height);
        self
    }

    /// Pins horizontal main-axis layout; cross-axis fills the parent when height is unset.
    pub fn width(mut self, width: Pixels) -> Self {
        self.model.frame_width = Some(width);
        self
    }

    /// Pins vertical main-axis layout; cross-axis fills the parent when width is unset.
    pub fn height(mut self, height: Pixels) -> Self {
        self.model.frame_height = Some(height);
        self
    }

    pub fn show_border(mut self, show_border: bool) -> Self {
        self.model.show_border = show_border;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn show_handle(mut self, show_handle: bool) -> Self {
        self.model.handle_visibility = if show_handle {
            ResizeHandleVisibility::Always
        } else {
            ResizeHandleVisibility::Hidden
        };
        self
    }

    pub fn handle_visibility(mut self, visibility: ResizeHandleVisibility) -> Self {
        self.model.handle_visibility = visibility;
        self
    }

    pub fn double_click_collapse(mut self, behavior: Option<ResizeCollapseBehavior>) -> Self {
        self.model.double_click_collapse = behavior;
        self
    }

    pub fn resize_handle(mut self, size: ResizeHandleSize) -> Self {
        self.model.resize_handle = size;
        self
    }

    pub fn handle_grip(mut self, handle_grip: bool) -> Self {
        self.model.handle_grip = handle_grip;
        self
    }

    pub fn keyboard_step(mut self, step: Pixels) -> Self {
        self.model.keyboard_step = step.as_f32().max(0.1);
        self
    }

    pub fn keyboard_shift_step(mut self, step: Pixels) -> Self {
        self.model.keyboard_shift_step = step.as_f32().max(0.1);
        self
    }

    pub fn panel(mut self, panel: ResizablePanelSpec) -> Self {
        self.model.panels.push(panel);
        self
    }

    pub fn panels<I>(mut self, panels: I) -> Self
    where
        I: IntoIterator<Item = ResizablePanelSpec>,
    {
        self.model.panels.extend(panels);
        self
    }

    pub fn template(mut self, template: Arc<dyn ResizablePanelsTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &ResizablePanelsRenderModel<'_>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = modified_resizable_panels_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn theme(mut self, theme: Arc<dyn ResizablePanelsTheme>) -> Self {
        self.model.theme = theme;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ResizablePanels> {
        cx.new(|cx| ResizablePanels::from_builder(self, cx))
    }
}

pub fn render_pane<E, F>(render: F) -> PanelRender
where
    E: IntoElement,
    F: Fn() -> E + 'static,
{
    Rc::new(move || render().into_any_element())
}

#[cfg(test)]
mod resize_handle_tests {
    use std::sync::Arc;

    use super::{
        ResizeCollapseBehavior, ResizeCollapseDirection, ResizeCollapseMode, ResizeHandleSize, ResizeHandleVisibility,
    };
    use super::{ResizablePanelsBuilder, default_resizable_panels_template};

    #[test]
    fn metrics_increase_across_presets() {
        let sm = ResizeHandleSize::Sm.metrics();
        let md = ResizeHandleSize::Md.metrics();
        let lg = ResizeHandleSize::Lg.metrics();
        assert!(sm.lane_px < md.lane_px && md.lane_px < lg.lane_px);
        assert!(sm.hit_target_px <= md.hit_target_px && md.hit_target_px <= lg.hit_target_px);
        assert!(sm.grip_cross_axis_px < md.grip_cross_axis_px && md.grip_cross_axis_px < lg.grip_cross_axis_px);
    }

    #[test]
    fn from_lane_px_maps_widths_to_presets() {
        assert_eq!(ResizeHandleSize::from_lane_px(18.0), ResizeHandleSize::Lg);
        assert_eq!(ResizeHandleSize::from_lane_px(10.0), ResizeHandleSize::Md);
        assert_eq!(ResizeHandleSize::from_lane_px(8.0), ResizeHandleSize::Sm);
    }

    #[test]
    fn show_handle_maps_to_visibility_policy() {
        assert_eq!(
            ResizablePanelsBuilder::new("hidden").show_handle(false).model.handle_visibility,
            ResizeHandleVisibility::Hidden
        );
        assert_eq!(
            ResizablePanelsBuilder::new("always").show_handle(true).model.handle_visibility,
            ResizeHandleVisibility::Always
        );
    }

    #[test]
    fn handle_visibility_sets_policy_directly() {
        let builder = ResizablePanelsBuilder::new("hover").handle_visibility(ResizeHandleVisibility::Hover);
        assert_eq!(builder.model.handle_visibility, ResizeHandleVisibility::Hover);
    }

    #[test]
    fn double_click_collapse_defaults_to_none() {
        let builder = ResizablePanelsBuilder::new("collapse-default");
        assert_eq!(builder.model.double_click_collapse, None);
    }

    #[test]
    fn double_click_collapse_sets_behavior() {
        let behavior = ResizeCollapseBehavior::new(ResizeCollapseMode::ToMinSize, ResizeCollapseDirection::Left);
        let builder = ResizablePanelsBuilder::new("collapse").double_click_collapse(Some(behavior));
        assert_eq!(builder.model.double_click_collapse, Some(behavior));
    }

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_resizable_panels_template();
        let builder = ResizablePanelsBuilder::new("panels-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }
}
