use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::theme::{ControlSize, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct FloatingMenuLook {
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
    let palette = &tokens.palette;
    let metrics = &tokens.metrics;
    let typography = &tokens.typography;
    let elevation = &tokens.elevation;
    let item_typography = scaled_menu_item_typography(typography, size);

    FloatingMenuLook {
        background: palette.surface.floating.background,
        foreground: palette.surface.floating.foreground,
        border: palette.surface.floating.border,
        shadow: elevation.menu.to_box_shadows(),
        radius: metrics.radius.lg,
        padding: metrics.padding_y(size) * 0.5,
        min_width: 180.0,
        item_disabled_foreground: palette.state.disabled.foreground,
        item_hover_background: palette.state.hover.background,
        item_hover_foreground: palette.state.hover.foreground,
        item_typography,
        item_height: metrics.control_height(size) * 0.9,
        item_padding_x: metrics.padding_x(size) * 0.75,
        item_gap: metrics.gap(size),
        item_icon_size: metrics.icon_size(size),
        item_radius: metrics.radius.sm,
        disabled_opacity: 0.56,
        submenu_offset_x: metrics.gap(size) * 0.5,
        separator_color: palette.surface.floating.border,
        separator_thickness: 1.0,
        separator_spacing: metrics.gap(size) * 0.5,
        separator_inset: 0.0,
    }
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
