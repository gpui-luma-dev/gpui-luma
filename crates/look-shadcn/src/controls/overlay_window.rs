use gpui::{Hsla, black};
use gpui_luma::controls::overlay_window::{OverlayWindowLook, OverlayWindowMode};
use gpui_luma::theme::ControlSize;

use crate::look::ShadcnLook;
use crate::tokens::{ShadcnFont, ShadcnRadius, ShadcnShadow, ShadcnToken};

pub fn overlay_window_look(theme: &ShadcnLook, size: ControlSize, mode: OverlayWindowMode) -> OverlayWindowLook {
    let tokens = theme.mode_tokens();
    let metrics = &tokens.metrics;
    let typography = &tokens.typography;
    let chrome = theme.chrome();
    let background = theme.token_color("popover").unwrap_or_else(|_| theme.color(ShadcnToken::Card));
    let foreground =
        theme.token_color("popover-foreground").unwrap_or_else(|_| theme.color(ShadcnToken::CardForeground));
    let border = theme.token_color("border").unwrap_or(chrome.border);
    OverlayWindowLook {
        background,
        border,
        foreground,
        backdrop_background: if mode == OverlayWindowMode::Modal {
            Hsla { a: 0.42, ..black() }
        } else {
            Hsla { a: 0.0, ..black() }
        },
        shadow: theme.shadow(ShadcnShadow::Lg),
        radius: theme.radius(ShadcnRadius::Lg),
        padding: metrics.padding_x(size),
        min_width: match size {
            ControlSize::Sm => 320.0,
            ControlSize::Md => 400.0,
            ControlSize::Lg => 480.0,
        },
        max_width: match size {
            ControlSize::Sm => 420.0,
            ControlSize::Md => 520.0,
            ControlSize::Lg => 620.0,
        },
        estimated_height: match size {
            ControlSize::Sm => 196.0,
            ControlSize::Md => 228.0,
            ControlSize::Lg => 264.0,
        },
        body: typography.text.body,
        font_family: theme.font(ShadcnFont::Sans),
    }
}
