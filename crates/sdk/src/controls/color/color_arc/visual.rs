use crate::controls::color::style::active_color_control_theme;
use gpui::Hsla;

#[derive(Clone, Copy)]
pub struct ColorArcVisual {
    pub disabled_overlay: Hsla,
}

pub fn default_color_arc_visual(enabled: bool) -> ColorArcVisual {
    let theme = active_color_control_theme();

    ColorArcVisual { disabled_overlay: theme.background.opacity(if enabled { 0.0 } else { 0.45 }) }
}
