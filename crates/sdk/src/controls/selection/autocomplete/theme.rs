use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::floating_menu::{FloatingMenuLook, default_floating_menu_look};
use crate::theme::{ControlSize, ThemeTokens};

#[derive(Clone, Debug)]
pub struct AutocompleteLook {
    pub status_color: Hsla,
    pub muted_text_color: Hsla,
    pub clear_icon_color: Hsla,
    pub clear_icon_hover_color: Hsla,
    pub menu: FloatingMenuLook,
}

pub trait AutocompleteTheme: Send + Sync {
    fn resolve(&self, size: ControlSize) -> AutocompleteLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultAutocompleteTheme {
    tokens: ThemeTokens,
}

pub fn default_autocomplete_theme() -> Arc<dyn AutocompleteTheme> {
    static THEME: OnceLock<Arc<dyn AutocompleteTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultAutocompleteTheme::default())).clone()
}

impl DefaultAutocompleteTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl AutocompleteTheme for DefaultAutocompleteTheme {
    fn resolve(&self, size: ControlSize) -> AutocompleteLook {
        default_autocomplete_look(&self.tokens, size)
    }
}

pub(crate) fn default_autocomplete_look(tokens: &ThemeTokens, size: ControlSize) -> AutocompleteLook {
    AutocompleteLook {
        status_color: tokens.palette.state.selected.background,
        muted_text_color: tokens.palette.surface.floating.foreground,
        clear_icon_color: tokens.palette.surface.floating.foreground,
        clear_icon_hover_color: tokens.palette.form.input.foreground,
        menu: default_floating_menu_look(tokens, size),
    }
}
