//! Radix text-area theme adapter (Classic / Surface / Soft).

use std::sync::Arc;

use gpui_luma::controls::textarea::{
    TextAreaLook, TextAreaPalette, TextAreaState, TextAreaTemplate, TextAreaTheme, ThemedTextAreaTemplate,
    compose_textarea_look,
};
use gpui_luma::theme::{ControlSize, MetricTokens, StandardBoxScale};

use crate::look::Look;
use crate::textfield::{TextFieldVariant, resolve_text_chrome_palette};

/// Radix `size` prop on Text Area. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextAreaSize {
    One,
    #[default]
    Two,
    Three,
}

impl TextAreaSize {
    pub const ALL: [Self; 3] = [Self::One, Self::Two, Self::Three];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Two => "2",
            Self::Three => "3",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::One => "Size 1",
            Self::Two => "Size 2",
            Self::Three => "Size 3",
        }
    }

    pub fn control_size(self) -> ControlSize {
        match self {
            Self::One => ControlSize::Sm,
            Self::Two => ControlSize::Md,
            Self::Three => ControlSize::Lg,
        }
    }
}

struct TextAreaThemeAdapter {
    look: Look,
    variant: TextFieldVariant,
}

impl TextAreaTheme for TextAreaThemeAdapter {
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
        if matches!(self.variant, TextFieldVariant::Soft | TextFieldVariant::Classic) {
            look.border_width = 0.0;
        }
        look
    }
}

pub fn textarea_theme(look: &Look) -> Arc<dyn TextAreaTheme> {
    textarea_theme_with(look, TextFieldVariant::default())
}

pub fn textarea_theme_with(look: &Look, variant: TextFieldVariant) -> Arc<dyn TextAreaTheme> {
    Arc::new(TextAreaThemeAdapter { look: look.clone(), variant })
}

pub fn textarea_template(look: &Look, variant: TextFieldVariant) -> Arc<dyn TextAreaTemplate> {
    Arc::new(ThemedTextAreaTemplate::new(textarea_theme_with(look, variant)))
}
