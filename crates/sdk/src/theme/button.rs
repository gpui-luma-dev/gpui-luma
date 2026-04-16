use gpui::{Hsla, hsla};

use crate::controls::button::{ButtonKind, ButtonSize, ButtonState};

#[derive(Clone, Copy, Debug)]
pub struct ButtonAppearance {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub focus_ring: Option<Hsla>,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
}

pub trait ButtonTheme: Send + Sync {
    fn resolve(&self, kind: ButtonKind, size: ButtonSize, state: ButtonState) -> ButtonAppearance;
}

#[derive(Clone, Debug)]
pub struct ButtonThemePack {
    colors: ThemeColors,
    metrics: ThemeMetrics,
}

#[derive(Clone, Debug)]
struct ThemeColors {
    surface: Hsla,
    surface_hover: Hsla,
    surface_pressed: Hsla,
    surface_disabled: Hsla,
    primary: Hsla,
    primary_hover: Hsla,
    primary_pressed: Hsla,
    destructive: Hsla,
    destructive_hover: Hsla,
    destructive_pressed: Hsla,
    text: Hsla,
    text_inverse: Hsla,
    text_disabled: Hsla,
    border: Hsla,
    focus_ring: Hsla,
}

#[derive(Clone, Debug)]
struct ThemeMetrics;

impl Default for ButtonThemePack {
    fn default() -> Self {
        Self {
            colors: ThemeColors {
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
                text: hsla(0.61, 0.22, 0.14, 1.0),
                text_inverse: hsla(0.0, 0.0, 1.0, 1.0),
                text_disabled: hsla(0.60, 0.08, 0.45, 1.0),
                border: hsla(0.60, 0.12, 0.76, 1.0),
                focus_ring: hsla(0.12, 0.92, 0.55, 1.0),
            },
            metrics: ThemeMetrics,
        }
    }
}

impl ButtonTheme for ButtonThemePack {
    fn resolve(&self, kind: ButtonKind, size: ButtonSize, state: ButtonState) -> ButtonAppearance {
        let background = match (kind, state.disabled, state.pressed, state.hovered) {
            (_, true, _, _) => self.colors.surface_disabled,
            (ButtonKind::Primary, false, true, _) => self.colors.primary_pressed,
            (ButtonKind::Primary, false, false, true) => self.colors.primary_hover,
            (ButtonKind::Primary, false, false, false) => self.colors.primary,
            (ButtonKind::Destructive, false, true, _) => self.colors.destructive_pressed,
            (ButtonKind::Destructive, false, false, true) => self.colors.destructive_hover,
            (ButtonKind::Destructive, false, false, false) => self.colors.destructive,
            (_, false, true, _) => self.colors.surface_pressed,
            (_, false, false, true) => self.colors.surface_hover,
            _ => self.colors.surface,
        };

        let foreground = match (kind, state.disabled) {
            (_, true) => self.colors.text_disabled,
            (ButtonKind::Default, false) => self.colors.text,
            _ => self.colors.text_inverse,
        };

        ButtonAppearance {
            background,
            foreground,
            border: self.colors.border,
            focus_ring: state.focused.then_some(self.colors.focus_ring),
            radius: self.metrics.radius(size),
            padding_x: self.metrics.padding_x(size),
            padding_y: self.metrics.padding_y(size),
            gap: self.metrics.gap(size),
        }
    }
}

impl ThemeMetrics {
    fn radius(&self, size: ButtonSize) -> f32 {
        match size {
            ButtonSize::Sm => 5.0,
            ButtonSize::Md => 6.0,
            ButtonSize::Lg => 7.0,
        }
    }

    fn padding_x(&self, size: ButtonSize) -> f32 {
        match size {
            ButtonSize::Sm => 10.0,
            ButtonSize::Md => 14.0,
            ButtonSize::Lg => 18.0,
        }
    }

    fn padding_y(&self, size: ButtonSize) -> f32 {
        match size {
            ButtonSize::Sm => 5.0,
            ButtonSize::Md => 8.0,
            ButtonSize::Lg => 10.0,
        }
    }

    fn gap(&self, size: ButtonSize) -> f32 {
        match size {
            ButtonSize::Sm => 6.0,
            ButtonSize::Md => 8.0,
            ButtonSize::Lg => 10.0,
        }
    }
}
