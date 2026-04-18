use gpui::{Hsla, hsla};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlSize {
    Sm,
    #[default]
    Md,
    Lg,
}

#[derive(Clone, Debug, Default)]
pub struct ThemeTokens {
    pub colors: ColorTokens,
    pub metrics: MetricTokens,
}

#[derive(Clone, Debug)]
pub struct ColorTokens {
    pub surface: Hsla,
    pub surface_hover: Hsla,
    pub surface_pressed: Hsla,
    pub surface_disabled: Hsla,
    pub primary: Hsla,
    pub primary_hover: Hsla,
    pub primary_pressed: Hsla,
    pub destructive: Hsla,
    pub destructive_hover: Hsla,
    pub destructive_pressed: Hsla,
    pub selected: Hsla,
    pub selected_hover: Hsla,
    pub selected_pressed: Hsla,
    pub text: Hsla,
    pub text_inverse: Hsla,
    pub text_disabled: Hsla,
    pub border: Hsla,
    pub focus_ring: Hsla,
}

#[derive(Clone, Debug)]
pub struct MetricTokens {
    pub sm: ControlMetricTokens,
    pub md: ControlMetricTokens,
    pub lg: ControlMetricTokens,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlMetricTokens {
    pub radius: f32,
    pub control_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
}

impl Default for ColorTokens {
    fn default() -> Self {
        Self {
            surface: hsla(0.60, 0.12, 0.94, 1.0),
            surface_hover: hsla(0.60, 0.16, 0.90, 1.0),
            surface_pressed: hsla(0.60, 0.18, 0.84, 1.0),
            surface_disabled: hsla(0.60, 0.08, 0.88, 1.0),
            primary: hsla(0.58, 0.84, 0.48, 1.0),
            primary_hover: hsla(0.58, 0.86, 0.42, 1.0),
            primary_pressed: hsla(0.58, 0.88, 0.35, 1.0),
            destructive: hsla(0.01, 0.74, 0.50, 1.0),
            destructive_hover: hsla(0.01, 0.76, 0.44, 1.0),
            destructive_pressed: hsla(0.01, 0.78, 0.37, 1.0),
            selected: hsla(0.44, 0.68, 0.36, 1.0),
            selected_hover: hsla(0.44, 0.70, 0.31, 1.0),
            selected_pressed: hsla(0.44, 0.72, 0.26, 1.0),
            text: hsla(0.61, 0.22, 0.14, 1.0),
            text_inverse: hsla(0.0, 0.0, 1.0, 1.0),
            text_disabled: hsla(0.60, 0.08, 0.45, 1.0),
            border: hsla(0.60, 0.12, 0.76, 1.0),
            focus_ring: hsla(0.12, 0.92, 0.55, 1.0),
        }
    }
}

impl Default for MetricTokens {
    fn default() -> Self {
        Self {
            sm: ControlMetricTokens { radius: 5.0, control_height: 28.0, padding_x: 10.0, padding_y: 5.0, gap: 6.0 },
            md: ControlMetricTokens { radius: 6.0, control_height: 36.0, padding_x: 14.0, padding_y: 8.0, gap: 8.0 },
            lg: ControlMetricTokens { radius: 7.0, control_height: 44.0, padding_x: 18.0, padding_y: 10.0, gap: 10.0 },
        }
    }
}

impl MetricTokens {
    pub fn for_size(&self, size: ControlSize) -> ControlMetricTokens {
        match size {
            ControlSize::Sm => self.sm,
            ControlSize::Md => self.md,
            ControlSize::Lg => self.lg,
        }
    }

    pub fn radius(&self, size: ControlSize) -> f32 {
        self.for_size(size).radius
    }

    pub fn control_height(&self, size: ControlSize) -> f32 {
        self.for_size(size).control_height
    }

    pub fn padding_x(&self, size: ControlSize) -> f32 {
        self.for_size(size).padding_x
    }

    pub fn padding_y(&self, size: ControlSize) -> f32 {
        self.for_size(size).padding_y
    }

    pub fn gap(&self, size: ControlSize) -> f32 {
        self.for_size(size).gap
    }
}
