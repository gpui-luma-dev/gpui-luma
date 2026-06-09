//! Selection panel — same popover list surface as selector items panel.

use gpui_luma::controls::selection_panel::SelectionPanelAppearance;
use gpui_luma::controls::selector_panel::SelectorItemsPanelAppearance;
use gpui_luma::theme::{ControlSize, ThemeMode};

use super::selector_items_panel::selector_items_panel_appearance;
use crate::mode::ShadcnModeTokens;

pub fn selection_panel_appearance(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> SelectionPanelAppearance {
    selection_panel_from_items_panel(selector_items_panel_appearance(mode, theme_mode, size))
}

pub fn selection_panel_from_items_panel(panel: SelectorItemsPanelAppearance) -> SelectionPanelAppearance {
    SelectionPanelAppearance {
        background: panel.background,
        foreground: panel.foreground,
        border: panel.border,
        shadow: panel.shadow,
        radius: panel.radius,
        padding: panel.padding,
        min_width: panel.min_width,
        item_disabled_foreground: panel.item_disabled_foreground,
        item_hover_background: panel.item_hover_background,
        item_hover_foreground: panel.item_hover_foreground,
        item_typography: panel.item_typography,
        item_height: panel.item_height,
        item_padding_x: panel.item_padding_x,
        item_gap: panel.item_gap,
        item_icon_size: panel.item_icon_size,
        item_radius: panel.item_radius,
    }
}
