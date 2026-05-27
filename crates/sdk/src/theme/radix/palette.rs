use anyhow::{Context as _, Result};
use gpui::{Hsla, hsla};

use super::catalog::CssTokenMap;
use super::color::darken;
use crate::theme::{ActionRolePalette, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct RadixActionRole {
    pub background: Hsla,
    pub foreground: Hsla,
    pub hover_background: Hsla,
    pub pressed_background: Hsla,
    pub border: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct RadixPalette {
    pub primary: RadixActionRole,
    pub secondary: RadixActionRole,
    pub outline: RadixActionRole,
    pub ghost: RadixActionRole,
    pub disabled_background: Hsla,
    pub disabled_foreground: Hsla,
    pub selected_background: Hsla,
    pub selected_foreground: Hsla,
    pub focus_ring: Hsla,
    pub app_background: Hsla,
    pub app_foreground: Hsla,
    pub app_muted_foreground: Hsla,
    pub border_default: Hsla,
    pub input_background: Hsla,
    pub muted_background: Hsla,
    pub panel_background: Hsla,
    pub body_text: Hsla,
}

impl RadixPalette {
    pub fn from_catalog(catalog: &CssTokenMap) -> Result<Self> {
        let primary = filled_role(catalog, "primary", "primary-foreground")?;
        let secondary = filled_role(catalog, "secondary", "secondary-foreground")?;
        let outline = outline_role(catalog)?;
        let ghost = ghost_role(catalog)?;

        Ok(Self {
            primary,
            secondary,
            outline,
            ghost,
            disabled_background: catalog.color("muted")?,
            disabled_foreground: catalog.color("muted-foreground")?,
            selected_background: primary.background,
            selected_foreground: primary.foreground,
            focus_ring: catalog.color("ring")?,
            app_background: catalog.color("background")?,
            app_foreground: catalog.color("foreground")?,
            app_muted_foreground: catalog.color("muted-foreground")?,
            border_default: catalog.color("border")?,
            input_background: catalog.color("input")?,
            muted_background: catalog.color("muted")?,
            panel_background: catalog.optional_color(&["card", "background"])?.context("card or background")?,
            body_text: catalog.color("foreground")?,
        })
    }

    /// One-time conversion from legacy Luma TOML until import emits Radix palette directly.
    pub fn from_luma_tokens(tokens: &ThemeTokens) -> Self {
        let palette = &tokens.palette;

        Self {
            primary: action_role_from(&palette.action.prominent),
            secondary: action_role_from(&palette.action.standard),
            outline: action_role_from(&palette.action.subtle),
            ghost: action_role_from(&palette.action.ghost),
            disabled_background: palette.state.disabled.background,
            disabled_foreground: palette.state.disabled.foreground,
            selected_background: palette.state.selected.background,
            selected_foreground: palette.state.selected.foreground,
            focus_ring: palette.focus.ring,
            app_background: palette.app.background,
            app_foreground: palette.app.foreground,
            app_muted_foreground: palette.app.muted_foreground,
            border_default: palette.border.default,
            input_background: palette.form.input.background,
            muted_background: palette.state.hover.background,
            panel_background: palette.surface.panel.background,
            body_text: palette.surface.subtle.foreground,
        }
    }
}

fn filled_role(catalog: &CssTokenMap, background_key: &str, foreground_key: &str) -> Result<RadixActionRole> {
    let background = catalog.color(background_key)?;
    let foreground = catalog.color(foreground_key)?;
    Ok(RadixActionRole {
        background,
        foreground,
        hover_background: darken(background, 0.06),
        pressed_background: darken(background, 0.12),
        border: background,
    })
}

fn outline_role(catalog: &CssTokenMap) -> Result<RadixActionRole> {
    let background = catalog.optional_color(&["card", "background"])?.context("card or background")?;
    let foreground = catalog.color("foreground")?;
    let border = catalog.color("border")?;
    let hover_background = catalog.optional_color(&["accent", "muted"])?.context("accent or muted")?;
    let pressed_background = catalog.color("muted")?;
    Ok(RadixActionRole { background, foreground, hover_background, pressed_background, border })
}

fn ghost_role(catalog: &CssTokenMap) -> Result<RadixActionRole> {
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);
    let foreground = catalog.color("foreground")?;
    let hover_background = catalog.optional_color(&["accent", "muted"])?.context("accent or muted")?;
    let pressed_background = catalog.color("muted")?;
    Ok(RadixActionRole {
        background: transparent,
        foreground,
        hover_background,
        pressed_background,
        border: transparent,
    })
}

fn action_role_from(role: &ActionRolePalette) -> RadixActionRole {
    RadixActionRole {
        background: role.background,
        foreground: role.foreground,
        hover_background: role.hover_background,
        pressed_background: role.pressed_background,
        border: role.border,
    }
}

impl RadixPalette {
    pub fn action(&self, style: super::RadixButtonStyle) -> RadixActionRole {
        match style {
            super::RadixButtonStyle::Primary => self.primary,
            super::RadixButtonStyle::Secondary => self.secondary,
            super::RadixButtonStyle::Outline => self.outline,
            super::RadixButtonStyle::Ghost => self.ghost,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{CssTokenMap, RadixPalette};

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
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    #[test]
    fn from_catalog_builds_distinct_action_roles() {
        let palette = RadixPalette::from_catalog(&sample_catalog()).expect("tokens should parse");
        assert_ne!(palette.primary.background, palette.outline.background);
        assert_ne!(palette.primary.hover_background, palette.primary.background);
    }
}
