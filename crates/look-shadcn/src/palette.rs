use anyhow::{Context as _, Result};
use gpui::{Hsla, hsla};

use gpui_luma::theme::{ActionRolePalette, ThemeMode, ThemeTokens};

use crate::catalog::CssTokenMap;
use crate::state_color::{algorithmic_state_color, catalog_state_color, token_base_from_palette};
use crate::tokens::ShadcnToken;
use gpui_luma::theme::InteractionLayer;

#[derive(Clone, Copy, Debug)]
pub struct ShadcnActionRole {
    pub background: Hsla,
    pub foreground: Hsla,
    pub hover_background: Hsla,
    pub pressed_background: Hsla,
    pub border: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct ShadcnPalette {
    pub primary: ShadcnActionRole,
    pub secondary: ShadcnActionRole,
    pub outline: ShadcnActionRole,
    pub ghost: ShadcnActionRole,
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
    pub accent_background: Hsla,
    pub accent_foreground: Hsla,
    pub destructive_background: Hsla,
    pub destructive_foreground: Hsla,
}

impl ShadcnPalette {
    pub fn from_catalog(catalog: &CssTokenMap, theme_mode: ThemeMode) -> Result<Self> {
        let primary = filled_role(catalog, "primary", "primary-foreground", theme_mode)?;
        let secondary = filled_role(catalog, "secondary", "secondary-foreground", theme_mode)?;
        let outline = outline_role(catalog)?;
        let ghost = ghost_role(catalog)?;
        let accent_background = catalog.optional_color(&["accent"])?.unwrap_or(catalog.color("muted")?);
        let accent_foreground = catalog.optional_color(&["accent-foreground"])?.unwrap_or(catalog.color("foreground")?);
        let destructive_background = catalog.optional_color(&["destructive"])?.unwrap_or(catalog.color("muted")?);
        let destructive_foreground =
            catalog.optional_color(&["destructive-foreground"])?.unwrap_or(catalog.color("muted-foreground")?);

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
            accent_background,
            accent_foreground,
            destructive_background,
            destructive_foreground,
        })
    }

    /// One-time conversion from legacy Luma TOML until import emits shadcn palette directly.
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
            accent_background: palette.action.ghost.hover_background,
            accent_foreground: palette.action.ghost.foreground,
            destructive_background: palette.state.disabled.background,
            destructive_foreground: palette.state.disabled.foreground,
        }
    }
}

fn filled_role(
    catalog: &CssTokenMap,
    background_key: &str,
    foreground_key: &str,
    theme_mode: ThemeMode,
) -> Result<ShadcnActionRole> {
    let background = catalog.color(background_key)?;
    let foreground = catalog.color(foreground_key)?;
    let hover_background = catalog_state_color(catalog, background_key, InteractionLayer::Hovered)
        .unwrap_or_else(|| algorithmic_state_color(background, InteractionLayer::Hovered, theme_mode, true));
    let pressed_background = catalog_state_color(catalog, background_key, InteractionLayer::Pressed)
        .unwrap_or_else(|| algorithmic_state_color(background, InteractionLayer::Pressed, theme_mode, true));
    Ok(ShadcnActionRole { background, foreground, hover_background, pressed_background, border: background })
}

fn outline_role(catalog: &CssTokenMap) -> Result<ShadcnActionRole> {
    let background = catalog.optional_color(&["card", "background"])?.context("card or background")?;
    let foreground = catalog.color("foreground")?;
    let border = catalog.color("border")?;
    let hover_background = catalog.optional_color(&["accent", "muted"])?.context("accent or muted")?;
    let pressed_background = catalog.color("muted")?;
    Ok(ShadcnActionRole { background, foreground, hover_background, pressed_background, border })
}

fn ghost_role(catalog: &CssTokenMap) -> Result<ShadcnActionRole> {
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);
    let foreground = catalog.color("foreground")?;
    let hover_background = catalog.optional_color(&["accent", "muted"])?.context("accent or muted")?;
    let pressed_background = catalog.color("muted")?;
    Ok(ShadcnActionRole {
        background: transparent,
        foreground,
        hover_background,
        pressed_background,
        border: transparent,
    })
}

impl ShadcnPalette {
    pub fn action(&self, style: crate::controls::ShadcnButtonStyle) -> ShadcnActionRole {
        match style {
            crate::controls::ShadcnButtonStyle::Primary => self.primary,
            crate::controls::ShadcnButtonStyle::Secondary => self.secondary,
            crate::controls::ShadcnButtonStyle::Outline => self.outline,
            crate::controls::ShadcnButtonStyle::Ghost => self.ghost,
        }
    }

    pub fn token_color(&self, token: ShadcnToken) -> Hsla {
        token_base_from_palette(self, token)
    }
}

fn action_role_from(role: &ActionRolePalette) -> ShadcnActionRole {
    ShadcnActionRole {
        background: role.background,
        foreground: role.foreground,
        hover_background: role.hover_background,
        pressed_background: role.pressed_background,
        border: role.border,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;

    use super::ShadcnPalette;

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
            ("destructive".into(), "oklch(0.55 0.22 25)".into()),
            ("destructive-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    #[test]
    fn from_catalog_builds_distinct_action_roles() {
        let palette = ShadcnPalette::from_catalog(&sample_catalog(), ThemeMode::Light).expect("tokens should parse");
        assert_ne!(palette.primary.background, palette.outline.background);
        assert_ne!(palette.primary.hover_background, palette.primary.background);
    }

    #[test]
    fn dark_primary_hover_lightens() {
        let palette = ShadcnPalette::from_catalog(&sample_catalog(), ThemeMode::Dark).expect("tokens should parse");
        assert!(palette.primary.hover_background.l > palette.primary.background.l);
    }
}
