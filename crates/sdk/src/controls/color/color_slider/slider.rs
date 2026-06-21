pub use super::color_thumb::ThumbShape;
use super::color_thumb::bar_main_axis_size;
use super::model::ColorSliderModel;
pub use super::surface::ColorSlider;
use crate::controls::slider::SliderThumbSize as SemanticThumbSize;
use crate::theme::ControlSize;
use gpui::{prelude::*, *};

pub mod sizing {
    pub const TRACK_THICKNESS_XSMALL: f32 = 4.0;
    pub const TRACK_THICKNESS_SMALL: f32 = 14.0;
    pub const TRACK_THICKNESS_MEDIUM: f32 = 24.0;
    pub const TRACK_THICKNESS_LARGE: f32 = 34.0;

    pub const THUMB_SIZE_XSMALL: f32 = 10.0;
    pub const THUMB_SIZE_SMALL: f32 = 12.0;
    pub const THUMB_SIZE_MEDIUM: f32 = 20.0;
    pub const THUMB_SIZE_LARGE: f32 = 30.0;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ColorInterpolation {
    #[default]
    Rgb,
    Hsl,
    Lab,
}

pub trait ColorSliderDelegate: 'static {
    fn style_background(&self, slider: &ColorSliderState, container: Div, window: &mut Window, cx: &App) -> Div;

    /// Get the color at a specific position (0.0 to 1.0) on the slider.
    fn get_color_at_position(&self, slider: &ColorSliderState, position: f32) -> Hsla;

    fn interpolation_method(&self, slider: &ColorSliderState) -> ColorInterpolation {
        slider.interpolation
    }
}

pub(super) fn normalized_value_percent(value: f32, start: f32, end: f32) -> f32 {
    let span = end - start;
    if span.abs() <= f32::EPSILON {
        return 0.0;
    }

    ((value - start) / span).clamp(0.0, 1.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ThumbPosition {
    /// Thumb stays within slider bounds (default behavior)
    #[default]
    InsideSlider,
    /// Thumb centerline aligns with slider ends (half thumb extends past ends)
    EdgeToEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ThumbSize {
    XSmall,
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColorSliderEvent {
    Change(f32),
    Release(f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ThumbConfig {
    pub position: ThumbPosition,
    pub size: ThumbSize,
    pub shape: ThumbShape,
}

pub struct SliderDimensions {
    pub axis: Axis,
    pub size: ControlSize,
    pub bounds: Bounds<Pixels>,
}

impl Default for SliderDimensions {
    fn default() -> Self {
        Self { axis: Axis::Horizontal, size: ControlSize::Md, bounds: Bounds::default() }
    }
}

/// State for [`ColorSlider`].
pub struct ColorSliderState {
    pub id: SharedString,
    pub value: f32,
    pub range: std::ops::Range<f32>,
    pub step: Option<f32>,
    pub reversed: bool,
    pub thumb: ThumbConfig,
    pub dimensions: SliderDimensions,
    pub delegate: Box<dyn ColorSliderDelegate>,
    pub style: StyleRefinement,
    pub interpolation: ColorInterpolation,
    pub disabled: bool,
    pub thumb_size_override: Option<SemanticThumbSize>,
    pub focus_handle: FocusHandle,
    interaction_active: bool,
}

impl ColorSliderState {
    pub fn from_model<V: 'static>(model: ColorSliderModel, cx: &mut Context<V>) -> Self {
        if let Err(err) = model.validate() {
            panic!("invalid ColorSliderModel: {err}");
        }

        let mut this = Self {
            id: model.id,
            value: model.value,
            range: model.range,
            step: model.step,
            reversed: model.reversed,
            thumb: model.thumb,
            dimensions: SliderDimensions { axis: model.axis, size: model.size, bounds: Bounds::default() },
            delegate: model.delegate,
            style: StyleRefinement::default(),
            interpolation: model.interpolation,
            disabled: model.disabled,
            thumb_size_override: model.thumb_size_override,
            focus_handle: cx.focus_handle(),
            interaction_active: false,
        };

        if let Some(radius) = model.corner_radius {
            this.style.corner_radii.top_left = Some(radius);
            this.style.corner_radii.top_right = Some(radius);
            this.style.corner_radii.bottom_left = Some(radius);
            this.style.corner_radii.bottom_right = Some(radius);
        }

        this.value = this.clamp_to_range(this.value);
        this
    }

    pub fn new<V: 'static>(
        id: impl Into<SharedString>,
        value: f32,
        delegate: Box<dyn ColorSliderDelegate>,
        cx: &mut Context<V>,
    ) -> Self {
        Self::from_model(ColorSliderModel::new(id, value, delegate), cx)
    }

    pub fn hue<V: 'static>(id: impl Into<SharedString>, value: f32, cx: &mut Context<V>) -> Self {
        ColorSliderModel::hue(id, value).build(cx)
    }

    pub fn channel<S: super::color_spec::ColorSpecification, V: 'static>(
        id: impl Into<SharedString>,
        value: f32,
        delegate: super::delegates::ChannelDelegate<S>,
        cx: &mut Context<V>,
    ) -> Self {
        ColorSliderModel::channel(id, value, delegate).build(cx)
    }

    pub fn alpha<S: super::color_spec::ColorSpecification, V: 'static>(
        id: impl Into<SharedString>,
        value: f32,
        delegate: super::delegates::AlphaDelegate<S>,
        cx: &mut Context<V>,
    ) -> Self {
        ColorSliderModel::alpha(id, value, delegate).build(cx)
    }

    pub fn gradient<V: 'static>(
        id: impl Into<SharedString>,
        value: f32,
        colors: Vec<Hsla>,
        cx: &mut Context<V>,
    ) -> Self {
        ColorSliderModel::gradient(id, value, colors).build(cx)
    }

    pub fn horizontal(mut self) -> Self {
        self.dimensions.axis = Axis::Horizontal;
        self
    }

    #[allow(dead_code)] // Kept for API parity with Slider orientation builders.
    pub fn vertical(mut self) -> Self {
        self.dimensions.axis = Axis::Vertical;
        self
    }

    pub fn min(mut self, min: f32) -> Self {
        self.range.start = min;
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.range.end = max;
        self
    }

    pub fn set_range(&mut self, min: f32, max: f32, cx: &mut Context<Self>) {
        let range_changed = self.range.start != min || self.range.end != max;
        if range_changed {
            self.range.start = min;
            self.range.end = max;
        }

        let clamped_value = self.clamp_to_range(self.value);
        let value_changed = self.value != clamped_value;
        if value_changed {
            self.value = clamped_value;
        }

        if range_changed || value_changed {
            cx.notify();
        }
    }

    pub fn reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.dimensions.size = size;
        if self.thumb_size_override.is_none() {
            self.thumb.size = Self::synced_thumb_size(self.dimensions.size);
        }
        self
    }

    pub fn thumb_size(mut self, size: SemanticThumbSize) -> Self {
        self.thumb_size_override = Some(size);
        self.thumb.size = Self::semantic_thumb_size(size);
        self
    }

    #[allow(dead_code)] // Runtime setter kept for API parity with builder-based configuration.
    pub fn set_reversed(&mut self, reversed: bool, cx: &mut Context<Self>) {
        if self.reversed != reversed {
            self.reversed = reversed;
            cx.notify();
        }
    }

    #[allow(dead_code)] // Parity helper with other lookless controls.
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        let disabled = !enabled;
        if self.disabled != disabled {
            self.disabled = disabled;
            cx.notify();
        }
    }

    pub fn edge_to_edge(mut self) -> Self {
        self.thumb.position = ThumbPosition::EdgeToEdge;
        self
    }

    pub fn thumb_xsmall(mut self) -> Self {
        self.thumb_size_override = None;
        self.thumb.size = ThumbSize::XSmall;
        self
    }

    pub fn thumb_small(mut self) -> Self {
        self.thumb_size_override = Some(SemanticThumbSize::Sm);
        self.thumb.size = ThumbSize::Small;
        self
    }

    pub fn thumb_medium(mut self) -> Self {
        self.thumb_size_override = Some(SemanticThumbSize::Md);
        self.thumb.size = ThumbSize::Medium;
        self
    }

    pub fn thumb_large(mut self) -> Self {
        self.thumb_size_override = Some(SemanticThumbSize::Lg);
        self.thumb.size = ThumbSize::Large;
        self
    }

    pub fn thumb_square(mut self) -> Self {
        self.thumb.shape = ThumbShape::Square;
        self
    }

    pub fn interpolation(mut self, interpolation: ColorInterpolation) -> Self {
        self.interpolation = interpolation;
        self
    }

    #[allow(dead_code)] // Runtime setter retained for dynamic interpolation switching.
    pub fn set_interpolation(&mut self, interpolation: ColorInterpolation, cx: &mut Context<Self>) {
        if self.interpolation != interpolation {
            self.interpolation = interpolation;
            cx.notify();
        }
    }

    pub fn set_value(&mut self, value: f32, cx: &mut Context<Self>) {
        let clamped_value = self.clamp_to_range(value);
        if self.value != clamped_value {
            self.value = clamped_value;
            cx.notify();
        }
    }

    pub fn set_size(&mut self, size: ControlSize, cx: &mut Context<Self>) {
        let synced_thumb = Self::synced_thumb_size(size);
        let needs_thumb_update = self.thumb_size_override.is_none() && self.thumb.size != synced_thumb;

        if self.dimensions.size != size || needs_thumb_update {
            self.dimensions.size = size;
            if self.thumb_size_override.is_none() {
                self.thumb.size = synced_thumb;
            }
            cx.notify();
        }
    }

    pub fn set_thumb_size(&mut self, size: SemanticThumbSize, cx: &mut Context<Self>) {
        let mapped = Self::semantic_thumb_size(size);
        if self.thumb_size_override != Some(size) || self.thumb.size != mapped {
            self.thumb_size_override = Some(size);
            self.thumb.size = mapped;
            cx.notify();
        }
    }

    pub fn clear_thumb_size(&mut self, cx: &mut Context<Self>) {
        let synced = Self::synced_thumb_size(self.dimensions.size);
        if self.thumb_size_override.is_some() || self.thumb.size != synced {
            self.thumb_size_override = None;
            self.thumb.size = synced;
            cx.notify();
        }
    }

    #[allow(dead_code)] // Runtime setter retained for external thumb style controls.
    pub fn set_thumb_position(&mut self, position: ThumbPosition, cx: &mut Context<Self>) {
        if self.thumb.position != position {
            self.thumb.position = position;
            cx.notify();
        }
    }

    #[allow(dead_code)] // Runtime setter retained for external thumb style controls.
    pub fn set_thumb_shape(&mut self, shape: ThumbShape, cx: &mut Context<Self>) {
        if self.thumb.shape != shape {
            self.thumb.shape = shape;
            cx.notify();
        }
    }

    pub fn set_corner_radius(&mut self, radius: AbsoluteLength, cx: &mut Context<Self>) {
        self.style.corner_radii.top_left = Some(radius);
        self.style.corner_radii.top_right = Some(radius);
        self.style.corner_radii.bottom_left = Some(radius);
        self.style.corner_radii.bottom_right = Some(radius);
        cx.notify();
    }

    #[allow(dead_code)] // Runtime counterpart to `set_corner_radius`, kept for API completeness.
    pub fn clear_corner_radius(&mut self, cx: &mut Context<Self>) {
        let had_custom = self.style.corner_radii.top_left.is_some()
            || self.style.corner_radii.top_right.is_some()
            || self.style.corner_radii.bottom_left.is_some()
            || self.style.corner_radii.bottom_right.is_some();

        if had_custom {
            self.style.corner_radii.top_left = None;
            self.style.corner_radii.top_right = None;
            self.style.corner_radii.bottom_left = None;
            self.style.corner_radii.bottom_right = None;
            cx.notify();
        }
    }

    #[allow(dead_code)] // Runtime setter kept for orientation parity with builder API.
    pub fn set_axis(&mut self, axis: Axis, cx: &mut Context<Self>) {
        if self.dimensions.axis != axis {
            self.dimensions.axis = axis;
            cx.notify();
        }
    }

    pub fn set_delegate(&mut self, delegate: Box<dyn ColorSliderDelegate>, cx: &mut Context<Self>) {
        self.delegate = delegate;
        cx.notify();
    }

    pub fn track_thickness(&self) -> f32 {
        match self.dimensions.size {
            ControlSize::Sm => sizing::TRACK_THICKNESS_SMALL,
            ControlSize::Md => sizing::TRACK_THICKNESS_MEDIUM,
            ControlSize::Lg => sizing::TRACK_THICKNESS_LARGE,
        }
    }

    pub fn thumb_size_px(&self) -> f32 {
        match self.thumb.size {
            ThumbSize::XSmall => sizing::THUMB_SIZE_XSMALL,
            ThumbSize::Small => sizing::THUMB_SIZE_SMALL,
            ThumbSize::Medium => sizing::THUMB_SIZE_MEDIUM,
            ThumbSize::Large => sizing::THUMB_SIZE_LARGE,
        }
    }

    pub fn track_inset(&self) -> f32 {
        match self.effective_thumb_position() {
            ThumbPosition::InsideSlider => 0.0,
            ThumbPosition::EdgeToEdge => self.thumb_main_axis_size() / 2.0,
        }
    }
}

impl ColorSliderState {
    fn synced_thumb_size(size: ControlSize) -> ThumbSize {
        match size {
            ControlSize::Sm => ThumbSize::Small,
            ControlSize::Md => ThumbSize::Medium,
            ControlSize::Lg => ThumbSize::Large,
        }
    }

    fn semantic_thumb_size(size: SemanticThumbSize) -> ThumbSize {
        match size {
            SemanticThumbSize::Sm => ThumbSize::Small,
            SemanticThumbSize::Md => ThumbSize::Medium,
            SemanticThumbSize::Lg => ThumbSize::Large,
        }
    }

    pub(super) fn clamp_to_range(&self, value: f32) -> f32 {
        value.clamp(self.range.start.min(self.range.end), self.range.end.max(self.range.start))
    }

    pub(super) fn effective_thumb_position(&self) -> ThumbPosition {
        let hint = self.thumb.shape.layout_hint();
        if hint.supported_positions.contains(&self.thumb.position) {
            self.thumb.position
        } else {
            hint.preferred_position
        }
    }

    pub(super) fn thumb_main_axis_size(&self) -> f32 {
        match self.thumb.shape {
            ThumbShape::Bar => bar_main_axis_size(px(self.thumb_size_px())).as_f32(),
            ThumbShape::Circle | ThumbShape::Square => self.thumb_size_px(),
        }
    }

    pub(super) fn update_from_mouse(&mut self, position: Point<Pixels>, _window: &mut Window, cx: &mut Context<Self>) {
        let size = if self.dimensions.axis == Axis::Vertical {
            self.dimensions.bounds.size.height
        } else {
            self.dimensions.bounds.size.width
        };

        if size <= px(0.0) {
            return;
        }

        let inset = px(self.track_inset());
        let track_length = size - inset * 2.0;

        if track_length <= px(0.0) {
            return;
        }

        let local_pos = if self.dimensions.axis == Axis::Vertical {
            position.y - self.dimensions.bounds.origin.y
        } else {
            position.x - self.dimensions.bounds.origin.x
        };

        let mut percentage = ((local_pos - inset) / track_length).clamp(0.0, 1.0);

        if self.reversed {
            percentage = 1.0 - percentage;
        }

        let mut value = self.range.start + (self.range.end - self.range.start) * percentage;

        if let Some(step) = self.step {
            let step = step.abs();
            if step > f32::EPSILON {
                // Snap relative to the configured range start, not absolute zero.
                value = self.range.start + ((value - self.range.start) / step).round() * step;
            }
        }

        self.value = value.clamp(self.range.start.min(self.range.end), self.range.end.max(self.range.start));
        cx.emit(ColorSliderEvent::Change(self.value));
        cx.notify();
    }

    fn emit_release(&self, cx: &mut Context<Self>) {
        cx.emit(ColorSliderEvent::Release(self.value));
    }

    pub(super) fn begin_interaction(&mut self) {
        self.interaction_active = true;
    }

    pub(super) fn end_interaction(&mut self, cx: &mut Context<Self>) {
        if !self.interaction_active {
            return;
        }
        self.interaction_active = false;
        self.emit_release(cx);
    }
}

impl Styled for ColorSliderState {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl EventEmitter<ColorSliderEvent> for ColorSliderState {}

impl Render for ColorSliderState {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        super::surface::ColorSlider::new(&cx.entity())
    }
}

#[cfg(test)]
mod tests {
    use super::normalized_value_percent;

    fn approx_eq(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-6, "expected {a} ~= {b}");
    }

    #[test]
    fn normalized_value_percent_handles_regular_and_degenerate_ranges() {
        approx_eq(normalized_value_percent(25.0, 0.0, 100.0), 0.25);
        approx_eq(normalized_value_percent(-10.0, 0.0, 100.0), 0.0);
        approx_eq(normalized_value_percent(150.0, 0.0, 100.0), 1.0);
        approx_eq(normalized_value_percent(10.0, 5.0, 5.0), 0.0);
    }

    #[test]
    fn normalized_value_percent_supports_descending_ranges() {
        approx_eq(normalized_value_percent(1.0, 1.0, 0.0), 0.0);
        approx_eq(normalized_value_percent(0.5, 1.0, 0.0), 0.5);
        approx_eq(normalized_value_percent(0.0, 1.0, 0.0), 1.0);
    }
}
