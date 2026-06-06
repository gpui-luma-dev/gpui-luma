//! Text area — same input tokens as standard text field.

use gpui_luma::controls::textarea::{TextAreaPalette, TextAreaState};
use gpui_luma::controls::textfield::{TextFieldState, TextFieldVariant};

use super::textfield::textfield_palette;
use crate::mode::ShadcnModeTokens;

pub(crate) fn textarea_palette(mode: &ShadcnModeTokens, state: TextAreaState, enabled: bool) -> TextAreaPalette {
    textarea_from_textfield(textfield_palette(mode, TextFieldVariant::Standard, textfield_state_from(state), enabled))
}

fn textfield_state_from(state: TextAreaState) -> TextFieldState {
    TextFieldState {
        hovered: state.hovered,
        focused: state.focused,
        focus_visible: state.focus_visible,
        invalid: state.invalid,
        cursor: state.cursor,
        selection_anchor: state.selection_anchor,
        preferred_column: state.preferred_column,
    }
}

fn textarea_from_textfield(textfield: gpui_luma::controls::textfield::TextFieldPalette) -> TextAreaPalette {
    TextAreaPalette {
        background: textfield.background,
        foreground: textfield.foreground,
        border: textfield.border,
        placeholder: textfield.placeholder,
        selection_background: textfield.selection_background,
        caret: textfield.caret,
        focus_ring: textfield.focus_ring,
        typography: textfield.typography,
        font_family: textfield.font_family.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use gpui_luma::theme::ThemeMode;

    use gpui_luma::controls::textfield::TextFieldState;

    use crate::appearance_context::AppearanceContext;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::controls::textfield::textfield_palette_from_catalog;
    use super::textarea_from_textfield;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn textarea_uses_same_input_tokens_as_textfield() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let ctx = AppearanceContext::new(&mode, gpui_luma::theme::ThemeMode::Light, Default::default());
        let textfield = textfield_palette_from_catalog(
            &ctx,
            gpui_luma::controls::textfield::TextFieldVariant::Standard,
            TextFieldState::default(),
            true,
        )
        .expect("textfield");
        let textarea = textarea_from_textfield(textfield);

        assert_eq!(textarea.background, catalog.color("background").expect("background"));
        assert_eq!(textarea.border, catalog.color("input").expect("input"));
        assert_eq!(textarea.foreground, catalog.color("foreground").expect("foreground"));
    }
}
