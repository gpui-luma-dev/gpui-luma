//! Radix popup-menu theme adapter (Copy / Actions menus).

use std::sync::Arc;

use gpui::{BoxShadow, FontWeight, black, point, px};
use luma::controls::floating_menu::FloatingMenuLook;
use luma::controls::popup_menu::{
    PopupMenuPalette, PopupMenuTheme, PopupMenuTriggerMetrics, PopupMenuTriggerStyle, compose_popup_menu_look,
};
use luma::theme::{InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale};

use crate::look::RadixLook;
use crate::semantic::SemanticRole;

struct RadixPopupMenuTheme {
    look: RadixLook,
}

impl PopupMenuTheme for RadixPopupMenuTheme {
    fn resolve(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
    ) -> PopupMenuPalette {
        let _ = metrics;
        let layer = state.layer();
        let (mut background, mut foreground, mut border) = match trigger_style {
            PopupMenuTriggerStyle::Primary => (
                self.look.resolve_role(SemanticRole::Primary).hsla(),
                self.look.resolve_role(SemanticRole::PrimaryForeground).hsla(),
                Some(self.look.resolve_role(SemanticRole::Primary).hsla()),
            ),
            PopupMenuTriggerStyle::Secondary => (
                self.look.resolve_role(SemanticRole::Soft).hsla(),
                self.look.resolve_role(SemanticRole::SoftForeground).hsla(),
                Some(self.look.resolve_role(SemanticRole::Soft).hsla()),
            ),
            PopupMenuTriggerStyle::Outline => (
                self.look.resolve_role(SemanticRole::Background).hsla(),
                self.look.resolve_role(SemanticRole::Foreground).hsla(),
                Some(self.look.resolve_role(SemanticRole::Border).hsla()),
            ),
            PopupMenuTriggerStyle::Ghost => {
                (gpui::hsla(0.0, 0.0, 0.0, 0.0), self.look.resolve_role(SemanticRole::Foreground).hsla(), None)
            }
        };

        match layer {
            InteractionLayer::Disabled => {
                background = self.look.resolve_role(SemanticRole::Surface).hsla();
                foreground = self.look.resolve_role(SemanticRole::MutedForeground).hsla();
                border = Some(self.look.resolve_role(SemanticRole::Border).hsla());
            }
            InteractionLayer::Hovered | InteractionLayer::Pressed => {
                if matches!(trigger_style, PopupMenuTriggerStyle::Ghost | PopupMenuTriggerStyle::Outline) {
                    background = self.look.resolve_role(SemanticRole::Soft).hsla();
                } else if matches!(trigger_style, PopupMenuTriggerStyle::Primary) {
                    background = self.look.resolve_step(crate::scale::ScaleFamily::Color, 10).hsla();
                }
            }
            InteractionLayer::Default => {}
        }

        if state.focused && !state.disabled {
            border = Some(self.look.resolve_role(SemanticRole::Focus).hsla());
        }

        PopupMenuPalette {
            trigger_background: background,
            trigger_foreground: foreground,
            trigger_border: border,
            trigger_shadow: None,
            trigger_typography: LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::MEDIUM },
            floating_menu: floating_menu_look(&self.look),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn resolve_look(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
        scale_factor: f32,
        _cx: &mut gpui::App,
    ) -> luma::controls::popup_menu::PopupMenuLook {
        let scale = StandardBoxScale::compute(metrics.size, &self.metrics(), scale_factor);
        let mut look = compose_popup_menu_look(&self.resolve(trigger_style, metrics, state), &scale);
        if let Some(radius) = metrics.trigger_radius_override {
            look.trigger_radius = radius;
        }
        look
    }
}

fn floating_menu_look(look: &RadixLook) -> FloatingMenuLook {
    let metrics = look.metrics();
    FloatingMenuLook {
        background: look.resolve_role(SemanticRole::Surface).hsla(),
        foreground: look.resolve_role(SemanticRole::Foreground).hsla(),
        border: look.resolve_role(SemanticRole::Border).hsla(),
        shadow: vec![BoxShadow {
            offset: point(px(0.0), px(10.0)),
            blur_radius: px(28.0),
            spread_radius: px(-12.0),
            color: gpui::Hsla { a: 0.28, ..black() },
            inset: false,
        }],
        radius: metrics.radius.lg,
        padding: metrics.spacing.s2,
        min_width: 180.0,
        item_disabled_foreground: look.resolve_role(SemanticRole::MutedForeground).hsla(),
        item_hover_background: look.resolve_role(SemanticRole::Soft).hsla(),
        item_hover_foreground: look.resolve_role(SemanticRole::SoftForeground).hsla(),
        item_typography: LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::NORMAL },
        item_height: metrics.control.md.height * 0.9,
        item_padding_x: metrics.control.md.padding_x * 0.75,
        item_gap: metrics.spacing.s2,
        item_icon_size: metrics.control.md.icon_size,
        item_radius: metrics.radius.sm,
        disabled_opacity: 0.56,
        submenu_offset_x: metrics.spacing.s1,
    }
}

pub fn popup_menu_theme(look: Arc<RadixLook>) -> Arc<dyn PopupMenuTheme> {
    Arc::new(RadixPopupMenuTheme { look: look.as_ref().clone() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_trigger_uses_accent() {
        let look = Arc::new(RadixLook::built_in());
        let theme = popup_menu_theme(look.clone());
        let palette = theme.resolve(
            PopupMenuTriggerStyle::Primary,
            PopupMenuTriggerMetrics::default(),
            InteractionState::default(),
        );
        assert_eq!(palette.trigger_background.l, look.resolve_role(SemanticRole::Primary).hsla().l);
    }
}
