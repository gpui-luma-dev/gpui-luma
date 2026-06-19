use std::collections::HashMap;
use std::sync::Arc;

use gpui::Hsla;
use gpui_luma::controls::slider::Slider;
use gpui_luma::controls::textfield::TextField;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::overrides::{ThemePaletteHslOverride, ThemeShadowOverride};

pub(super) const TOKEN_CATEGORIES: &[(&str, &[(&str, &str)])] = &[
    ("BASE", &[("background", "Background"), ("foreground", "Foreground")]),
    ("PRIMARY", &[("primary", "Background"), ("primary-foreground", "Foreground")]),
    ("SECONDARY", &[("secondary", "Background"), ("secondary-foreground", "Foreground")]),
    ("ACCENT", &[("accent", "Background"), ("accent-foreground", "Foreground")]),
    ("CARD", &[("card", "Background"), ("card-foreground", "Foreground")]),
    ("POPOVER", &[("popover", "Background"), ("popover-foreground", "Foreground")]),
    ("MUTED", &[("muted", "Background"), ("muted-foreground", "Foreground")]),
    ("DESTRUCTIVE", &[("destructive", "Background"), ("destructive-foreground", "Foreground")]),
    ("BORDER & INPUT", &[("border", "Border"), ("input", "Input"), ("ring", "Ring")]),
    (
        "CHART",
        &[
            ("chart-1", "Chart 1"),
            ("chart-2", "Chart 2"),
            ("chart-3", "Chart 3"),
            ("chart-4", "Chart 4"),
            ("chart-5", "Chart 5"),
        ],
    ),
    (
        "SIDEBAR",
        &[
            ("sidebar", "Background"),
            ("sidebar-foreground", "Foreground"),
            ("sidebar-primary", "Primary"),
            ("sidebar-primary-foreground", "Primary Foreground"),
            ("sidebar-accent", "Accent"),
            ("sidebar-accent-foreground", "Accent Foreground"),
            ("sidebar-border", "Border"),
            ("sidebar-ring", "Ring"),
        ],
    ),
];

pub(super) const OTHER_CATEGORIES: &[&str] = &["HSL ADJUSTMENTS", "RADIUS", "SPACING", "SHADOW"];
pub(super) const METRIC_STEP_REM: f32 = 0.01;
pub(super) const METRIC_FIELD_WIDTH: f32 = 70.0;
pub(super) const SHADOW_COLOR_SWATCH_SIZE: f32 = 28.0;
pub(super) const SHADOW_COLOR_FIELD_WIDTH: f32 = 250.0;
pub(super) const SHADOW_SECTION_GAP: f32 = 4.0;
pub(super) const DEFAULT_RADIUS_REM: f32 = 0.5;
pub(super) const DEFAULT_SPACING_REM: f32 = 0.25;
pub(super) const REM_IN_PX: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum SidebarTab {
    #[default]
    Colors,
    Typography,
    Other,
}

impl SidebarTab {
    pub(super) fn from_id(id: &str) -> Option<Self> {
        match id {
            "colors" => Some(Self::Colors),
            "typography" => Some(Self::Typography),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

/// Theme token editing state owned by the sidebar (selectors, fields, overrides).
pub(super) struct ThemeSidebarViewModel {
    pub look: Arc<ShadcnLook>,
    pub global_overrides: HashMap<String, Hsla>,
    pub palette_hsl: ThemePaletteHslOverride,
    pub token_fields: HashMap<String, TextField>,
    pub palette_hue_field: TextField,
    pub palette_saturation_field: TextField,
    pub palette_lightness_field: TextField,
    pub palette_hue_slider: Slider,
    pub palette_saturation_slider: Slider,
    pub palette_lightness_slider: Slider,
    pub radius_field: TextField,
    pub spacing_field: TextField,
    pub radius_slider: Slider,
    pub spacing_slider: Slider,
    pub shadow_override: ThemeShadowOverride,
    pub shadow_color_field: TextField,
    pub shadow_opacity_field: TextField,
    pub shadow_blur_field: TextField,
    pub shadow_spread_field: TextField,
    pub shadow_offset_x_field: TextField,
    pub shadow_offset_y_field: TextField,
    pub shadow_opacity_slider: Slider,
    pub shadow_blur_slider: Slider,
    pub shadow_spread_slider: Slider,
    pub shadow_offset_x_slider: Slider,
    pub shadow_offset_y_slider: Slider,
}
