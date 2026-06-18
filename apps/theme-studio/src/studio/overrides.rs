use std::collections::HashMap;

use gpui::Hsla;
use gpui_luma_look_shadcn::ShadcnLook;

pub const RADIUS_REM_MIN: f32 = 0.0;
pub const RADIUS_REM_MAX: f32 = 5.0;
pub const SPACING_REM_MIN: f32 = 0.0;
pub const SPACING_REM_MAX: f32 = 0.35;
pub const SHADOW_OPACITY_MIN: f32 = 0.0;
pub const SHADOW_OPACITY_MAX: f32 = 1.0;
pub const SHADOW_BLUR_MIN: f32 = 0.0;
pub const SHADOW_BLUR_MAX: f32 = 32.0;
pub const SHADOW_SPREAD_MIN: f32 = -12.0;
pub const SHADOW_SPREAD_MAX: f32 = 20.0;
pub const SHADOW_OFFSET_X_MIN: f32 = -20.0;
pub const SHADOW_OFFSET_X_MAX: f32 = 20.0;
pub const SHADOW_OFFSET_Y_MIN: f32 = -8.0;
pub const SHADOW_OFFSET_Y_MAX: f32 = 24.0;

#[derive(Clone, Debug, PartialEq)]
pub struct ThemeShadowOverride {
    pub color: Hsla,
    pub blur_px: f32,
    pub spread_px: f32,
    pub offset_x_px: f32,
    pub offset_y_px: f32,
}

impl ThemeShadowOverride {
    pub fn opacity(&self) -> f32 {
        self.color.a
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.color.a = clamp_shadow_opacity(opacity);
    }

    pub fn to_css_value(&self) -> String {
        format!(
            "{}px {}px {}px {}px hsl({} {}% {}% / {})",
            format_shadow_number(self.offset_x_px),
            format_shadow_number(self.offset_y_px),
            format_shadow_number(self.blur_px),
            format_shadow_number(self.spread_px),
            (self.color.h * 360.0).round(),
            (self.color.s * 100.0).round(),
            (self.color.l * 100.0).round(),
            format_shadow_number(self.color.a),
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct StudioOverrides {
    pub global_color_overrides: HashMap<String, Hsla>,
    pub radius_rem: Option<f32>,
    pub spacing_rem: Option<f32>,
    pub shadow: Option<ThemeShadowOverride>,
}

impl StudioOverrides {
    pub fn global_color_override(&self, token: &str) -> Option<Hsla> {
        self.global_color_overrides.get(token).copied()
    }

    pub fn radius_rem(&self) -> Option<f32> {
        self.radius_rem
    }

    pub fn spacing_rem(&self) -> Option<f32> {
        self.spacing_rem
    }

    pub fn shadow_override(&self) -> Option<&ThemeShadowOverride> {
        self.shadow.as_ref()
    }

    pub fn set_global_color(&mut self, token: String, color: Hsla) {
        self.global_color_overrides.insert(token, color);
    }

    pub fn set_radius_rem(&mut self, rem: f32) {
        self.radius_rem = Some(clamp_radius_rem(rem));
    }

    pub fn set_spacing_rem(&mut self, rem: f32) {
        self.spacing_rem = Some(clamp_spacing_rem(rem));
    }

    pub fn set_shadow_override(&mut self, shadow: ThemeShadowOverride) {
        self.shadow = Some(shadow);
    }

    pub fn token_overrides(&self) -> HashMap<String, String> {
        let mut overrides = HashMap::new();
        if let Some(radius_rem) = self.radius_rem {
            overrides.insert("radius".to_string(), rem_css_value(radius_rem));
        }
        if let Some(spacing_rem) = self.spacing_rem {
            overrides.insert("spacing".to_string(), rem_css_value(spacing_rem));
        }
        if let Some(shadow) = &self.shadow {
            overrides.insert("shadow".to_string(), shadow.to_css_value());
        }
        overrides
    }

    pub fn clear_metric_overrides(&mut self) {
        self.radius_rem = None;
        self.spacing_rem = None;
    }

    pub fn clear_shadow_override(&mut self) {
        self.shadow = None;
    }

    pub fn clear_all_overrides(&mut self) {
        self.global_color_overrides.clear();
        self.clear_metric_overrides();
        self.clear_shadow_override();
    }
}

pub fn clamp_radius_rem(rem: f32) -> f32 {
    rem.clamp(RADIUS_REM_MIN, RADIUS_REM_MAX)
}

pub fn clamp_spacing_rem(rem: f32) -> f32 {
    rem.clamp(SPACING_REM_MIN, SPACING_REM_MAX)
}

pub fn clamp_shadow_opacity(opacity: f32) -> f32 {
    opacity.clamp(SHADOW_OPACITY_MIN, SHADOW_OPACITY_MAX)
}

pub fn clamp_shadow_blur(blur_px: f32) -> f32 {
    blur_px.clamp(SHADOW_BLUR_MIN, SHADOW_BLUR_MAX)
}

pub fn clamp_shadow_spread(spread_px: f32) -> f32 {
    spread_px.clamp(SHADOW_SPREAD_MIN, SHADOW_SPREAD_MAX)
}

pub fn clamp_shadow_offset_x(offset_x_px: f32) -> f32 {
    offset_x_px.clamp(SHADOW_OFFSET_X_MIN, SHADOW_OFFSET_X_MAX)
}

pub fn clamp_shadow_offset_y(offset_y_px: f32) -> f32 {
    offset_y_px.clamp(SHADOW_OFFSET_Y_MIN, SHADOW_OFFSET_Y_MAX)
}

pub fn resolved_shadow_override(look: &ShadcnLook, overrides: &StudioOverrides) -> ThemeShadowOverride {
    overrides.shadow.clone().unwrap_or_else(|| default_shadow_override(look))
}

pub fn default_shadow_override(look: &ShadcnLook) -> ThemeShadowOverride {
    if let Ok(mut layers) = look.parse_shadow_token("shadow")
        && let Some(layer) = layers.drain(..).next()
    {
        return ThemeShadowOverride {
            color: layer.color,
            blur_px: layer.blur_radius.as_f32(),
            spread_px: layer.spread_radius.as_f32(),
            offset_x_px: layer.offset.x.as_f32(),
            offset_y_px: layer.offset.y.as_f32(),
        };
    }

    ThemeShadowOverride {
        color: Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.12 },
        blur_px: 10.0,
        spread_px: 0.0,
        offset_x_px: 0.0,
        offset_y_px: 4.0,
    }
}

fn rem_css_value(rem: f32) -> String {
    format!("{:.4}rem", rem)
}

fn format_shadow_number(value: f32) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    let mut text = format!("{rounded:.2}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}
