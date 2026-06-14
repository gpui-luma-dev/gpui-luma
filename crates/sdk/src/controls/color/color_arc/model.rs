use gpui::{Context, SharedString};

use crate::controls::color::style::Size;

use super::arc::{ColorArcDelegate, ColorArcState};

pub struct ColorArcModel {
    pub id: SharedString,
    pub value: f32,
    pub range: std::ops::Range<f32>,
    pub step: Option<f32>,
    pub reversed: bool,
    pub size: Size,
    pub arc_thickness: Option<f32>,
    pub arc_thickness_size: Option<Size>,
    pub thumb_size: Option<f32>,
    pub delegate: Box<dyn ColorArcDelegate>,
    pub disabled: bool,
    pub start_degrees: f32,
    pub sweep_degrees: f32,
}

impl ColorArcModel {
    pub fn new(id: impl Into<SharedString>, value: f32, delegate: Box<dyn ColorArcDelegate>) -> Self {
        Self {
            id: id.into(),
            value,
            range: 0.0..1.0,
            step: None,
            reversed: false,
            size: Size::Medium,
            arc_thickness: None,
            arc_thickness_size: None,
            thumb_size: None,
            delegate,
            disabled: false,
            start_degrees: 0.0,
            sweep_degrees: 180.0,
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

    pub fn arc_thickness(mut self, arc_thickness: Option<f32>) -> Self {
        self.arc_thickness = arc_thickness.map(|v| v.max(1.0));
        self
    }

    pub fn arc_thickness_size(mut self, arc_thickness_size: Option<Size>) -> Self {
        self.arc_thickness_size = arc_thickness_size;
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

    pub fn start_degrees(mut self, start_degrees: f32) -> Self {
        self.start_degrees = start_degrees;
        self
    }

    pub fn sweep_degrees(mut self, sweep_degrees: f32) -> Self {
        self.sweep_degrees = sweep_degrees.max(0.0);
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

    pub fn build<V: 'static>(self, cx: &mut Context<V>) -> ColorArcState {
        ColorArcState::from_model(self, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::color::color_arc::delegates::HueArcDelegate;

    #[test]
    fn validate_rejects_non_finite_value_and_step() {
        let invalid_value =
            ColorArcModel::new("invalid-value", f32::NAN, Box::new(HueArcDelegate { saturation: 1.0, lightness: 0.5 }));
        assert!(invalid_value.validate().is_err());

        let invalid_step =
            ColorArcModel::new("invalid-step", 0.5, Box::new(HueArcDelegate { saturation: 1.0, lightness: 0.5 }))
                .step(Some(0.0));
        assert!(invalid_step.validate().is_err());

        let valid = ColorArcModel::new("valid", 0.5, Box::new(HueArcDelegate { saturation: 1.0, lightness: 0.5 }))
            .step(Some(0.01));
        assert!(valid.validate().is_ok());
    }
}
