//! Autocomplete / combobox chrome — textfield + floating menu tokens.

use gpui_luma::controls::autocomplete::AutocompleteTextBoxAppearance;
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use super::floating_menu::floating_menu_appearance;
use crate::resolve::resolve_color;
use crate::mode::ShadcnModeTokens;

pub(crate) fn autocomplete_textbox_appearance(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> AutocompleteTextBoxAppearance {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        autocomplete_textbox_appearance_from_palette(&ctx, size)
    } else {
        autocomplete_textbox_appearance_from_catalog(&ctx, size)
            .unwrap_or_else(|err| panic!("autocomplete properties: {err}"))
    }
}

fn autocomplete_textbox_appearance_from_palette(
    ctx: &AppearanceContext,
    size: ControlSize,
) -> AutocompleteTextBoxAppearance {
    let palette = ctx.palette();
    AutocompleteTextBoxAppearance {
        status_color: palette.primary.background,
        muted_text_color: palette.app_muted_foreground,
        clear_icon_color: palette.app_muted_foreground,
        clear_icon_hover_color: palette.app_foreground,
        menu: floating_menu_appearance(ctx.tokens, ctx.theme_mode, size),
    }
}

pub(crate) fn autocomplete_textbox_appearance_from_catalog(
    ctx: &AppearanceContext,
    size: ControlSize,
) -> anyhow::Result<AutocompleteTextBoxAppearance> {
    let catalog = ctx.catalog();
    Ok(AutocompleteTextBoxAppearance {
        status_color: resolve_color(catalog, "primary")?,
        muted_text_color: resolve_color(catalog, "muted-foreground")?,
        clear_icon_color: resolve_color(catalog, "muted-foreground")?,
        clear_icon_hover_color: resolve_color(catalog, "foreground")?,
        menu: floating_menu_appearance(ctx.tokens, ctx.theme_mode, size),
    })
}
