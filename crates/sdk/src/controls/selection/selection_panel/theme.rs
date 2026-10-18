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

pub fn default_selection_panel_look(tokens: &ThemeTokens, size: ControlSize) -> SelectionPanelLook {
    let menu = crate::controls::floating_menu::default_floating_menu_look(tokens, size);
    let mut look = SelectionPanelLook {
        background: menu.background,
        foreground: menu.foreground,
        border: menu.border,
        shadow: menu.shadow,
        radius: menu.radius,
        padding: menu.padding,
        min_width: menu.min_width,
        item_disabled_foreground: menu.item_disabled_foreground,
        item_hover_background: menu.item_hover_background,
        item_hover_foreground: menu.item_hover_foreground,
        item_typography: tokens.typography.text.label,
        item_height: menu.item_height,
        item_padding_x: menu.item_padding_x,
        item_gap: menu.item_gap,
        item_icon_size: menu.item_icon_size,
        item_radius: menu.item_radius,
    };
    crate::theme::stylesheet::apply_common_geometry!(tokens.stylesheet.selection_panel, size.stylesheet_key(), SelectionPanelGeometry, look,
        [padding => padding, min_width => min_width, item_height => item_height,
         item_padding_x => item_padding_x, item_gap => item_gap, item_icon_size => item_icon_size,
         font_size => item_typography.size, line_height => item_typography.line_height]);
    look
}
