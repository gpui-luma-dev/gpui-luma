use crate::controls::color::style::active_color_control_theme;
use gpui::Hsla;

#[derive(Clone, Copy)]
pub struct ColorRingVisual {
    pub border: Hsla,
    pub disabled_overlay: Hsla,
    pub center_hole: Hsla,
}

pub fn default_color_ring_visual(enabled: bool) -> ColorRingVisual {
    let theme = active_color_control_theme();

    ColorRingVisual {
        border: theme.border,
        disabled_overlay: theme.background.opacity(if enabled { 0.0 } else { 0.45 }),
        center_hole: theme.background,
    }
}
