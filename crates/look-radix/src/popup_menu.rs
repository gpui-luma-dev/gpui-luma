//! Radix popup-menu theme adapter (Copy / Actions menus).

use std::sync::Arc;

use gpui::{BoxShadow, FontWeight, black, point, px};
use luma::controls::floating_menu::FloatingMenuLook;
use luma::controls::popup_menu::{
    PopupMenuPalette, PopupMenuTemplate, PopupMenuTheme, PopupMenuTriggerMetrics, PopupMenuTriggerStyle,
    ThemedPopupMenuTemplate, compose_popup_menu_look,
};
use luma::theme::{InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale};

use crate::look::Look;
use crate::semantic::SemanticRole;
use crate::tone::Tone;

/// Radix Themes menu content variants, as carried by the SDK popup menu control.
///
/// The variant only governs the highlighted item: the panel itself is identical.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopupMenuVariant {
    /// Highlighted item takes the solid accent (`accent-9` on `accent-contrast`).
    #[default]
    Solid,
    /// Highlighted item takes a soft accent tint (`accent-4`), text unchanged.
    Soft,
}

struct PopupMenuThemeAdapter {
    look: Look,
    variant: PopupMenuVariant,
    tone: Tone,
}

impl PopupMenuTheme for PopupMenuThemeAdapter {
    fn resolve(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
    ) -> PopupMenuPalette {
        let _ = metrics;
        let layer = state.layer();
        let tone = self.tone;
        let step = |step| tone.step(&self.look, step);
        let (mut background, mut foreground, mut border) = match trigger_style {
            PopupMenuTriggerStyle::Primary => (step(9), tone.contrast(&self.look), Some(step(9))),
            PopupMenuTriggerStyle::Secondary => (step(3), step(11), Some(step(3))),
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
                    background = step(3);
                } else if matches!(trigger_style, PopupMenuTriggerStyle::Primary) {
                    background = step(10);
                } else if matches!(trigger_style, PopupMenuTriggerStyle::Secondary) {
                    background = step(4);
                }
            }
            InteractionLayer::Default => {}
        }

        if state.focused && !state.disabled {
            border = Some(step(8));
        }

        PopupMenuPalette {
            trigger_background: background,
            trigger_foreground: foreground,
            trigger_border: border,
            trigger_shadow: None,
            trigger_typography: LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::MEDIUM },
            floating_menu: floating_menu_look(&self.look, self.variant, self.tone),
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

fn floating_menu_look(look: &Look, variant: PopupMenuVariant, tone: Tone) -> FloatingMenuLook {
    let metrics = look.metrics();
    let (item_hover_background, item_hover_foreground) = match variant {
        PopupMenuVariant::Solid => (tone.step(look, 9), tone.contrast(look)),
        PopupMenuVariant::Soft => (tone.step(look, 4), look.resolve_role(SemanticRole::Foreground).hsla()),
    };

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
        item_hover_background,
        item_hover_foreground,
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

pub fn popup_menu_theme(look: &Look, variant: PopupMenuVariant, tone: Tone) -> Arc<dyn PopupMenuTheme> {
    Arc::new(PopupMenuThemeAdapter { look: look.clone(), variant, tone })
}

pub fn popup_menu_template(look: &Look, variant: PopupMenuVariant, tone: Tone) -> Arc<dyn PopupMenuTemplate> {
    Arc::new(ThemedPopupMenuTemplate::new(popup_menu_theme(look, variant, tone)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trigger_palette(look: &Look, tone: Tone) -> PopupMenuPalette {
        popup_menu_theme(look, PopupMenuVariant::default(), tone).resolve(
            PopupMenuTriggerStyle::Primary,
            PopupMenuTriggerMetrics::default(),
            InteractionState::default(),
        )
    }

    #[test]
    fn primary_trigger_uses_accent() {
        let look = Look::built_in();
        let palette = trigger_palette(&look, Tone::Accent);

        assert_eq!(palette.trigger_background.l, look.resolve_role(SemanticRole::Primary).hsla().l);
    }

    #[test]
    fn gray_tone_locks_the_trigger_to_the_neutral_scale() {
        let look = Look::built_in();
        let gray = trigger_palette(&look, Tone::Gray);

        assert_eq!(gray.trigger_background, Tone::Gray.step(&look, 9));
        assert_ne!(gray.trigger_background, trigger_palette(&look, Tone::Accent).trigger_background);
    }

    #[test]
    fn solid_variant_highlights_items_with_the_solid_accent() {
        let look = Look::built_in();
        let menu = floating_menu_look(&look, PopupMenuVariant::Solid, Tone::Accent);

        assert_eq!(menu.item_hover_background, look.resolve_role(SemanticRole::Primary).hsla());
        assert_eq!(menu.item_hover_foreground, look.resolve_role(SemanticRole::PrimaryForeground).hsla());
    }

    #[test]
    fn soft_variant_tints_items_and_keeps_the_panel_text() {
        let look = Look::built_in();
        let solid = floating_menu_look(&look, PopupMenuVariant::Solid, Tone::Accent);
        let soft = floating_menu_look(&look, PopupMenuVariant::Soft, Tone::Accent);

        assert_eq!(soft.item_hover_foreground, look.resolve_role(SemanticRole::Foreground).hsla());
        assert!(soft.item_hover_background.l > solid.item_hover_background.l);
        assert_eq!(soft.background, solid.background);
    }

    #[test]
    fn gray_tone_keeps_the_panel_and_only_swaps_the_highlight() {
        let look = Look::built_in();
        let accent = floating_menu_look(&look, PopupMenuVariant::Solid, Tone::Accent);
        let gray = floating_menu_look(&look, PopupMenuVariant::Solid, Tone::Gray);

        assert_eq!(gray.item_hover_background, Tone::Gray.step(&look, 9));
        assert_ne!(gray.item_hover_background, accent.item_hover_background);
        assert_eq!(gray.background, accent.background);
    }
}
