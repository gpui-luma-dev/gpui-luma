use gpui::{BoxShadow, Hsla};

use crate::theme::{ControlSize, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct SelectionPanelLook {
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
}

pub fn default_selection_panel_appearance(tokens: &ThemeTokens, size: ControlSize) -> SelectionPanelLook {
    let palette = &tokens.palette;
    let metrics = &tokens.metrics;
    let typography = &tokens.typography;
    let elevation = &tokens.elevation;

    SelectionPanelLook {
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
        item_typography: typography.text.label,
        item_height: metrics.control_height(size) * 0.9,
        item_padding_x: metrics.padding_x(size) * 0.75,
        item_gap: metrics.gap(size),
        item_icon_size: metrics.control_height(size) * 0.44,
        item_radius: metrics.radius.sm,
    }
}
