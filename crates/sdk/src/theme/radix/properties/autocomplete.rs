//! Autocomplete / combobox chrome — textfield + floating menu tokens.

use crate::controls::autocomplete::AutocompleteTextBoxAppearance;
use crate::theme::{ControlSize, ThemeMode};

use super::floating_menu::floating_menu_appearance;
use super::resolve::resolve_color;
use super::super::catalog::CssTokenMap;
use super::super::mode::RadixModeTokens;
use super::super::palette::RadixPalette;

pub(crate) fn autocomplete_textbox_appearance(
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> AutocompleteTextBoxAppearance {
    if mode.catalog.tokens.is_empty() {
        autocomplete_textbox_appearance_from_palette(&mode.palette, mode, theme_mode, size)
    } else {
        autocomplete_textbox_appearance_from_catalog(&mode.catalog, mode, theme_mode, size)
            .unwrap_or_else(|err| panic!("autocomplete properties: {err}"))
    }
}

fn autocomplete_textbox_appearance_from_palette(
    palette: &RadixPalette,
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> AutocompleteTextBoxAppearance {
    AutocompleteTextBoxAppearance {
        status_color: palette.primary.background,
        muted_text_color: palette.app_muted_foreground,
        clear_icon_color: palette.app_muted_foreground,
        clear_icon_hover_color: palette.app_foreground,
        menu: floating_menu_appearance(mode, theme_mode, size),
    }
}

pub(crate) fn autocomplete_textbox_appearance_from_catalog(
    catalog: &CssTokenMap,
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> anyhow::Result<AutocompleteTextBoxAppearance> {
    Ok(AutocompleteTextBoxAppearance {
        status_color: resolve_color(catalog, "primary")?,
        muted_text_color: resolve_color(catalog, "muted-foreground")?,
        clear_icon_color: resolve_color(catalog, "muted-foreground")?,
        clear_icon_hover_color: resolve_color(catalog, "foreground")?,
        menu: floating_menu_appearance(mode, theme_mode, size),
    })
}
