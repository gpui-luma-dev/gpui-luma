use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

use gpui::{BoxShadow, Hsla, SharedString};
use gpui_luma::theme::{LumaTheme, ThemeMode};
use crate::catalog::{CssTokenCatalog, parse_css_catalog};
use crate::mode::ShadcnModeTokens;

use crate::shadow::parse_shadow_token;
use crate::tokens::{ShadcnFont, ShadcnRadius, ShadcnShadow, ShadcnToken};

const GENERIC_FAMILIES: &[&str] = &[
    "ui-sans-serif",
    "ui-serif",
    "ui-monospace",
    "system-ui",
    "-apple-system",
    "BlinkMacSystemFont",
    "sans-serif",
    "serif",
    "monospace",
    "cursive",
    "fantasy",
];

/// Runtime shadcn appearance resolver backed by parsed CSS token catalogs.
#[derive(Clone)]
pub struct ShadcnLook {
    state: Arc<ShadcnLookState>,
}

struct ShadcnLookState {
    catalog: CssTokenCatalog,
    light: ShadcnModeTokens,
    dark: ShadcnModeTokens,
    mode: AtomicU8,
}

impl ShadcnLook {
    pub fn native() -> Self {
        Self::from_theme(LumaTheme::native())
    }

    /// Loads a theme file. Legacy Luma-shaped TOML is converted once at the boundary.
    pub fn from_toml_str(source: &str) -> anyhow::Result<Self> {
        Ok(Self::from_theme(LumaTheme::from_toml_str(source)?))
    }

    /// Loads shadcn palette tokens from tweakcn-style CSS (`:root` / `.dark` custom properties).
    pub fn from_css_str(source: &str) -> anyhow::Result<Self> {
        let catalog = parse_css_catalog(source)?;
        Ok(Self {
            state: Arc::new(ShadcnLookState {
                light: ShadcnModeTokens::from_catalog(catalog.light_map())?,
                dark: ShadcnModeTokens::from_catalog(catalog.dark_map())?,
                catalog,
                mode: AtomicU8::new(mode_to_u8(ThemeMode::Light)),
            }),
        })
    }

    pub fn from_css_path(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let source = std::fs::read_to_string(path.as_ref())
            .map_err(|err| anyhow::anyhow!("read shadcn theme css {}: {err}", path.as_ref().display()))?;
        Self::from_css_str(&source)
    }

    pub fn from_theme(theme: LumaTheme) -> Self {
        Self {
            state: Arc::new(ShadcnLookState {
                catalog: CssTokenCatalog { light: Default::default(), dark: Default::default() },
                light: ShadcnModeTokens::from_luma_tokens(theme.mode(ThemeMode::Light)),
                dark: ShadcnModeTokens::from_luma_tokens(theme.mode(ThemeMode::Dark)),
                mode: AtomicU8::new(mode_to_u8(ThemeMode::Light)),
            }),
        }
    }

    pub fn mode(&self) -> ThemeMode {
        u8_to_mode(self.state.mode.load(Ordering::Relaxed))
    }

    pub fn set_mode(&self, mode: ThemeMode) {
        self.state.mode.store(mode_to_u8(mode), Ordering::Relaxed);
    }

    pub fn mode_tokens(&self) -> &ShadcnModeTokens {
        match self.mode() {
            ThemeMode::Light => &self.state.light,
            ThemeMode::Dark => &self.state.dark,
        }
    }

    pub fn catalog(&self) -> &CssTokenCatalog {
        &self.state.catalog
    }

    /// `true` when the look was loaded from tweakcn/shadcn CSS (non-empty catalog).
    pub fn has_css_catalog(&self) -> bool {
        !self.state.light.catalog.tokens.is_empty() || !self.state.dark.catalog.tokens.is_empty()
    }

    pub fn token(&self, name: &str) -> Option<&str> {
        self.mode_tokens().catalog.get(name)
    }

    /// Resolves colors directly from the loaded palette.
    pub fn color(&self, token: ShadcnToken) -> Hsla {
        let palette = &self.mode_tokens().palette;
        match token {
            ShadcnToken::Background => palette.app_background,
            ShadcnToken::Foreground => palette.app_foreground,
            ShadcnToken::Card => palette.panel_background,
            ShadcnToken::CardForeground => palette.body_text,
            ShadcnToken::Popover => palette.panel_background,
            ShadcnToken::PopoverForeground => palette.body_text,
            ShadcnToken::Primary => palette.primary.background,
            ShadcnToken::PrimaryForeground => palette.primary.foreground,
            ShadcnToken::Secondary => palette.secondary.background,
            ShadcnToken::SecondaryForeground => palette.secondary.foreground,
            ShadcnToken::Muted => palette.muted_background,
            ShadcnToken::MutedForeground => palette.app_muted_foreground,
            ShadcnToken::Accent => palette.ghost.hover_background,
            ShadcnToken::AccentForeground => palette.ghost.foreground,
            ShadcnToken::Destructive => palette.disabled_background,
            ShadcnToken::DestructiveForeground => palette.disabled_foreground,
            ShadcnToken::Border => palette.border_default,
            ShadcnToken::Input => palette.input_background,
            ShadcnToken::Ring => palette.focus_ring,
        }
    }

    /// Resolves fonts to their loaded families.
    pub fn font(&self, role: ShadcnFont) -> SharedString {
        let key = match role {
            ShadcnFont::Sans => "font-sans",
            ShadcnFont::Serif => "font-serif",
            ShadcnFont::Mono => "font-mono",
        };
        self.token(key).map(first_font_family).unwrap_or_else(|| match role {
            ShadcnFont::Sans => SharedString::from("sans-serif"),
            ShadcnFont::Serif => SharedString::from("serif"),
            ShadcnFont::Mono => SharedString::from("monospace"),
        })
    }

    /// Resolves border-radius to raw pixels based on the current theme radius.
    pub fn radius(&self, role: ShadcnRadius) -> f32 {
        let base_radius = self.parse_pixel_token("radius").unwrap_or(8.0);
        match role {
            ShadcnRadius::None => 0.0,
            ShadcnRadius::Sm => (base_radius - 4.0).max(0.0),
            ShadcnRadius::Md => (base_radius - 2.0).max(0.0),
            ShadcnRadius::Lg => base_radius,
            ShadcnRadius::Xl => base_radius + 4.0,
        }
    }

    /// Resolves custom shadow structures for GPUI box-shadow arrays.
    pub fn shadow(&self, role: ShadcnShadow) -> Vec<BoxShadow> {
        let shadow_key = match role {
            ShadcnShadow::None => return vec![],
            ShadcnShadow::TwoXs => "shadow-2xs",
            ShadcnShadow::Xs => "shadow-xs",
            ShadcnShadow::Sm => "shadow-sm",
            ShadcnShadow::Default => "shadow",
            ShadcnShadow::Md => "shadow-md",
            ShadcnShadow::Lg => "shadow-lg",
            ShadcnShadow::Xl => "shadow-xl",
            ShadcnShadow::TwoXl => "shadow-2xl",
        };

        self.parse_shadow_token(shadow_key).unwrap_or_default()
    }

    pub fn parse_pixel_token(&self, name: &str) -> Option<f32> {
        self.token(name).and_then(parse_length_px)
    }

    pub fn parse_shadow_token(&self, token_key: &str) -> anyhow::Result<Vec<BoxShadow>> {
        parse_shadow_token(&self.mode_tokens().catalog, token_key)
    }
}

fn mode_to_u8(mode: ThemeMode) -> u8 {
    match mode {
        ThemeMode::Light => 0,
        ThemeMode::Dark => 1,
    }
}

fn u8_to_mode(value: u8) -> ThemeMode {
    match value {
        0 => ThemeMode::Light,
        _ => ThemeMode::Dark,
    }
}

fn parse_length_px(raw: &str) -> Option<f32> {
    let value = raw.trim();
    if let Some(rem) = value.strip_suffix("rem") {
        return rem.trim().parse::<f32>().ok().map(|n| n * 16.0);
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse().ok();
    }
    value.parse().ok()
}

fn normalize_font_family(family: &str) -> SharedString {
    if family.eq_ignore_ascii_case("Rajdhani") {
        return SharedString::from("Rajdhani Variable");
    }
    SharedString::from(family.to_string())
}

fn first_font_family(raw: &str) -> SharedString {
    for part in raw.split(',') {
        let candidate = part.trim().trim_matches(['\'', '"']);
        if candidate.is_empty() {
            continue;
        }
        if GENERIC_FAMILIES.iter().any(|generic| candidate.eq_ignore_ascii_case(generic)) {
            continue;
        }
        return normalize_font_family(candidate);
    }

    normalize_font_family(raw.split(',').next().unwrap_or(raw).trim().trim_matches(['\'', '"']))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_look_resolves_primary_color() {
        let look = ShadcnLook::native();
        let primary = look.color(ShadcnToken::Primary);
        assert!(primary.a > 0.0);
    }

    #[test]
    fn radius_scales_from_base_token() {
        let look = ShadcnLook::native();
        assert!(look.radius(ShadcnRadius::Lg) >= look.radius(ShadcnRadius::Sm));
    }
}
