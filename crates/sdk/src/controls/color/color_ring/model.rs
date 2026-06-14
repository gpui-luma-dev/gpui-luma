use gpui::{Context, Hsla, SharedString};

use crate::controls::color::style::Size;

use super::ring::{ColorRingDelegate, ColorRingMouseBehavior, ColorRingState};

pub struct ColorRingModel {
    pub id: SharedString,
    pub value: f32,
    pub range: std::ops::Range<f32>,
    pub step: Option<f32>,
    pub reversed: bool,
    pub size: Size,
    pub ring_thickness: Option<f32>,
    pub ring_thickness_size: Option<Size>,
    pub thumb_size: Option<f32>,
    pub delegate: Box<dyn ColorRingDelegate>,
    pub disabled: bool,
    pub ring_inner_border: bool,
    pub ring_outer_border: bool,
    pub ring_border_color: Option<Hsla>,
    pub rotation_degrees: f32,
    pub allow_inner_target: bool,
    pub mouse_behavior: ColorRingMouseBehavior,
}

impl ColorRingModel {
    pub fn new(id: impl Into<SharedString>, value: f32, delegate: Box<dyn ColorRingDelegate>) -> Self {
        Self {
            id: id.into(),
            value,
            range: 0.0..1.0,
            step: None,
            reversed: false,
            size: Size::Medium,
            ring_thickness: None,
            ring_thickness_size: None,
            thumb_size: None,
            delegate,
            disabled: false,
            ring_inner_border: true,
            ring_outer_border: true,
            ring_border_color: None,
            rotation_degrees: 0.0,
            allow_inner_target: false,
            mouse_behavior: ColorRingMouseBehavior::default(),
        }
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

    pub fn ring_thickness(mut self, ring_thickness: Option<f32>) -> Self {
        self.ring_thickness = ring_thickness.map(|v| v.max(1.0));
        self
    }

    pub fn ring_thickness_size(mut self, ring_thickness_size: Option<Size>) -> Self {
        self.ring_thickness_size = ring_thickness_size;
        self
    }

    pub fn thumb_size(mut self, thumb_size: Option<f32>) -> Self {
        self.thumb_size = thumb_size.map(|v| v.max(4.0));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn ring_inner_border(mut self, enabled: bool) -> Self {
        self.ring_inner_border = enabled;
        self
    }

    pub fn ring_outer_border(mut self, enabled: bool) -> Self {
        self.ring_outer_border = enabled;
        self
    }

    pub fn ring_border_color(mut self, color: Option<Hsla>) -> Self {
        self.ring_border_color = color;
        self
    }

    pub fn rotation_degrees(mut self, rotation_degrees: f32) -> Self {
        self.rotation_degrees = rotation_degrees;
        self
    }

    pub fn allow_inner_target(mut self, allow_inner_target: bool) -> Self {
        self.allow_inner_target = allow_inner_target;
        self
    }

    pub fn mouse_behavior(mut self, mouse_behavior: ColorRingMouseBehavior) -> Self {
        self.mouse_behavior = mouse_behavior;
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

    pub fn build<V: 'static>(self, cx: &mut Context<V>) -> ColorRingState {
        ColorRingState::from_model(self, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::color::color_ring::delegates::HueRingDelegate;

    #[test]
    fn validate_rejects_non_finite_value_and_step() {
        let invalid_value = ColorRingModel::new(
            "invalid-value",
            f32::INFINITY,
            Box::new(HueRingDelegate { saturation: 1.0, lightness: 0.5 }),
        );
        assert!(invalid_value.validate().is_err());

        let invalid_step =
            ColorRingModel::new("invalid-step", 0.5, Box::new(HueRingDelegate { saturation: 1.0, lightness: 0.5 }))
                .step(Some(0.0));
        assert!(invalid_step.validate().is_err());

        let valid = ColorRingModel::new("valid", 0.5, Box::new(HueRingDelegate { saturation: 1.0, lightness: 0.5 }))
            .step(Some(0.01));
        assert!(valid.validate().is_ok());
    }
}
