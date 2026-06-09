//! Inspect metadata for `autocomplete`.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{AppearanceContext, ColorSource, LookResolver, ResolvedColor, ShadcnModeTokens};

use super::floating_menu::{FloatingMenuInspectMetrics, FloatingMenuInspectPalette};

pub struct AutocompleteChromeInspectPalette {
    pub status_color: ResolvedColor,
    pub muted_text_color: ResolvedColor,
    pub clear_icon_color: ResolvedColor,
    pub clear_icon_hover_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct AutocompleteInspectMetrics {
    pub textfield: crate::controls::textfield::TextFieldInspectMetrics,
    pub menu: FloatingMenuInspectMetrics,
}

pub fn inspect_autocomplete_chrome_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
) -> AutocompleteChromeInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if ctx.catalog().tokens.is_empty() {
        let appearance =
            gpui_luma_look_shadcn::paint::autocomplete_textbox_appearance_from_palette(&ctx, ControlSize::Md);
        return AutocompleteChromeInspectPalette {
            status_color: resolved_from_hsla(appearance.status_color, ColorSource::CssVar { token: "primary".into() }),
            muted_text_color: resolved_from_hsla(
                appearance.muted_text_color,
                ColorSource::CssVar { token: "muted-foreground".into() },
            ),
            clear_icon_color: resolved_from_hsla(
                appearance.clear_icon_color,
                ColorSource::CssVar { token: "muted-foreground".into() },
            ),
            clear_icon_hover_color: resolved_from_hsla(
                appearance.clear_icon_hover_color,
                ColorSource::CssVar { token: "foreground".into() },
            ),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "autocomplete_chrome_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_autocomplete_chrome_colors(&resolver, true)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::AutocompleteChromeColorTable::fallback());
    AutocompleteChromeInspectPalette {
        status_color: colors.status_color,
        muted_text_color: colors.muted_text_color,
        clear_icon_color: colors.clear_icon_color,
        clear_icon_hover_color: colors.clear_icon_hover_color,
    }
}

pub fn inspect_autocomplete_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> AutocompleteInspectMetrics {
    AutocompleteInspectMetrics {
        textfield: crate::controls::textfield::inspect_textfield_metrics(mode, theme_mode, size),
        menu: crate::controls::floating_menu::inspect_floating_menu_metrics(mode, theme_mode, size),
    }
}

pub fn inspect_autocomplete_menu_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> FloatingMenuInspectPalette {
    crate::controls::floating_menu::inspect_floating_menu_color_palette(mode, theme_mode, size)
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{retro_arcade_catalog, sample_catalog};

    #[test]
    fn autocomplete_chrome_metadata_matches_table() {
        assert_eq!(
            gpui_luma_look_shadcn::stylesheet::resolve_autocomplete_chrome_colors_metadata(
                gpui_luma_look_shadcn::embedded_stylesheet()
            )
            .len(),
            1
        );
    }

    #[test]
    fn inspect_autocomplete_chrome_uses_primary() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let palette = inspect_autocomplete_chrome_color_palette(&mode, ThemeMode::Light);
        assert_eq!(palette.status_color.value, catalog.color("primary").expect("primary"));
    }
}
