use crate::controls::color::style::active_color_control_theme;
use gpui::{hsla, Hsla};

#[derive(Clone, Copy)]
pub struct ColorSliderVisual {
    pub border: Hsla,
    pub disabled_overlay: Hsla,
    pub blocked_overlay: Hsla,
}

pub fn default_color_slider_visual(enabled: bool) -> ColorSliderVisual {
    let theme = active_color_control_theme();

    ColorSliderVisual {
        border: theme.border,
        disabled_overlay: theme.background.opacity(if enabled { 0.0 } else { 0.45 }),
        blocked_overlay: if theme.is_dark() {
            hsla(0., 0., 0.18, 1.)
        } else {
            hsla(0., 0., 0.72, 1.)
        },
    }
}
