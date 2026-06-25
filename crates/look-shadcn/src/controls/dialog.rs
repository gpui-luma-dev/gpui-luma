use gpui::{Hsla, black};
use gpui_luma::controls::dialog::{DialogLook, DialogMode};
use gpui_luma::theme::ControlSize;

use crate::look::ShadcnLook;
use crate::tokens::{ShadcnFont, ShadcnRadius, ShadcnShadow, ShadcnTextRole, ShadcnTextSize, ShadcnToken};

pub fn dialog_look(theme: &ShadcnLook, size: ControlSize, mode: DialogMode) -> DialogLook {
    let tokens = theme.mode_tokens();
    let metrics = &tokens.metrics;
    let typography = &tokens.typography;
    let chrome = theme.chrome();
    let background = theme.token_color("popover").unwrap_or_else(|_| theme.color(ShadcnToken::Card));
    let foreground =
        theme.token_color("popover-foreground").unwrap_or_else(|_| theme.color(ShadcnToken::CardForeground));
    let border = theme.token_color("border").unwrap_or(chrome.border);
    let muted = theme.token_color("muted-foreground").unwrap_or(chrome.muted_text);

    DialogLook {
        background,
        border,
        title_color: foreground,
        description_color: muted,
        body_color: foreground,
        backdrop_background: if mode == DialogMode::Modal {
            Hsla { a: 0.42, ..black() }
        } else {
            Hsla { a: 0.0, ..black() }
        },
        shadow: theme.shadow(ShadcnShadow::Lg),
        radius: theme.radius(ShadcnRadius::Lg),
        padding: metrics.padding_x(size),
        section_gap: metrics.gap(size),
        header_gap: (metrics.gap(size) * 0.5).max(6.0),
        footer_gap: metrics.gap(size) * 0.75,
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
        title: match size {
            ControlSize::Sm => typography.text.label,
            ControlSize::Md => theme.typography_role(ShadcnTextRole::H4),
            ControlSize::Lg => theme.typography_scale(ShadcnTextSize::Xl),
        },
        description: typography.text.caption,
        body: typography.text.body,
        font_family: theme.font(ShadcnFont::Sans),
    }
}
