use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{
    ControlSize, FloatingMenuAppearance, ThemePartUsage, ThemeTokens, ThemeUsage,
    floating_menu::default_floating_menu_appearance,
};

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

pub const AUTOCOMPLETE_TEXTBOX_THEME_USAGE: ThemeUsage = ThemeUsage {
    component: "Autocomplete TextBox",
    parts: &[
        ThemePartUsage {
            part: "status accent",
            token: "state.selected.background",
            states: &["complete", "selected"],
            appearance_fields: &["AutocompleteTextBoxAppearance.status_color"],
        },
        ThemePartUsage {
            part: "muted status text",
            token: "surface.floating.foreground",
            states: &["default"],
            appearance_fields: &[
                "AutocompleteTextBoxAppearance.muted_text_color",
                "AutocompleteTextBoxAppearance.clear_icon_color",
            ],
        },
        ThemePartUsage {
            part: "clear hover icon",
            token: "form.input.foreground",
            states: &["hovered"],
            appearance_fields: &["AutocompleteTextBoxAppearance.clear_icon_hover_color"],
        },
        ThemePartUsage {
            part: "popup menu background",
            token: "surface.floating.background",
            states: &["open"],
            appearance_fields: &["AutocompleteTextBoxAppearance.menu.background"],
        },
        ThemePartUsage {
            part: "popup menu border",
            token: "surface.floating.border",
            states: &["open"],
            appearance_fields: &["AutocompleteTextBoxAppearance.menu.border"],
        },
        ThemePartUsage {
            part: "popup item hover",
            token: "state.hover.background",
            states: &["item hovered", "item highlighted"],
            appearance_fields: &["AutocompleteTextBoxAppearance.menu.item_hover_background"],
        },
    ],
};

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
