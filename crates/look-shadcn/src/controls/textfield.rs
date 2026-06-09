//! Text field property mappings (shadcn / Radix):
//!
//! **Surface** (shadcn Input): `border-input`, light transparent fill, dark `input/30`,
//! `selection:bg-primary`, `selection:text-primary-foreground`, `placeholder:text-muted-foreground`.
//!
//! **Soft** (Radix soft): filled `muted` background, no border, same text/selection tokens.

use gpui_luma::controls::textfield::{TextFieldPalette, TextFieldState};
use gpui_luma::theme::{InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::focus::focus_ring_color;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{StylesheetConfig, embedded_stylesheet, find_textfield_color_rule, resolve_textfield_color_rule};

fn textfield_style_key(style: ShadcnTextFieldStyle) -> &'static str {
    match style {
        ShadcnTextFieldStyle::Surface => "surface",
        ShadcnTextFieldStyle::Soft => "soft",
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ShadcnTextFieldStyle {
    #[default]
    Surface,
    Soft,
}

#[derive(Clone, Debug)]
pub struct TextFieldColorTable {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub placeholder: ResolvedColor,
    pub icon: ResolvedColor,
    pub selection_background: ResolvedColor,
    pub selection_foreground: ResolvedColor,
    pub caret: ResolvedColor,
}

impl TextFieldColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            foreground: ResolvedColor::fallback_foreground(),
            border: ResolvedColor::fallback_foreground(),
            placeholder: ResolvedColor::fallback_foreground(),
            icon: ResolvedColor::fallback_foreground(),
            selection_background: ResolvedColor::fallback_foreground(),
            selection_foreground: ResolvedColor::fallback_foreground(),
            caret: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_textfield_colors(
    resolver: &LookResolver<'_>,
    style: ShadcnTextFieldStyle,
    enabled: bool,
    invalid: bool,
    theme_mode: ThemeMode,
) -> anyhow::Result<TextFieldColorTable> {
    resolve_textfield_colors_with_stylesheet(resolver, embedded_stylesheet(), style, enabled, invalid, theme_mode)
}

pub fn resolve_textfield_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    style: ShadcnTextFieldStyle,
    enabled: bool,
    invalid: bool,
    theme_mode: ThemeMode,
) -> anyhow::Result<TextFieldColorTable> {
    let rule = find_textfield_color_rule(stylesheet, textfield_style_key(style), enabled, invalid, theme_mode)
        .ok_or_else(|| anyhow::anyhow!("no matching textfield color rule"))?;
    let colors = resolve_textfield_color_rule(resolver, rule)?;
    Ok(TextFieldColorTable {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
        placeholder: colors.placeholder,
        icon: colors.icon,
        selection_background: colors.selection_background,
        selection_foreground: colors.selection_foreground,
        caret: colors.caret,
    })
}

pub fn textfield_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
) -> TextFieldPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let typography = ctx.typography();
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "textfield");
    let colors = resolve_textfield_colors(&resolver, style, enabled, state.invalid, ctx.theme_mode)
        .unwrap_or_else(|_| TextFieldColorTable::fallback());
    let focus_ring = if enabled && state.focus_visible {
        Some(focus_ring_color(catalog).unwrap_or_else(|err| panic!("textfield properties: {err}")))
    } else {
        None
    };

    TextFieldPalette {
        background: colors.background.hsla(),
        foreground: colors.foreground.hsla(),
        border: colors.border.hsla(),
        placeholder: colors.placeholder.hsla(),
        icon: colors.icon.hsla(),
        selection_background: colors.selection_background.hsla(),
        selection_foreground: colors.selection_foreground.hsla(),
        caret: colors.caret.hsla(),
        focus_ring,
        typography: typography.text.body,
        font_family: typography.font.sans.family.clone().into(),
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use gpui_luma::controls::textfield::TextFieldState;
    use gpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::color::with_alpha;
    use crate::mode::ShadcnModeTokens;

    use super::{ShadcnTextFieldStyle, textfield_palette};

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
        let appearance =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Surface, TextFieldState::default(), true);

        assert_eq!(appearance.background, gpui::hsla(0.0, 0.0, 0.0, 0.0));
        assert_eq!(appearance.border, catalog.color("input").expect("input"));
        assert_eq!(appearance.foreground, catalog.color("foreground").expect("foreground"));
        assert_eq!(appearance.selection_foreground, catalog.color("primary-foreground").expect("primary-foreground"));
    }

    #[test]
    fn surface_textfield_dark_uses_input_fill_at_thirty_percent() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let appearance =
            textfield_palette(&mode, ThemeMode::Dark, ShadcnTextFieldStyle::Surface, TextFieldState::default(), true);
        let input = catalog.color("input").expect("input");

        assert_eq!(appearance.background, with_alpha(input, 0.30));
        assert_eq!(appearance.border, input);
    }

    #[test]
    fn soft_textfield_uses_muted_fill_and_no_border() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let appearance =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Soft, TextFieldState::default(), true);

        assert_eq!(appearance.background, catalog.color("muted").expect("muted"));
        assert_eq!(appearance.border, gpui::hsla(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn surface_textfield_hover_does_not_change_background() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let default =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Surface, TextFieldState::default(), true);
        let mut hovered = TextFieldState::default();
        hovered.hovered = true;
        let appearance = textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Surface, hovered, true);

        assert_eq!(appearance.background, default.background);
    }
}
