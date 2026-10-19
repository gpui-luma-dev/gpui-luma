use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::theme::{ControlSize, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct FloatingMenuLook {
    /// Shared popup edge margin in logical pixels.
    pub window_margin: f32,
    /// Positive completion/status adornments.
    pub positive: Hsla,
    /// Popup border thickness in pixels.
    pub border_width: f32,
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub shadow: Vec<BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub min_width: f32,
    pub item_disabled_foreground: Hsla,
    pub item_hover_background: Hsla,
    pub item_hover_foreground: Hsla,
    pub item_typography: LumaTextStyle,
    pub item_height: f32,
    pub item_padding_x: f32,
    pub item_gap: f32,
    pub item_icon_size: f32,
    pub item_radius: f32,
    pub disabled_opacity: f32,
    pub submenu_offset_x: f32,
    /// Divider line color.
    pub separator_color: Hsla,
    /// Divider line thickness in pixels.
    pub separator_thickness: f32,
    /// Vertical padding on each side of the divider, in pixels.
    pub separator_spacing: f32,
    /// Horizontal inset on each side of the divider, in pixels.
    pub separator_inset: f32,
}

impl FloatingMenuLook {
    /// Total row height, including dividers, excluding the panel padding.
    pub fn rows_height(&self, items: &[crate::infra::menu_item::MenuItem]) -> f32 {
        items
            .iter()
            .map(|item| {
                if item.is_separator() {
                    self.separator_thickness + 2.0 * self.separator_spacing
                } else {
                    self.item_height
                }
            })
            .sum()
    }
}

pub trait FloatingMenuTheme: Send + Sync {
    fn resolve(&self) -> FloatingMenuLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultFloatingMenuTheme {
    tokens: ThemeTokens,
}

pub fn default_floating_menu_theme() -> Arc<dyn FloatingMenuTheme> {
    static THEME: OnceLock<Arc<dyn FloatingMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultFloatingMenuTheme::default())).clone()
}

impl DefaultFloatingMenuTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl FloatingMenuTheme for DefaultFloatingMenuTheme {
    fn resolve(&self) -> FloatingMenuLook {
        default_floating_menu_look(&self.tokens, ControlSize::Md)
    }
}

pub(crate) fn default_floating_menu_look(tokens: &ThemeTokens, size: ControlSize) -> FloatingMenuLook {
    let palette = tokens.stylesheet.recipes.palette("floating_menu", &tokens.palette);
    let metrics = &tokens.metrics;
    let typography = &tokens.typography;
    let elevation = &tokens.elevation;
    let item_typography = scaled_menu_item_typography(typography, size);

    let key = size.stylesheet_key();
    let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default().floating_menu.authored_geometry(key);
    let recipe = tokens.stylesheet.floating_menu.authored_geometry(key).with_fallback(&baseline);
    let mut look = FloatingMenuLook {
        window_margin: tokens
            .stylesheet
            .with_sdk_defaults()
            .popover
            .authored_geometry("panel")
            .window_margin
            .unwrap_or_default(),
        positive: tokens.stylesheet.recipes.palette("floating_menu", &tokens.palette).data.positive,
        border_width: recipe.border_width.unwrap_or(metrics.border_width.default),
        background: palette.surface.floating.background,
        foreground: palette.surface.floating.foreground,
        border: palette.surface.floating.border,
        shadow: elevation.menu.to_box_shadows(),
        radius: metrics.radius.lg,
        padding: metrics.padding_y(size) * recipe.padding_scale.unwrap_or_default(),
        min_width: recipe.min_width.unwrap_or_default(),
        item_disabled_foreground: palette.state.disabled.foreground,
        item_hover_background: palette.state.hover.background,
        item_hover_foreground: palette.state.hover.foreground,
        item_typography,
        item_height: metrics.control_height(size) * recipe.item_height_scale.unwrap_or_default(),
        item_padding_x: metrics.padding_x(size) * recipe.item_padding_scale.unwrap_or_default(),
        item_gap: metrics.gap(size),
        item_icon_size: metrics.icon_size(size),
        item_radius: metrics.radius.sm,
        disabled_opacity: recipe.disabled_opacity.unwrap_or_default(),
        submenu_offset_x: metrics.gap(size) * recipe.submenu_offset_scale.unwrap_or_default(),
        separator_color: palette.surface.floating.border,
        separator_thickness: recipe.separator_thickness.unwrap_or_default(),
        separator_spacing: metrics.gap(size) * recipe.submenu_offset_scale.unwrap_or_default(),
        separator_inset: recipe.separator_inset.unwrap_or_default(),
    };
    crate::theme::stylesheet::apply_common_geometry!(tokens.stylesheet.floating_menu, key, FloatingMenuGeometry, look,
        [border_width => border_width, padding => padding, min_width => min_width, item_height => item_height,
         item_padding_x => item_padding_x, item_gap => item_gap, item_icon_size => item_icon_size,
         font_size => item_typography.size, line_height => item_typography.line_height,
         submenu_offset_x => submenu_offset_x, separator_thickness => separator_thickness,
         separator_spacing => separator_spacing, separator_inset => separator_inset,
         disabled_opacity => disabled_opacity], defaults = Default::default());
    look
}

fn scaled_menu_item_typography(typography: &crate::theme::LumaTypography, size: ControlSize) -> LumaTextStyle {
    match size {
        ControlSize::Sm => typography.text.scale.sm,
        ControlSize::Md => typography.text.body,
        ControlSize::Lg => typography.text.scale.lg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_floating_menu_typography_uses_text_tokens() {
        let mut tokens = ThemeTokens::default();
        tokens.typography.text.scale.sm.size = 13.0;
        tokens.typography.text.body.size = 15.0;
        tokens.typography.text.scale.lg.size = 17.0;

        let small = default_floating_menu_look(&tokens, ControlSize::Sm);
        let medium = default_floating_menu_look(&tokens, ControlSize::Md);
        let large = default_floating_menu_look(&tokens, ControlSize::Lg);

        assert_eq!(small.item_typography.size, 13.0);
        assert_eq!(medium.item_typography.size, 15.0);
        assert_eq!(large.item_typography.size, 17.0);
    }

    #[test]
    fn default_floating_menu_icon_size_uses_control_metric() {
        let mut tokens = ThemeTokens::default();
        tokens.typography.text.body.size = 15.5;
        tokens.metrics.control.md.icon_size = 17.0;

        let look = default_floating_menu_look(&tokens, ControlSize::Md);

        assert_eq!(look.item_icon_size, 17.0);
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    FloatingMenuStylesheet,
    FloatingMenuGeometryStylesheet,
    FloatingMenuGeometry,
    ResolvedFloatingMenuGeometry,
    "floating_menu",
    [
        border_width,
        padding,
        min_width,
        item_height,
        item_padding_x,
        item_gap,
        item_icon_size,
        font_size,
        line_height,
        submenu_offset_x,
        padding_scale,
        item_height_scale,
        item_padding_scale,
        submenu_offset_scale,
        separator_thickness,
        separator_spacing,
        separator_inset
    ],
    unit[disabled_opacity]
);

impl FloatingMenuStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: FloatingMenuGeometryStylesheet {
                min_width: Some(180.0),
                padding_scale: Some(0.5),
                item_height_scale: Some(0.9),
                item_padding_scale: Some(0.75),
                submenu_offset_scale: Some(0.5),
                disabled_opacity: Some(0.56),
                separator_thickness: Some(1.0),
                separator_inset: Some(0.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
