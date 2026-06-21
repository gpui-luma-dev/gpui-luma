use gpui::{AbsoluteLength, Context, Hsla, SharedString};

use crate::controls::slider::SliderThumbSize as SemanticThumbSize;
use crate::theme::ControlSize;

use super::delegates::{AlphaDelegate, ChannelDelegate, GradientDelegate, HueDelegate};
use super::slider::{
    Axis, ColorInterpolation, ColorSliderDelegate, ColorSliderState, ThumbConfig, ThumbPosition, ThumbSize,
};

pub struct ColorSliderModel {
    pub id: SharedString,
    pub value: f32,
    pub range: std::ops::Range<f32>,
    pub step: Option<f32>,
    pub reversed: bool,
    pub thumb: ThumbConfig,
    pub size: ControlSize,
    pub thumb_size_override: Option<SemanticThumbSize>,
    pub axis: Axis,
    pub interpolation: ColorInterpolation,
    pub disabled: bool,
    pub corner_radius: Option<AbsoluteLength>,
    pub delegate: Box<dyn ColorSliderDelegate>,
}

impl ColorSliderModel {
    pub fn new(id: impl Into<SharedString>, value: f32, delegate: Box<dyn ColorSliderDelegate>) -> Self {
        Self {
            id: id.into(),
            value,
            range: 0.0..1.0,
            step: None,
            reversed: false,
            thumb: ThumbConfig::default(),
            size: ControlSize::Md,
            thumb_size_override: None,
            axis: Axis::Horizontal,
            interpolation: ColorInterpolation::default(),
            disabled: false,
            corner_radius: None,
            delegate,
        }
    }

    pub fn hue(id: impl Into<SharedString>, value: f32) -> Self {
        Self::new(id, value, Box::new(HueDelegate)).max(360.0)
    }

    pub fn channel<S: super::color_spec::ColorSpecification>(
        id: impl Into<SharedString>,
        value: f32,
        delegate: ChannelDelegate<S>,
    ) -> Self {
        Self::new(id, value, Box::new(delegate))
    }

    pub fn alpha<S: super::color_spec::ColorSpecification>(
        id: impl Into<SharedString>,
        value: f32,
        delegate: AlphaDelegate<S>,
    ) -> Self {
        Self::new(id, value, Box::new(delegate))
    }

    pub fn gradient(id: impl Into<SharedString>, value: f32, colors: Vec<Hsla>) -> Self {
        Self::new(id, value, Box::new(GradientDelegate { colors }))
    }

    pub fn horizontal(mut self) -> Self {
        self.axis = Axis::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.axis = Axis::Vertical;
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

    pub fn step(mut self, step: Option<f32>) -> Self {
        self.step = step.map(f32::abs);
        self
    }

    pub fn reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        if self.thumb_size_override.is_none() {
            self.thumb.size = thumb_size_from_control_size(size);
        }
        self
    }

    pub fn thumb_size(mut self, size: SemanticThumbSize) -> Self {
        self.thumb_size_override = Some(size);
        self.thumb.size = thumb_size_from_semantic(size);
        self
    }

    pub fn clear_thumb_size(mut self) -> Self {
        self.thumb_size_override = None;
        self.thumb.size = thumb_size_from_control_size(self.size);
        self
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

    pub fn thumb_shape(mut self, shape: super::color_thumb::ThumbShape) -> Self {
        self.thumb.shape = shape;
        self
    }

    pub fn interpolation(mut self, interpolation: ColorInterpolation) -> Self {
        self.interpolation = interpolation;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.disabled = !enabled;
        self
    }

    pub fn corner_radius(mut self, radius: AbsoluteLength) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    pub fn clear_corner_radius(mut self) -> Self {
        self.corner_radius = None;
        self
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.value.is_finite() {
            return Err("value must be finite");
        }
        if !self.range.start.is_finite() || !self.range.end.is_finite() {
            return Err("range bounds must be finite");
        }
        if let Some(step) = self.step
            && (!step.is_finite() || step <= f32::EPSILON)
        {
            return Err("step must be finite and greater than zero");
        }
        Ok(())
    }

    pub fn build<V: 'static>(self, cx: &mut Context<V>) -> ColorSliderState {
        ColorSliderState::from_model(self, cx)
    }
}

fn thumb_size_from_control_size(size: ControlSize) -> ThumbSize {
    match size {
        ControlSize::Sm => ThumbSize::Small,
        ControlSize::Md => ThumbSize::Medium,
        ControlSize::Lg => ThumbSize::Large,
    }
}

fn thumb_size_from_semantic(size: SemanticThumbSize) -> ThumbSize {
    match size {
        SemanticThumbSize::Sm => ThumbSize::Small,
        SemanticThumbSize::Md => ThumbSize::Medium,
        SemanticThumbSize::Lg => ThumbSize::Large,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_non_finite_value_and_step() {
        let invalid_value = ColorSliderModel::hue("invalid-value", f32::NAN);
        assert!(invalid_value.validate().is_err());

        let invalid_step = ColorSliderModel::hue("invalid-step", 180.0).step(Some(0.0));
        assert!(invalid_step.validate().is_err());

        let valid = ColorSliderModel::hue("valid", 180.0).step(Some(1.0));
        assert!(valid.validate().is_ok());
    }
}
