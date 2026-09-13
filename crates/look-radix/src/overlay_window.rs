//! Radix overlay-window theme for swatch info panels.

use std::sync::Arc;

use gpui::{BoxShadow, FontWeight, SharedString, black, point, px};
use luma::controls::overlay_window::{OverlayWindowLook, OverlayWindowMode, OverlayWindowTheme};
use luma::theme::{ControlSize, LumaTextStyle};

use crate::look::Look;
use crate::semantic::SemanticRole;

struct OverlayWindowThemeAdapter {
    look: Look,
}

impl OverlayWindowTheme for OverlayWindowThemeAdapter {
    fn resolve(&self, size: ControlSize, mode: OverlayWindowMode) -> OverlayWindowLook {
        let metrics = self.look.metrics();
        OverlayWindowLook {
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
            min_width: match size {
                ControlSize::Sm => 280.0,
                ControlSize::Md => 360.0,
                ControlSize::Lg => 440.0,
            },
            max_width: match size {
                ControlSize::Sm => 360.0,
                ControlSize::Md => 440.0,
                ControlSize::Lg => 520.0,
            },
            estimated_height: match size {
                ControlSize::Sm => 180.0,
                ControlSize::Md => 220.0,
                ControlSize::Lg => 260.0,
            },
            body: LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::NORMAL },
            font_family: SharedString::from("System UI"),
        }
    }
}

pub fn overlay_window_theme(look: &Look) -> Arc<dyn OverlayWindowTheme> {
    Arc::new(OverlayWindowThemeAdapter { look: look.clone() })
}
