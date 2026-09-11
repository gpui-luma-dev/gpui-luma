//! Radix text-area theme adapter (Classic / Surface / Soft).

use std::sync::Arc;

use luma::controls::textarea::{TextAreaLook, TextAreaPalette, TextAreaState, TextAreaTheme, compose_textarea_look};
use luma::theme::{ControlSize, MetricTokens, StandardBoxScale};

use crate::look::RadixLook;
use crate::textfield::{RadixTextFieldVariant, resolve_text_chrome_palette};

struct RadixTextAreaTheme {
    look: RadixLook,
    variant: RadixTextFieldVariant,
}

impl TextAreaTheme for RadixTextAreaTheme {
    fn resolve(&self, state: TextAreaState, enabled: bool) -> TextAreaPalette {
        let field =
            resolve_text_chrome_palette(&self.look, self.variant, state.hovered, state.focused, state.invalid, enabled);
        TextAreaPalette {
            background: field.background,
            foreground: field.foreground,
            border: field.border,
            placeholder: field.placeholder,
            selection_background: field.selection_background,
            selection_foreground: field.selection_foreground,
            caret: field.caret,
            shadow: field.shadow,
            typography: field.typography,
            font_family: field.font_family.to_string(),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn resolve_look(
        &self,
        state: TextAreaState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> TextAreaLook {
        let mut look = compose_textarea_look(&self.resolve(state, enabled), scale, self.metrics().border_width.default);
        let _ = size;
        if matches!(self.variant, RadixTextFieldVariant::Soft | RadixTextFieldVariant::Classic) {
            look.border_width = 0.0;
        }
        look
    }
}

pub fn textarea_theme(look: Arc<RadixLook>) -> Arc<dyn TextAreaTheme> {
    textarea_theme_with(look, RadixTextFieldVariant::default())
}

pub fn textarea_theme_with(look: Arc<RadixLook>, variant: RadixTextFieldVariant) -> Arc<dyn TextAreaTheme> {
    Arc::new(RadixTextAreaTheme { look: look.as_ref().clone(), variant })
}
