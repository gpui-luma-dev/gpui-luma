//! Text area — same input tokens as standard text field.

use crate::controls::textarea::{TextAreaAppearance, TextAreaState};
use crate::controls::textfield::{TextFieldState, TextFieldVariant};

use super::textfield::textfield_appearance;
use super::super::mode::RadixModeTokens;

pub(crate) fn textarea_appearance(mode: &RadixModeTokens, state: TextAreaState, enabled: bool) -> TextAreaAppearance {
    textarea_from_textfield(textfield_appearance(
        mode,
        TextFieldVariant::Standard,
        textfield_state_from(state),
        enabled,
    ))
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

fn textarea_from_textfield(textfield: crate::controls::textfield::TextFieldAppearance) -> TextAreaAppearance {
    TextAreaAppearance {
        background: textfield.background,
        foreground: textfield.foreground,
        border: textfield.border,
        placeholder: textfield.placeholder,
        selection_background: textfield.selection_background,
        caret: textfield.caret,
        focus_ring: textfield.focus_ring,
        typography: textfield.typography,
        font_family: textfield.font_family.to_string(),
        min_height: textfield.min_height,
        padding_x: textfield.padding_x,
        padding_y: textfield.padding_y,
        radius: textfield.radius,
        border_width: textfield.border_width,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::controls::textfield::TextFieldState;

    use super::super::super::catalog::CssTokenMap;
    use super::super::super::mode::RadixModeTokens;
    use super::super::textfield::textfield_appearance_from_catalog;
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
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let textfield = textfield_appearance_from_catalog(
            &catalog,
            &mode,
            crate::controls::textfield::TextFieldVariant::Standard,
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
