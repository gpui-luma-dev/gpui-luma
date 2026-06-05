use gpui::{InteractiveElement, StatefulInteractiveElement, Styled, px};

use crate::context::with_active_look;
use crate::tokens::{ShadcnFont, ShadcnRadius, ShadcnStyle, ShadcnTextSize, ShadcnToken};

fn resolve_style_color(style: ShadcnStyle) -> Option<gpui::Hsla> {
    with_active_look(|look| {
        look.map(|look| {
            let mut color = look.color(style.token);
            if let Some(alpha) = style.opacity {
                color.a = alpha;
            }
            color
        })
    })
}

/// Styling extensions that resolve shadcn tokens from the active look context.
pub trait ShadcnElementExt: Styled + Sized {
    fn bg_cn(self, style: impl Into<ShadcnStyle>) -> Self {
        let style = style.into();
        if let Some(color) = resolve_style_color(style) {
            self.bg(color)
        } else {
            self
        }
    }

    fn text_cn(self, style: impl Into<ShadcnStyle>) -> Self {
        let style = style.into();
        if let Some(color) = resolve_style_color(style) {
            self.text_color(color)
        } else {
            self
        }
    }

    fn border_cn(self, style: impl Into<ShadcnStyle>) -> Self {
        let style = style.into();
        if let Some(color) = resolve_style_color(style) {
            self.border_color(color)
        } else {
            self
        }
    }

    fn hover_bg_cn(self, style: impl Into<ShadcnStyle>) -> Self
    where
        Self: InteractiveElement,
    {
        let style = style.into();
        self.hover(move |s| {
            if let Some(color) = resolve_style_color(style) {
                s.bg(color)
            } else {
                s
            }
        })
    }

    fn hover_text_cn(self, style: impl Into<ShadcnStyle>) -> Self
    where
        Self: InteractiveElement,
    {
        let style = style.into();
        self.hover(move |s| {
            if let Some(color) = resolve_style_color(style) {
                s.text_color(color)
            } else {
                s
            }
        })
    }

    fn active_bg_cn(self, style: impl Into<ShadcnStyle>) -> Self
    where
        Self: StatefulInteractiveElement,
    {
        let style = style.into();
        self.active(move |s| {
            if let Some(color) = resolve_style_color(style) {
                s.bg(color)
            } else {
                s
            }
        })
    }

    fn focus_border_cn(self, style: impl Into<ShadcnStyle>) -> Self
    where
        Self: InteractiveElement,
    {
        let style = style.into();
        self.focus(move |s| {
            if let Some(color) = resolve_style_color(style) {
                s.border_color(color)
            } else {
                s
            }
        })
    }

    /// Draws focus ring using active ring color (matches ring-ring focus styles).
    fn focus_ring_cn(self) -> Self
    where
        Self: InteractiveElement,
    {
        self.focus(|s| {
            if let Some(color) = resolve_style_color(ShadcnToken::Ring.into()) {
                s.border_color(color)
            } else {
                s
            }
        })
    }

    fn rounded_cn(self, role: ShadcnRadius) -> Self {
        with_active_look(|look| {
            if let Some(look) = look {
                self.rounded(px(look.radius(role)))
            } else {
                self
            }
        })
    }

    fn font_cn(self, role: ShadcnFont) -> Self {
        with_active_look(|look| {
            if let Some(look) = look {
                self.font_family(look.font(role))
            } else {
                self
            }
        })
    }

    fn text_size_cn(self, size: ShadcnTextSize) -> Self {
        self.text_size(px(size.px()))
    }

    fn gap_cn(self, step: f32) -> Self {
        with_active_look(|look| {
            if let Some(look) = look {
                let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
                self.gap(px(step * spacing))
            } else {
                self
            }
        })
    }

    fn p_cn(self, step: f32) -> Self {
        with_active_look(|look| {
            if let Some(look) = look {
                let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
                self.p(px(step * spacing))
            } else {
                self
            }
        })
    }

    fn px_cn(self, step: f32) -> Self {
        with_active_look(|look| {
            if let Some(look) = look {
                let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
                self.px(px(step * spacing))
            } else {
                self
            }
        })
    }

    fn py_cn(self, step: f32) -> Self {
        with_active_look(|look| {
            if let Some(look) = look {
                let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
                self.py(px(step * spacing))
            } else {
                self
            }
        })
    }
}

impl<S: Styled> ShadcnElementExt for S {}
