//! Radix context menus use the same panel and item treatments as popup menus.
use std::sync::Arc;
use gpui::FontWeight;
use luma::controls::context_menu::{ContextMenuLook, ContextMenuTemplate, ContextMenuTheme, ThemedContextMenuTemplate};
use luma::theme::{InteractionState, LumaTextStyle, MetricTokens};
use crate::{Look, SemanticRole, Tone};

/// Solid or Soft highlighted items, shared with Radix popup menus.
pub type ContextMenuVariant = crate::PopupMenuVariant;
struct ContextMenuThemeAdapter {
    look: Look,
    variant: ContextMenuVariant,
    tone: Tone,
}
impl ContextMenuTheme for ContextMenuThemeAdapter {
    fn resolve(&self, state: InteractionState) -> ContextMenuLook {
        let metrics = self.look.metrics();
        ContextMenuLook {
            target_background: self.look.resolve_role(SemanticRole::Surface).hsla(),
            target_foreground: self
                .look
                .resolve_role(if state.disabled {
                    SemanticRole::MutedForeground
                } else {
                    SemanticRole::Foreground
                })
                .hsla(),
            target_border: self
                .look
                .resolve_role(if state.focused && !state.disabled {
                    SemanticRole::Focus
                } else {
                    SemanticRole::Border
                })
                .hsla(),
            target_typography: LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::NORMAL },
            target_radius: metrics.radius.md,
            target_padding_x: metrics.control.md.padding_x,
            target_padding_y: metrics.control.md.padding_y,
            target_min_width: 200.0,
            floating_menu: crate::popup_menu::floating_menu_look(&self.look, self.variant, self.tone),
        }
    }
    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }
}
pub fn context_menu_theme(look: &Look, variant: ContextMenuVariant, tone: Tone) -> Arc<dyn ContextMenuTheme> {
    Arc::new(ContextMenuThemeAdapter { look: look.clone(), variant, tone })
}
pub fn context_menu_template(look: &Look, variant: ContextMenuVariant, tone: Tone) -> Arc<dyn ContextMenuTemplate> {
    Arc::new(ThemedContextMenuTemplate::new(context_menu_theme(look, variant, tone)))
}
#[cfg(test)]
mod tests {
    use super::*;
    use luma::theme::ThemeMode;
    #[test]
    fn menu_matches_popup_styling_in_both_modes_and_tones() {
        let look = Look::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for variant in [ContextMenuVariant::Solid, ContextMenuVariant::Soft] {
                for tone in [Tone::Accent, Tone::Gray] {
                    let actual =
                        context_menu_theme(&look, variant, tone).resolve(InteractionState::default()).floating_menu;
                    let expected = crate::popup_menu::floating_menu_look(&look, variant, tone);
                    assert_eq!(actual.background, expected.background);
                    assert_eq!(actual.border, expected.border);
                    assert_eq!(actual.item_hover_background, expected.item_hover_background);
                    assert_eq!(actual.item_hover_foreground, expected.item_hover_foreground);
                }
            }
        }
    }
    #[test]
    fn focus_and_disabled_target_follow_radix_roles() {
        let look = Look::built_in();
        let theme = context_menu_theme(&look, ContextMenuVariant::Solid, Tone::Accent);
        let focused = theme.resolve(InteractionState { focused: true, ..Default::default() });
        assert_eq!(focused.target_border, look.resolve_role(SemanticRole::Focus).hsla());
        let disabled = theme.resolve(InteractionState { focused: true, disabled: true, ..Default::default() });
        assert_eq!(disabled.target_border, look.resolve_role(SemanticRole::Border).hsla());
        assert_eq!(disabled.target_foreground, look.resolve_role(SemanticRole::MutedForeground).hsla());
    }
}
