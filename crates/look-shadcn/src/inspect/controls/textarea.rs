//! Inspect metadata for `textarea` (delegates to textfield).

use gpui_luma::controls::textarea::TextAreaState;
use gpui_luma::theme::{ControlSize, ThemeMode};
use crate::{ShadcnModeTokens, ShadcnTextFieldStyle};

use super::textfield::{
    TextFieldInspectMetrics, TextFieldInspectPalette, inspect_textfield_color_palette, inspect_textfield_metrics,
};

fn textfield_state_from(state: TextAreaState) -> gpui_luma::controls::textfield::TextFieldState {
    gpui_luma::controls::textfield::TextFieldState {
        hovered: state.hovered,
        focused: state.focused,
        focus_visible: state.focus_visible,
        invalid: state.invalid,
        cursor: state.cursor,
        selection_anchor: state.selection_anchor,
        preferred_column: state.preferred_column,
    }
}

pub fn inspect_textarea_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextAreaState,
    enabled: bool,
) -> TextFieldInspectPalette {
    inspect_textfield_color_palette(mode, theme_mode, style, textfield_state_from(state), enabled)
}

pub fn inspect_textarea_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> TextFieldInspectMetrics {
    inspect_textfield_metrics(mode, theme_mode, size)
}
