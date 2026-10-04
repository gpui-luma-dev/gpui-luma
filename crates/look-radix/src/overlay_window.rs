//! Radix overlay-window theme for swatch info panels.

use std::sync::Arc;

use gpui::{BoxShadow, FontWeight, SharedString, black, point, px};
use gpui_luma::controls::overlay_window::{OverlayWindowLook, OverlayWindowMode, OverlayWindowTheme};
use gpui_luma::theme::{ControlSize, LumaTextStyle};

use crate::look::Look;
use crate::semantic::SemanticRole;

struct OverlayWindowThemeAdapter {
    look: Look,
}

impl OverlayWindowTheme for OverlayWindowThemeAdapter {
    fn resolve(&self, size: ControlSize, mode: OverlayWindowMode) -> OverlayWindowLook {
        let metrics = self.look.metrics();
        let fallback = gpui_luma::controls::overlay_window::default_overlay_window_theme().resolve(size, mode);
        let mut resolved = OverlayWindowLook {
            background: self.look.resolve_role(SemanticRole::Surface).hsla(),
            border: self.look.resolve_role(SemanticRole::Border).hsla(),
            foreground: self.look.resolve_role(SemanticRole::Foreground).hsla(),
            backdrop_background: if mode == OverlayWindowMode::Modal {
                gpui::Hsla { a: 0.42, ..black() }
            } else {
                gpui::Hsla { a: 0.0, ..black() }
            },
            shadow: vec![BoxShadow {
                offset: point(px(0.0), px(18.0)),
                blur_radius: px(38.0),
                spread_radius: px(-16.0),
                color: gpui::Hsla { a: 0.22, ..black() },
                inset: false,
            }],
            radius: metrics.radius.xl,
            padding: metrics.padding_x(size),
            min_width: fallback.min_width,
            max_width: fallback.max_width,
            estimated_height: fallback.estimated_height,
            body: LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::NORMAL },
            font_family: SharedString::from("System UI"),
        };
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        let geometry = self.look.common_stylesheet().overlay_window.resolve_geometry(
            key,
            gpui_luma::theme::stylesheet::OverlayWindowGeometry {
                padding: resolved.padding,
                min_width: resolved.min_width,
                max_width: resolved.max_width,
                estimated_height: resolved.estimated_height,
                font_size: resolved.body.size,
                line_height: resolved.body.line_height,
            },
        );
        resolved.padding = geometry.padding.value_px;
        resolved.min_width = geometry.min_width.value_px;
        resolved.max_width = geometry.max_width.value_px;
        resolved.estimated_height = geometry.estimated_height.value_px;
        resolved.body.size = geometry.font_size.value_px;
        resolved.body.line_height = geometry.line_height.value_px;
        resolved
    }
}

pub fn overlay_window_theme(look: &Look) -> Arc<dyn OverlayWindowTheme> {
    Arc::new(OverlayWindowThemeAdapter { look: look.clone() })
}
