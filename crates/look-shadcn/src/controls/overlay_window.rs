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
    let mut look = OverlayWindowLook {
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
    };
    let geometry = tokens.stylesheet().common.overlay_window.resolve_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        gpui_luma::theme::stylesheet::OverlayWindowGeometry {
            padding: look.padding,
            min_width: look.min_width,
            max_width: look.max_width,
            estimated_height: look.estimated_height,
            font_size: look.body.size,
            line_height: look.body.line_height,
        },
    );
    look.padding = geometry.padding.value_px;
    look.min_width = geometry.min_width.value_px;
    look.max_width = geometry.max_width.value_px;
    look.estimated_height = geometry.estimated_height.value_px;
    crate::tables::typography::apply_resolved_geometry_typography(
        &mut look.body,
        &geometry.font_size,
        &geometry.line_height,
    );

    look
}
