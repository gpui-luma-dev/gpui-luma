use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{ControlSize, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct StepperLook {
    pub complete_bg: Hsla,
    pub complete_fg: Hsla,
    pub in_progress_bg: Hsla,
    pub in_progress_fg: Hsla,
    pub incomplete_bg: Hsla,
    pub incomplete_border: Hsla,
    pub incomplete_fg: Hsla,
    pub track_active_color: Hsla,
    pub track_muted_color: Hsla,
    pub step_badge_size: f32,
    pub track_thickness: f32,
}

pub trait StepperTheme: Send + Sync {
    fn resolve(&self, enabled: bool, size: ControlSize) -> StepperLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultStepperTheme {
    tokens: ThemeTokens,
}

pub fn default_stepper_theme() -> Arc<dyn StepperTheme> {
    static THEME: OnceLock<Arc<dyn StepperTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultStepperTheme::default())).clone()
}

impl DefaultStepperTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl StepperTheme for DefaultStepperTheme {
    fn resolve(&self, enabled: bool, size: ControlSize) -> StepperLook {
        let palette = &self.tokens.palette;

        let track_color = if enabled {
            palette.surface.subtle.background
        } else {
            palette.state.disabled.background
        };
        let progress_color = if enabled {
            palette.state.selected.background
        } else {
            palette.state.disabled.foreground
        };

        let (incomplete_bg, incomplete_border, incomplete_fg) = if enabled {
            (palette.surface.subtle.background, palette.border.default, palette.surface.subtle.foreground)
        } else {
            let disabled = palette.state.disabled.foreground;
            (palette.state.disabled.background, disabled, disabled)
        };

        let (complete_bg, complete_fg, in_progress_bg, in_progress_fg, track_active_color, track_muted_color) =
            if enabled {
                (progress_color, track_color, progress_color, track_color, progress_color, track_color)
            } else {
                let badge_bg = palette.surface.subtle.background;
                (badge_bg, progress_color, badge_bg, progress_color, progress_color, badge_bg)
            };

        StepperLook {
            complete_bg,
            complete_fg,
            in_progress_bg,
            in_progress_fg,
            incomplete_bg,
            incomplete_border,
            incomplete_fg,
            track_active_color,
            track_muted_color,
            step_badge_size: step_badge_size(size),
            track_thickness: track_thickness(size),
        }
    }
}

fn step_badge_size(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 24.0,
        ControlSize::Md => 32.0,
        ControlSize::Lg => 40.0,
    }
}

fn track_thickness(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 2.0,
        ControlSize::Md => 2.0,
        ControlSize::Lg => 3.0,
    }
}
