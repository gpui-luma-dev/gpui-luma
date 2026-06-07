//! Text field property mappings (shadcn / Radix):
//!
//! **Surface** (shadcn Input): `border-input`, light transparent fill, dark `input/30`,
//! `selection:bg-primary`, `selection:text-primary-foreground`, `placeholder:text-muted-foreground`.
//!
//! **Soft** (Radix soft): filled `muted` background, no border, same text/selection tokens.

use gpui::{Hsla, hsla};

use gpui_luma::controls::textfield::{TextFieldPalette, TextFieldState};
use gpui_luma::theme::{InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::color::with_alpha;
use crate::focus::focus_ring_color;
use crate::mode::ShadcnModeTokens;
use crate::resolve::resolve_color;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ShadcnTextFieldStyle {
    #[default]
    Surface,
    Soft,
}

pub(crate) fn textfield_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
) -> TextFieldPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        textfield_palette_from_palette(&ctx, style, state, enabled)
    } else {
        textfield_palette_from_catalog(&ctx, style, state, enabled)
            .unwrap_or_else(|err| panic!("textfield properties: {err}"))
    }
}

fn surface_background(palette: &crate::palette::ShadcnPalette, theme_mode: ThemeMode) -> Hsla {
    match theme_mode {
        ThemeMode::Light => hsla(0.0, 0.0, 0.0, 0.0),
        ThemeMode::Dark => with_alpha(palette.input_background, 0.30),
    }
}

fn surface_background_from_catalog(
    catalog: &crate::catalog::CssTokenMap,
    theme_mode: ThemeMode,
) -> anyhow::Result<Hsla> {
    match theme_mode {
        ThemeMode::Light => Ok(hsla(0.0, 0.0, 0.0, 0.0)),
        ThemeMode::Dark => Ok(with_alpha(resolve_color(catalog, "input")?, 0.30)),
    }
}

fn shared_textfield_tokens_from_palette(
    palette: &crate::palette::ShadcnPalette,
) -> (Hsla, Hsla, Hsla, Hsla, Hsla, Hsla) {
    (
        palette.app_foreground,
        palette.app_muted_foreground,
        palette.app_muted_foreground,
        palette.selected_background,
        palette.selected_foreground,
        palette.app_foreground,
    )
}

fn shared_textfield_tokens_from_catalog(
    catalog: &crate::catalog::CssTokenMap,
) -> anyhow::Result<(Hsla, Hsla, Hsla, Hsla, Hsla, Hsla)> {
    Ok((
        resolve_color(catalog, "foreground")?,
        resolve_color(catalog, "muted-foreground")?,
        resolve_color(catalog, "muted-foreground")?,
        resolve_color(catalog, "primary")?,
        resolve_color(catalog, "primary-foreground")?,
        resolve_color(catalog, "foreground")?,
    ))
}

fn textfield_palette_from_palette(
    ctx: &AppearanceContext,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
) -> TextFieldPalette {
    let palette = ctx.palette();
    let typography = ctx.typography();
    let theme_mode = ctx.theme_mode;
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);
    let (foreground, placeholder, icon, selection_background, selection_foreground, caret) =
        shared_textfield_tokens_from_palette(palette);

    let (background, border) = match (style, enabled) {
        (ShadcnTextFieldStyle::Surface, true) => {
            let border = if state.invalid {
                palette.focus_ring
            } else {
                palette.input_background
            };
            (surface_background(palette, theme_mode), border)
        }
        (ShadcnTextFieldStyle::Soft, true) => (palette.muted_background, transparent),
        (ShadcnTextFieldStyle::Surface, false) => (palette.disabled_background, palette.input_background),
        (ShadcnTextFieldStyle::Soft, false) => (palette.disabled_background, transparent),
    };

    let (foreground, placeholder, icon, caret) = if enabled {
        (foreground, placeholder, icon, caret)
    } else {
        (
            palette.disabled_foreground,
            palette.disabled_foreground,
            palette.disabled_foreground,
            palette.disabled_foreground,
        )
    };

    TextFieldPalette {
        background,
        foreground,
        border,
        placeholder,
        icon,
        selection_background,
        selection_foreground,
        caret,
        focus_ring: (enabled && state.focus_visible).then_some(palette.focus_ring),
        typography: typography.text.body,
        font_family: typography.font.sans.family.clone().into(),
    }
}

pub(crate) fn textfield_palette_from_catalog(
    ctx: &AppearanceContext,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
) -> anyhow::Result<TextFieldPalette> {
    let catalog = ctx.catalog();
    let typography = ctx.typography();
    let theme_mode = ctx.theme_mode;
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);
    let (foreground, placeholder, icon, selection_background, selection_foreground, caret) =
        shared_textfield_tokens_from_catalog(catalog)?;

    let (background, border) = match (style, enabled) {
        (ShadcnTextFieldStyle::Surface, true) => {
            let border = if state.invalid {
                focus_ring_color(catalog)?
            } else {
                resolve_color(catalog, "input")?
            };
            (surface_background_from_catalog(catalog, theme_mode)?, border)
        }
        (ShadcnTextFieldStyle::Soft, true) => (resolve_color(catalog, "muted")?, transparent),
        (ShadcnTextFieldStyle::Surface, false) => (resolve_color(catalog, "muted")?, resolve_color(catalog, "input")?),
        (ShadcnTextFieldStyle::Soft, false) => (resolve_color(catalog, "muted")?, transparent),
    };

    let disabled_foreground = resolve_color(catalog, "muted-foreground")?;
    let (foreground, placeholder, icon, caret) = if enabled {
        (foreground, placeholder, icon, caret)
    } else {
        (disabled_foreground, disabled_foreground, disabled_foreground, disabled_foreground)
    };

    Ok(TextFieldPalette {
        background,
        foreground,
        border,
        placeholder,
        icon,
        selection_background,
        selection_foreground,
        caret,
        focus_ring: (enabled && state.focus_visible).then(|| focus_ring_color(catalog)).transpose()?,
        typography: typography.text.body,
        font_family: typography.font.sans.family.clone().into(),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui_luma::controls::textfield::TextFieldState;
    use gpui_luma::theme::ThemeMode;

    use crate::appearance_context::AppearanceContext;
    use crate::catalog::CssTokenMap;
    use crate::color::with_alpha;
    use crate::mode::ShadcnModeTokens;

    use super::{ShadcnTextFieldStyle, textfield_palette_from_catalog};

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn surface_textfield_light_uses_transparent_fill_and_input_border() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let ctx = AppearanceContext::new(&mode, ThemeMode::Light, Default::default());
        let appearance =
            textfield_palette_from_catalog(&ctx, ShadcnTextFieldStyle::Surface, TextFieldState::default(), true)
                .expect("textfield");

        assert_eq!(appearance.background, gpui::hsla(0.0, 0.0, 0.0, 0.0));
        assert_eq!(appearance.border, catalog.color("input").expect("input"));
        assert_eq!(appearance.foreground, catalog.color("foreground").expect("foreground"));
        assert_eq!(appearance.selection_foreground, catalog.color("primary-foreground").expect("primary-foreground"));
    }

    #[test]
    fn surface_textfield_dark_uses_input_fill_at_thirty_percent() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let ctx = AppearanceContext::new(&mode, ThemeMode::Dark, Default::default());
        let appearance =
            textfield_palette_from_catalog(&ctx, ShadcnTextFieldStyle::Surface, TextFieldState::default(), true)
                .expect("textfield");
        let input = catalog.color("input").expect("input");

        assert_eq!(appearance.background, with_alpha(input, 0.30));
        assert_eq!(appearance.border, input);
    }

    #[test]
    fn soft_textfield_uses_muted_fill_and_no_border() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let ctx = AppearanceContext::new(&mode, ThemeMode::Light, Default::default());
        let appearance =
            textfield_palette_from_catalog(&ctx, ShadcnTextFieldStyle::Soft, TextFieldState::default(), true)
                .expect("textfield");

        assert_eq!(appearance.background, catalog.color("muted").expect("muted"));
        assert_eq!(appearance.border, gpui::hsla(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn surface_textfield_hover_does_not_change_background() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let ctx = AppearanceContext::new(&mode, ThemeMode::Light, Default::default());
        let default =
            textfield_palette_from_catalog(&ctx, ShadcnTextFieldStyle::Surface, TextFieldState::default(), true)
                .expect("textfield");
        let mut hovered = TextFieldState::default();
        hovered.hovered = true;
        let appearance =
            textfield_palette_from_catalog(&ctx, ShadcnTextFieldStyle::Surface, hovered, true).expect("textfield");

        assert_eq!(appearance.background, default.background);
    }
}
