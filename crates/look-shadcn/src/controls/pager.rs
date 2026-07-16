use gpui::FontWeight;
use gpui_luma::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole};
use gpui_luma::controls::pager::{PagerLook, PagerStyle};
use gpui_luma::theme::{ControlSize, InteractionState};

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

pub fn pager_button_look(
    theme: &ShadcnLook,
    pager_look: &PagerLook,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> ButtonFamilyLook {
    let mut look = theme.resolve_outline_button(role, ControlSize::Sm, state);
    look.height = pager_look.button_size;
    look.radius = pager_look.radius;
    look.gap = pager_look.gap;
    look.typography.size = pager_look.typography.size;
    look.typography.line_height = pager_look.typography.line_height;
    look.typography.weight = if matches!(role, ButtonFamilyRole::Toggle { selected: true }) {
        FontWeight::SEMIBOLD
    } else {
        pager_look.typography.weight
    };

    if matches!(role, ButtonFamilyRole::Icon) {
        look.padding_x = 0.0;
        look.padding_y = 0.0;
        look.icon_size = pager_look.button_size * 0.44;
    } else {
        look.padding_x = pager_look.padding_x;
        look.padding_y = 0.0;
    }

    look
}
