use gpui_luma::controls::pager::{PagerLook, PagerStyle};
use gpui_luma::theme::ControlSize;

use crate::look::ShadcnLook;

pub fn pager_look(theme: &ShadcnLook, enabled: bool, style: PagerStyle) -> PagerLook {
    let tokens = theme.mode_tokens();
    let palette = &tokens.palette;
    let metrics = &tokens.metrics;
    let typography = theme.typography_scale(crate::ShadcnTextSize::Xs);
    let compact = matches!(style, PagerStyle::Minimal);

    PagerLook {
        panel_background: if enabled {
            palette.panel_background
        } else {
            palette.disabled_background
        },
        border: palette.border_default,
        body_text: if enabled {
            palette.body_text
        } else {
            palette.disabled_foreground
        },
        muted_text: if enabled {
            palette.app_muted_foreground
        } else {
            palette.disabled_foreground
        },
        selected_background: palette.selected_background,
        selected_foreground: palette.selected_foreground,
        hover_background: palette.accent_background,
        shadow: vec![],
        typography,
        button_size: if compact {
            28.0
        } else {
            metrics.control_height(ControlSize::Sm)
        },
        button_min_width: if compact { 28.0 } else { 32.0 },
        control_height: metrics.control_height(ControlSize::Sm),
        radius: metrics.radius(ControlSize::Sm),
        padding_x: 8.0,
        padding_y: 8.0,
        gap: 4.0,
        group_gap: if compact { 8.0 } else { 16.0 },
        disabled_opacity: 0.56,
    }
}
