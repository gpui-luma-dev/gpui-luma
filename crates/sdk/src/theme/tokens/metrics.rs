use super::ControlSize;

#[derive(Clone, Copy, Debug)]
pub struct MetricTokens {
    pub spacing: SpacingTokens,
    pub radius: RadiusTokens,
    pub border_width: BorderWidthTokens,
    pub focus: FocusMetricTokens,
    pub control: ControlMetricScale,
}

#[derive(Clone, Copy, Debug)]
pub struct SpacingTokens {
    pub s0: f32,
    pub s1: f32,
    pub s2: f32,
    pub s3: f32,
    pub s4: f32,
    pub s5: f32,
    pub s6: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct RadiusTokens {
    pub none: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub pill: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct BorderWidthTokens {
    pub hairline: f32,
    pub default: f32,
    pub strong: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct FocusMetricTokens {
    pub width: f32,
    pub offset: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlMetricScale {
    pub sm: ControlMetricTokens,
    pub md: ControlMetricTokens,
    pub lg: ControlMetricTokens,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlMetricTokens {
    pub radius: f32,
    pub height: f32,
    pub control_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub icon_size: f32,
}

impl Default for MetricTokens {
    fn default() -> Self {
        let sm = ControlMetricTokens::new(28.0, 10.0, 5.0, 6.0, 5.0, 14.0);
        let md = ControlMetricTokens::new(36.0, 14.0, 8.0, 8.0, 6.0, 16.0);
        let lg = ControlMetricTokens::new(44.0, 18.0, 10.0, 10.0, 8.0, 18.0);

        Self {
            spacing: SpacingTokens { s0: 0.0, s1: 4.0, s2: 6.0, s3: 8.0, s4: 12.0, s5: 16.0, s6: 24.0 },
            radius: RadiusTokens { none: 0.0, sm: 3.0, md: 6.0, lg: 8.0, xl: 12.0, pill: 999.0 },
            border_width: BorderWidthTokens { hairline: 0.5, default: 1.0, strong: 2.0 },
            focus: FocusMetricTokens { width: 1.0, offset: 0.0 },
            control: ControlMetricScale { sm, md, lg },
        }
    }
}

impl ControlMetricTokens {
    pub fn new(height: f32, padding_x: f32, padding_y: f32, gap: f32, radius: f32, icon_size: f32) -> Self {
        Self { radius, height, control_height: height, padding_x, padding_y, gap, icon_size }
    }
}

impl MetricTokens {
    pub fn for_size(&self, size: ControlSize) -> ControlMetricTokens {
        match size {
            ControlSize::Sm => self.control.sm,
            ControlSize::Md => self.control.md,
            ControlSize::Lg => self.control.lg,
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

    pub fn icon_size(&self, size: ControlSize) -> f32 {
        self.for_size(size).icon_size
    }
}
