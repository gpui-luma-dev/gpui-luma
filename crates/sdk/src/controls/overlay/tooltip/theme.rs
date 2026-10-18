use std::sync::Arc;
use gpui::{BoxShadow, Hsla, SharedString};
use crate::theme::ThemeTokens;

/// Shared tooltip placement and pointer dimensions, in logical pixels.
#[derive(Clone, Copy)]
pub struct TooltipPlacementGeometry {
    pub gap: f32,
    pub margin: f32,
    pub pointer_size: f32,
    pub pointer_inset: f32,
}
impl From<&crate::theme::stylesheet::ResolvedTooltipGeometry> for TooltipPlacementGeometry {
    fn from(value: &crate::theme::stylesheet::ResolvedTooltipGeometry) -> Self {
        Self {
            gap: value.target_gap.value_px,
            margin: value.window_margin.value_px,
            pointer_size: value.pointer_size.value_px,
            pointer_inset: value.pointer_inset.value_px,
        }
    }
}
impl Default for TooltipPlacementGeometry {
    fn default() -> Self {
        (&crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .tooltip
            .resolve_geometry("", Default::default()))
            .into()
    }
}

/// Resolved tooltip style. Looks own all ordinary presentation defaults.
#[derive(Clone)]
pub struct TooltipLook {
    pub placement: TooltipPlacementGeometry,
    /// Shortcut spacing, typography and de-emphasis from the shared stylesheet.
    pub shortcut_gap: f32,
    pub shortcut_font_size: f32,
    pub shortcut_opacity: f32,
    pub background: Hsla,
    pub foreground: Hsla,
    pub padding: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub max_width: f32,
    pub text_size: f32,
    pub line_height: f32,
    pub shadow: Vec<BoxShadow>,
}

impl TooltipLook {
    /// Shared compact geometry for look adapters. Colors and corner radius remain
    /// look-owned; adapters may override any resolved field before returning it.
    pub fn compact(background: Hsla, foreground: Hsla, radius: f32) -> Self {
        let geometry = crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .tooltip
            .resolve_geometry("compact", crate::theme::stylesheet::TooltipGeometry::default());
        Self {
            placement: (&geometry).into(),
            shortcut_gap: geometry.shortcut_gap.value_px,
            shortcut_font_size: geometry.shortcut_font_size.value_px,
            shortcut_opacity: geometry.shortcut_opacity.value_px,
            background,
            foreground,
            radius,
            padding: geometry.padding.value_px,
            padding_y: geometry.padding_y.value_px,
            max_width: geometry.max_width.value_px,
            text_size: geometry.font_size.value_px,
            line_height: geometry.line_height.value_px,
            shadow: Vec::new(),
        }
    }
}

pub trait TooltipTheme: Send + Sync {
    fn resolve(&self) -> TooltipLook;
}

impl TooltipTheme for TooltipLook {
    fn resolve(&self) -> TooltipLook {
        self.clone()
    }
}

impl TooltipLook {
    /// Capture complete tooltip styling from a resolved SDK/common token snapshot.
    pub fn from_tokens(tokens: &ThemeTokens, key: &str) -> Self {
        let geometry = tokens
            .stylesheet
            .tooltip
            .resolve_geometry(key, crate::theme::stylesheet::TooltipGeometry::default());
        let palette = tokens.stylesheet.recipes.palette("tooltip", &tokens.palette);
        Self {
            placement: (&geometry).into(),
            shortcut_gap: geometry.shortcut_gap.value_px,
            shortcut_font_size: geometry.shortcut_font_size.value_px,
            shortcut_opacity: geometry.shortcut_opacity.value_px,
            background: palette.surface.floating.background,
            foreground: palette.surface.floating.foreground,
            padding: geometry.padding.value_px,
            padding_y: geometry.padding_y.value_px,
            radius: tokens.metrics.radius.md,
            max_width: geometry.max_width.value_px,
            text_size: geometry.font_size.value_px,
            line_height: geometry.line_height.value_px,
            shadow: tokens.elevation.menu.to_box_shadows(),
        }
    }
}

pub fn default_tooltip_theme() -> Arc<dyn TooltipTheme> {
    Arc::new(TooltipLook::from_tokens(&ThemeTokens::default(), "md"))
}

/// Presentation-only input, resolved from the explicitly bound draft look on every frame.
#[derive(Clone)]
pub struct TooltipRenderModel {
    pub placement: TooltipPlacementGeometry,
    pub shortcut_gap: f32,
    pub shortcut_font_size: f32,
    pub shortcut_opacity: f32,
    pub text: SharedString,
    pub shortcut: Option<SharedString>,
    pub background: Hsla,
    pub foreground: Hsla,
    pub padding: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub max_width: f32,
    pub text_size: f32,
    pub line_height: f32,
    pub shadow: Vec<BoxShadow>,
}
