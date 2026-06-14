use gpui::{Context, Hsla, SharedString};

use crate::controls::color::style::Size;

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
    pub axis: Axis,
    pub size: Size,
    pub interpolation: ColorInterpolation,
    pub disabled: bool,
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
            axis: Axis::Horizontal,
            size: Size::Medium,
            interpolation: ColorInterpolation::default(),
            disabled: false,
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

    pub fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }

    pub fn edge_to_edge(mut self) -> Self {
        self.thumb.position = ThumbPosition::EdgeToEdge;
        self
    }

    pub fn thumb_xsmall(mut self) -> Self {
        self.thumb.size = ThumbSize::XSmall;
        self
    }

    pub fn thumb_small(mut self) -> Self {
        self.thumb.size = ThumbSize::Small;
        self
    }

    pub fn thumb_medium(mut self) -> Self {
        self.thumb.size = ThumbSize::Medium;
        self
    }

    pub fn thumb_large(mut self) -> Self {
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
