//! Radix text-field theme adapter for the SDK [`TextFieldTheme`] contract.

use std::sync::Arc;

use gpui::{FontWeight, SharedString};
use luma::controls::textfield::{
    TextFieldLook, TextFieldPalette, TextFieldState, TextFieldTheme, TextFieldVariant, compose_textfield_look,
};
use luma::theme::{ControlSize, LumaTextStyle, MetricTokens, StandardBoxScale};

use crate::look::RadixLook;
use crate::semantic::SemanticRole;

struct RadixTextFieldTheme {
    look: RadixLook,
}

impl TextFieldTheme for RadixTextFieldTheme {
    fn resolve(&self, _variant: TextFieldVariant, state: TextFieldState, enabled: bool) -> TextFieldPalette {
        resolve_textfield_palette(&self.look, state, enabled)
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn resolve_look(
        &self,
        variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> TextFieldLook {
        let mut palette = self.resolve(variant, state, enabled);
        palette.typography = typography_for_size(size);
        compose_textfield_look(&palette, scale, self.metrics().border_width.default)
    }
}

pub fn textfield_theme(look: Arc<RadixLook>) -> Arc<dyn TextFieldTheme> {
    Arc::new(RadixTextFieldTheme { look: look.as_ref().clone() })
}

fn resolve_textfield_palette(look: &RadixLook, state: TextFieldState, enabled: bool) -> TextFieldPalette {
    let background = look.resolve_role(SemanticRole::Background).hsla();
    let foreground = look.resolve_role(SemanticRole::Foreground).hsla();
    let placeholder = look.resolve_role(SemanticRole::MutedForeground).hsla();
    let selection_background = look.resolve_role(SemanticRole::Soft).hsla();
    let selection_foreground = look.resolve_role(SemanticRole::SoftForeground).hsla();

    let (background, foreground, border, placeholder, icon, selection_background, selection_foreground, caret) =
        if !enabled {
            (
                look.resolve_role(SemanticRole::Surface).hsla(),
                look.resolve_role(SemanticRole::MutedForeground).hsla(),
                look.resolve_role(SemanticRole::Border).hsla(),
                look.resolve_role(SemanticRole::MutedForeground).hsla(),
                look.resolve_role(SemanticRole::MutedForeground).hsla(),
                selection_background,
                selection_foreground,
                look.resolve_role(SemanticRole::MutedForeground).hsla(),
            )
        } else {
            let border = if state.invalid {
                look.resolve_role(SemanticRole::Destructive).hsla()
            } else if state.focused {
                look.resolve_role(SemanticRole::Focus).hsla()
            } else if state.hovered {
                look.resolve_step(crate::scale::ScaleFamily::Gray, 7).hsla()
            } else {
                look.resolve_role(SemanticRole::Border).hsla()
            };
            (
                background,
                foreground,
                border,
                placeholder,
                placeholder,
                selection_background,
                selection_foreground,
                foreground,
            )
        };

    TextFieldPalette {
        background,
        foreground,
        border,
        placeholder,
        icon,
        selection_background,
        selection_foreground,
        caret,
        shadow: None,
        typography: typography_for_size(ControlSize::Md),
        font_family: SharedString::from("System UI"),
    }
}

fn typography_for_size(size: ControlSize) -> LumaTextStyle {
    match size {
        ControlSize::Sm => LumaTextStyle { size: 12.5, line_height: 18.0, weight: FontWeight::NORMAL },
        ControlSize::Md => LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::NORMAL },
        ControlSize::Lg => LumaTextStyle { size: 16.0, line_height: 22.0, weight: FontWeight::NORMAL },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focused_border_uses_focus_role() {
        let look = Arc::new(RadixLook::built_in());
        let theme = textfield_theme(look.clone());
        let palette = theme.resolve(
            TextFieldVariant::Standard,
            TextFieldState { focused: true, focus_visible: true, ..Default::default() },
            true,
        );
        let expected = look.resolve_role(SemanticRole::Focus).hsla();
        assert_eq!(palette.border.l, expected.l);
    }
}
