use gpui::{BoxShadow, Hsla};

use crate::theme::{ControlSize, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage};

#[derive(Clone, Debug)]
pub struct SelectionPanelAppearance {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub shadow: Vec<BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub min_width: f32,
    pub item_disabled_foreground: Hsla,
    pub item_hover_background: Hsla,
    pub item_typography: LumaTextStyle,
    pub item_height: f32,
    pub item_padding_x: f32,
    pub item_gap: f32,
    pub item_icon_size: f32,
    pub item_radius: f32,
}

pub const SELECTION_PANEL_THEME_USAGE: ThemeUsage = ThemeUsage {
    label: "Selection Panel",
    parts: &[
        ThemePartUsage {
            part: "panel background",
            token: "surface.floating.background",
            states: &["default"],
            appearance_fields: &["SelectionPanelAppearance.background"],
        },
        ThemePartUsage {
            part: "panel foreground",
            token: "surface.floating.foreground",
            states: &["default"],
            appearance_fields: &["SelectionPanelAppearance.foreground"],
        },
        ThemePartUsage {
            part: "panel border",
            token: "surface.floating.border",
            states: &["default"],
            appearance_fields: &["SelectionPanelAppearance.border"],
        },
        ThemePartUsage {
            part: "item hover background",
            token: "state.hover.background",
            states: &["hovered", "active", "pressed"],
            appearance_fields: &["SelectionPanelAppearance.item_hover_background"],
        },
        ThemePartUsage {
            part: "item disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &["SelectionPanelAppearance.item_disabled_foreground"],
        },
    ],
};

pub fn selection_panel_theme_usage() -> &'static ThemeUsage {
    &SELECTION_PANEL_THEME_USAGE
}

pub fn default_selection_panel_appearance(tokens: &ThemeTokens, size: ControlSize) -> SelectionPanelAppearance {
    let palette = &tokens.palette;
    let metrics = &tokens.metrics;
    let typography = &tokens.typography;
    let elevation = &tokens.elevation;

    SelectionPanelAppearance {
        background: palette.surface.floating.background,
        foreground: palette.surface.floating.foreground,
        border: palette.surface.floating.border,
        shadow: elevation.menu.to_box_shadows(),
        radius: metrics.radius.lg,
        padding: metrics.padding_y(size) * 0.5,
        min_width: 180.0,
        item_disabled_foreground: palette.state.disabled.foreground,
        item_hover_background: palette.state.hover.background,
        item_typography: typography.text.label,
        item_height: metrics.control_height(size) * 0.9,
        item_padding_x: metrics.padding_x(size) * 0.75,
        item_gap: metrics.gap(size),
        item_icon_size: metrics.control_height(size) * 0.44,
        item_radius: metrics.radius.sm,
    }
}
