use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::floating_menu::{FloatingMenuLook, default_floating_menu_look};
use crate::theme::{ControlSize, ThemeTokens};

#[derive(Clone, Debug)]
pub struct AutocompleteLook {
    pub status_color: Hsla,
    /// Status text comes from this theme, rather than a template-local default.
    pub status_typography: crate::theme::LumaTextStyle,
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
    let mut look = AutocompleteLook {
        status_color: tokens.palette.state.selected.background,
        status_typography: tokens.typography.text.scale.sm,
        muted_text_color: tokens.palette.surface.floating.foreground,
        clear_icon_color: tokens.palette.surface.floating.foreground,
        clear_icon_hover_color: tokens.palette.form.input.foreground,
        menu: default_floating_menu_look(tokens, size),
    };
    let geometry = tokens.stylesheet.autocomplete.resolve_geometry(
        size.stylesheet_key(),
        crate::theme::stylesheet::AutocompleteGeometry {
            status_font_size: look.status_typography.size,
            status_line_height: look.status_typography.line_height,
            ..Default::default()
        },
    );
    look.status_typography.size = geometry.status_font_size.value_px;
    look.status_typography.line_height = geometry.status_line_height.value_px;
    look
}

#[cfg(test)]
mod stylesheet_tests {
    use super::*;
    use crate::{
        color::GamutMapping,
        theme::{ThemeMode, stylesheet::CommonStylesheet},
    };

    #[test]
    fn status_typography_comes_from_theme_tokens_then_common_overrides() {
        let sheet = CommonStylesheet::resolve_layers(&[
            "[common.autocomplete.geometry]\nstatus_font_size=19\nstatus_line_height=27",
        ])
        .unwrap();
        let mut tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
        tokens.typography.text.scale.sm.weight = gpui::FontWeight::BOLD;
        let theme = DefaultAutocompleteTheme::new(tokens);
        for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
            let look = theme.resolve(size);
            assert_eq!(
                (look.status_typography.size, look.status_typography.line_height, look.status_typography.weight),
                (19.0, 27.0, gpui::FontWeight::BOLD)
            );
        }
    }
}
