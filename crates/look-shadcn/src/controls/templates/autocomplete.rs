use std::sync::Arc;

use gpui_luma::controls::autocomplete::AutocompleteTheme;
use gpui_luma::theme::ControlSize;

use crate::controls::autocomplete::autocomplete_textbox_look;
use crate::look::ShadcnLook;

struct ShadcnAutocompleteTheme {
    theme: ShadcnLook,
}

impl AutocompleteTheme for ShadcnAutocompleteTheme {
    fn resolve(&self, size: ControlSize) -> gpui_luma::controls::autocomplete::AutocompleteLook {
        let tokens = self.theme.mode_tokens();
        autocomplete_textbox_look(tokens.as_ref(), self.theme.mode(), size)
    }
}

pub fn autocomplete_theme(theme: ShadcnLook) -> Arc<dyn AutocompleteTheme> {
    Arc::new(ShadcnAutocompleteTheme { theme: theme.clone() })
}
