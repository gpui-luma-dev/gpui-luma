use gpui::{Hsla, hsla};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

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
pub struct ThemeModes {
    pub light: ThemeTokens,
    pub dark: ThemeTokens,
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

impl ThemeTokens {
    pub fn light() -> Self {
        Self { colors: ColorTokens::light(), metrics: MetricTokens::default() }
    }

    pub fn dark() -> Self {
        Self { colors: ColorTokens::dark(), metrics: MetricTokens::default() }
    }
}

impl ThemeModes {
    pub fn new(light: ThemeTokens, dark: ThemeTokens) -> Self {
        Self { light, dark }
    }

    pub fn single(tokens: ThemeTokens) -> Self {
        Self { light: tokens.clone(), dark: tokens }
    }

    pub fn tokens(&self, mode: ThemeMode) -> &ThemeTokens {
        match mode {
            ThemeMode::Light => &self.light,
            ThemeMode::Dark => &self.dark,
        }
    }

    pub fn tokens_mut(&mut self, mode: ThemeMode) -> &mut ThemeTokens {
        match mode {
            ThemeMode::Light => &mut self.light,
            ThemeMode::Dark => &mut self.dark,
        }
    }

    pub fn into_tokens(self, mode: ThemeMode) -> ThemeTokens {
        match mode {
            ThemeMode::Light => self.light,
            ThemeMode::Dark => self.dark,
        }
    }
}

impl Default for ThemeModes {
    fn default() -> Self {
        Self { light: ThemeTokens::light(), dark: ThemeTokens::dark() }
    }
}

impl Default for ColorTokens {
    fn default() -> Self {
        Self::light()
    }
}

impl ColorTokens {
    pub fn light() -> Self {
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

    pub fn dark() -> Self {
        Self {
            surface: hsla(0.60, 0.12, 0.16, 1.0),
            surface_hover: hsla(0.60, 0.14, 0.21, 1.0),
            surface_pressed: hsla(0.60, 0.16, 0.26, 1.0),
            surface_disabled: hsla(0.60, 0.08, 0.13, 1.0),
            primary: hsla(0.58, 0.84, 0.58, 1.0),
            primary_hover: hsla(0.58, 0.86, 0.64, 1.0),
            primary_pressed: hsla(0.58, 0.88, 0.70, 1.0),
            destructive: hsla(0.01, 0.74, 0.58, 1.0),
            destructive_hover: hsla(0.01, 0.76, 0.64, 1.0),
            destructive_pressed: hsla(0.01, 0.78, 0.70, 1.0),
            selected: hsla(0.44, 0.68, 0.46, 1.0),
            selected_hover: hsla(0.44, 0.70, 0.52, 1.0),
            selected_pressed: hsla(0.44, 0.72, 0.58, 1.0),
            text: hsla(0.60, 0.10, 0.92, 1.0),
            text_inverse: hsla(0.0, 0.0, 1.0, 1.0),
            text_disabled: hsla(0.60, 0.08, 0.56, 1.0),
            border: hsla(0.60, 0.10, 0.30, 1.0),
            focus_ring: hsla(0.12, 0.92, 0.62, 1.0),
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

#[cfg(test)]
mod tests {
    use super::{ColorTokens, ThemeMode, ThemeModes, ThemeTokens};

    #[test]
    fn default_tokens_are_light_tokens() {
        assert_eq!(ThemeTokens::default().colors.surface, ColorTokens::light().surface);
    }

    #[test]
    fn theme_modes_select_the_requested_token_set() {
        let modes = ThemeModes::default();

        assert_eq!(modes.tokens(ThemeMode::Light).colors.surface, ThemeTokens::light().colors.surface);
        assert_eq!(modes.tokens(ThemeMode::Dark).colors.surface, ThemeTokens::dark().colors.surface);
        assert_ne!(modes.tokens(ThemeMode::Light).colors.surface, modes.tokens(ThemeMode::Dark).colors.surface);
    }
}
