//! Text field property mappings for shadcn-inspired inputs:
//!
//! **Outline** (shadcn Input): `border-border`, light transparent fill, dark `input/30`,
//! `selection:bg-primary`, `selection:text-primary-foreground`, `placeholder:text-muted-foreground`.
//!
//! **Input** (selector / combobox triggers): same fill as Outline (`transparent` light,
//! `input/30` dark; hover `input/50` in selector palette). Opaque controls use `border-border`.
//! Used by selector, combobox, search_selector, and autocomplete — not standalone text fields.
//!
//! **Primary**: bordered control with opaque `background` fill and `shadow-xs` elevation.
//!
//! **Surface**: filled `muted` background, no border, same text/selection tokens.

use gpui_luma::controls::textfield::{TextFieldLook, TextFieldPalette, TextFieldState, compose_textfield_look};
use gpui_luma::theme::{ControlSize, InteractionState, LumaTextStyle, StandardBoxScale, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::shadow::parse_shadow_token;
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_textfield_color_rule, find_textfield_elevation_rule,
    resolve_textfield_color_rule, resolve_stylesheet_shadow_token,
};

fn textfield_style_key(style: ShadcnTextFieldStyle) -> &'static str {
    match style {
        ShadcnTextFieldStyle::Outline => "outline",
        ShadcnTextFieldStyle::Input => "input",
        ShadcnTextFieldStyle::Primary => "primary",
        ShadcnTextFieldStyle::Surface => "surface",
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ShadcnTextFieldStyle {
    #[default]
    Outline,
    Input,
    Primary,
    Surface,
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

pub(crate) fn textfield_elevation_shadow(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnTextFieldStyle,
) -> Option<Vec<gpui::BoxShadow>> {
    let rule = find_textfield_elevation_rule(stylesheet, style)?;
    let token = resolve_stylesheet_shadow_token(&rule.shadow)?;
    let shadows = parse_shadow_token(ctx.catalog(), &token).ok()?;
    if shadows.is_empty() { None } else { Some(shadows) }
}

pub fn textfield_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
) -> TextFieldPalette {
    textfield_palette_for_size(mode, theme_mode, style, state, enabled, ControlSize::Md)
}

pub fn textfield_palette_for_size(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
    size: ControlSize,
) -> TextFieldPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let stylesheet = embedded_stylesheet();
    let catalog = ctx.catalog();
    let typography = ctx.typography();
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "textfield");
    let mut colors = resolve_textfield_colors(&resolver, style, enabled, state.invalid, ctx.theme_mode)
        .unwrap_or_else(|_| TextFieldColorTable::fallback());
    if state.focused && state.focus_visible && !state.invalid && enabled {
        if let Ok(focus_border) = resolver.resolve_decl("ring") {
            colors.border = focus_border;
        }
    }
    // Keep elevation in the look when disabled so SDK hosts can reserve projection
    // space; templates gate paint with `enabled` / `should_paint_shadow`.
    let shadow = textfield_elevation_shadow(&ctx, stylesheet, style);

    let mut background = colors.background.hsla();
    if style == ShadcnTextFieldStyle::Input
        && theme_mode == ThemeMode::Dark
        && enabled
        && state.hovered
        && let Ok(hover_fill) = resolver.resolve_decl("input/50")
    {
        background = hover_fill.hsla();
    }

    let mut text_style = typography.text.body;
    apply_textfield_control_size_typography(&mut text_style, mode, size);

    TextFieldPalette {
        background,
        foreground: colors.foreground.hsla(),
        border: colors.border.hsla(),
        placeholder: colors.placeholder.hsla(),
        icon: colors.icon.hsla(),
        selection_background: colors.selection_background.hsla(),
        selection_foreground: colors.selection_foreground.hsla(),
        caret: colors.caret.hsla(),
        shadow,
        typography: text_style,
        font_family: typography.font.sans.family.clone().into(),
    }
}

/// Scales text-field typography from `button.metrics.*.font_size` (same curve as buttons/selectors).
pub fn apply_textfield_control_size_typography(
    typography: &mut LumaTextStyle,
    mode: &ShadcnModeTokens,
    size: ControlSize,
) {
    super::apply_button_metrics_typography(typography, mode, size);
}

pub fn textfield_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
    size: ControlSize,
    scale: &StandardBoxScale,
) -> TextFieldLook {
    let palette = textfield_palette_for_size(mode, theme_mode, style, state, enabled, size);
    compose_textfield_look(&palette, scale, mode.metrics.border_width.default)
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use gpui_luma::controls::textfield::TextFieldState;
    use gpui_luma::theme::{ControlSize, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::color::with_alpha;
    use crate::mode::ShadcnModeTokens;

    use super::{ShadcnTextFieldStyle, textfield_palette, textfield_palette_for_size};

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("card".into(), "oklch(0.98 0.02 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.7200 0.0120 205.0000)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("shadow-xs".into(), "0 1px 2px 0px hsl(0 0% 0% / 0.05)".into()),
        ]))
    }

    #[test]
    fn input_textfield_light_uses_transparent_fill_and_border_token() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Input, TextFieldState::default(), true);

        assert_eq!(look.background, gpui::hsla(0.0, 0.0, 0.0, 0.0));
        assert_eq!(look.border, catalog.color("border").expect("border"));
        assert!(look.shadow.is_none());
    }

    #[test]
    fn input_textfield_dark_uses_input_fill_and_border_token() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Dark, ShadcnTextFieldStyle::Input, TextFieldState::default(), true);
        let input = catalog.color("input").expect("input");

        assert_eq!(look.background, with_alpha(input, 0.30));
        assert_eq!(look.border, catalog.color("border").expect("border"));
        assert!(look.shadow.is_none());
    }

    #[test]
    fn input_textfield_dark_hover_uses_input_half_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let default =
            textfield_palette(&mode, ThemeMode::Dark, ShadcnTextFieldStyle::Input, TextFieldState::default(), true);
        let mut hovered = TextFieldState::default();
        hovered.hovered = true;
        let look = textfield_palette(&mode, ThemeMode::Dark, ShadcnTextFieldStyle::Input, hovered, true);
        let input = catalog.color("input").expect("input");

        assert_eq!(default.background, with_alpha(input, 0.30));
        assert_eq!(look.background, with_alpha(input, 0.50));
    }

    #[test]
    fn outline_textfield_light_uses_transparent_fill_and_border_token() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Outline, TextFieldState::default(), true);

        assert_eq!(look.background, gpui::hsla(0.0, 0.0, 0.0, 0.0));
        assert_eq!(look.border, catalog.color("border").expect("border"));
        assert_eq!(look.foreground, catalog.color("foreground").expect("foreground"));
        assert_eq!(look.selection_foreground, catalog.color("primary-foreground").expect("primary-foreground"));
        assert!(look.shadow.is_none());
    }

    #[test]
    fn primary_textfield_light_uses_background_fill_and_shadow() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Primary, TextFieldState::default(), true);

        assert_eq!(look.background, catalog.color("background").expect("background"));
        assert_eq!(look.border, catalog.color("border").expect("border"));
        assert!(look.shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn primary_textfield_disabled_retains_elevation_shadow_for_layout() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Primary, TextFieldState::default(), false);

        assert_eq!(look.background, catalog.color("muted").expect("muted"));
        assert!(look.shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn outline_textfield_dark_uses_input_fill_and_border_token() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Dark, ShadcnTextFieldStyle::Outline, TextFieldState::default(), true);
        let input = catalog.color("input").expect("input");

        assert_eq!(look.background, with_alpha(input, 0.30));
        assert_eq!(look.border, catalog.color("border").expect("border"));
    }

    #[test]
    fn surface_textfield_uses_muted_fill_and_no_border() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Surface, TextFieldState::default(), true);

        assert_eq!(look.background, catalog.color("muted").expect("muted"));
        assert_eq!(look.border, gpui::hsla(0.0, 0.0, 0.0, 0.0));
        assert!(look.shadow.is_none());
    }

    #[test]
    fn outline_textfield_hover_does_not_change_background() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let default =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Outline, TextFieldState::default(), true);
        let mut hovered = TextFieldState::default();
        hovered.hovered = true;
        let look = textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Outline, hovered, true);

        assert_eq!(look.background, default.background);
    }

    #[test]
    fn palette_typography_uses_stylesheet_font_size() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let sm = textfield_palette_for_size(
            &mode,
            ThemeMode::Light,
            ShadcnTextFieldStyle::Outline,
            TextFieldState::default(),
            true,
            ControlSize::Sm,
        );
        let md = textfield_palette_for_size(
            &mode,
            ThemeMode::Light,
            ShadcnTextFieldStyle::Outline,
            TextFieldState::default(),
            true,
            ControlSize::Md,
        );
        let lg = textfield_palette_for_size(
            &mode,
            ThemeMode::Light,
            ShadcnTextFieldStyle::Outline,
            TextFieldState::default(),
            true,
            ControlSize::Lg,
        );

        assert!((sm.typography.size - 12.0).abs() < f32::EPSILON);
        assert!((md.typography.size - 14.0).abs() < f32::EPSILON);
        assert!((lg.typography.size - 16.0).abs() < f32::EPSILON);
        assert!(lg.typography.line_height > md.typography.line_height);
        assert!(md.typography.line_height > sm.typography.line_height);
    }
}
