//! Text area — surface/soft variants share the text field token resolver.

use gpui_luma::controls::textarea::{TextAreaPalette, TextAreaState};
use gpui_luma::controls::textfield::TextFieldState;
use gpui_luma::theme::ThemeMode;

use super::textfield::{ShadcnTextFieldStyle, textfield_palette};
use crate::mode::ShadcnModeTokens;

pub fn textarea_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextAreaState,
    enabled: bool,
) -> TextAreaPalette {
    textarea_from_textfield(textfield_palette(mode, theme_mode, style, textfield_state_from(state), enabled))
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
        selection_foreground: textfield.selection_foreground,
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

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::controls::textfield::{ShadcnTextFieldStyle, textfield_palette};
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
            ("input".into(), "oklch(0.7200 0.0120 205.0000)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn textarea_uses_same_surface_tokens_as_textfield() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let textfield =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Surface, TextFieldState::default(), true);
        let textarea = textarea_from_textfield(textfield);

        assert_eq!(textarea.background, gpui::hsla(0.0, 0.0, 0.0, 0.0));
        assert_eq!(textarea.border, catalog.color("border").expect("border"));
        assert_eq!(textarea.foreground, catalog.color("foreground").expect("foreground"));
        assert_eq!(textarea.selection_foreground, catalog.color("primary-foreground").expect("primary-foreground"));
    }

    #[test]
    fn soft_textarea_uses_muted_fill_and_no_border() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let textfield =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Soft, TextFieldState::default(), true);
        let textarea = textarea_from_textfield(textfield);

        assert_eq!(textarea.background, catalog.color("muted").expect("muted"));
        assert_eq!(textarea.border, gpui::hsla(0.0, 0.0, 0.0, 0.0));
    }
}
