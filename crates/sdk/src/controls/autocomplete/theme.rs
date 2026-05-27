use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::floating_menu::{FloatingMenuAppearance, default_floating_menu_appearance};
use crate::theme::{ControlSize, ThemeTokens};

#[derive(Clone, Debug)]
pub struct AutocompleteTextBoxAppearance {
    pub status_color: Hsla,
    pub muted_text_color: Hsla,
    pub clear_icon_color: Hsla,
    pub clear_icon_hover_color: Hsla,
    pub menu: FloatingMenuAppearance,
}

pub trait AutocompleteTextBoxTheme: Send + Sync {
    fn resolve(&self) -> AutocompleteTextBoxAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultAutocompleteTextBoxTheme {
    tokens: ThemeTokens,
}

pub fn default_autocomplete_textbox_theme() -> Arc<dyn AutocompleteTextBoxTheme> {
    static THEME: OnceLock<Arc<dyn AutocompleteTextBoxTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultAutocompleteTextBoxTheme::default())).clone()
}

impl DefaultAutocompleteTextBoxTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl AutocompleteTextBoxTheme for DefaultAutocompleteTextBoxTheme {
    fn resolve(&self) -> AutocompleteTextBoxAppearance {
        default_autocomplete_textbox_appearance(&self.tokens, ControlSize::Md)
    }
}

pub(crate) fn default_autocomplete_textbox_appearance(
    tokens: &ThemeTokens,
    size: ControlSize,
) -> AutocompleteTextBoxAppearance {
    AutocompleteTextBoxAppearance {
        status_color: tokens.palette.state.selected.background,
        muted_text_color: tokens.palette.surface.floating.foreground,
        clear_icon_color: tokens.palette.surface.floating.foreground,
        clear_icon_hover_color: tokens.palette.form.input.foreground,
        menu: default_floating_menu_appearance(tokens, size),
    }
}
