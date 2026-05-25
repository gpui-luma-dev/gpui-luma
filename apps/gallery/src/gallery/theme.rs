pub(in crate::gallery) use gpui_luma::theme::{LumaChrome as GalleryChrome, LumaThemePack as GalleryThemePack};

/// Gallery-imported themes (SDK keeps [`gpui_luma::theme::LumaTheme::native`] / `default-theme.toml` only).
pub(in crate::gallery) mod themes {
    pub const ASTROVISTA: &str = include_str!("../assets/themes/tweakcn-astrovista.toml");
    pub const JARVIS: &str = include_str!("../assets/themes/tweakcn-jarvis.toml");
}

/// Startup theme selected via CLI (`default` | `astrovista` | `jarvis`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalleryThemeChoice {
    /// SDK [`GalleryThemePack::new`] / `default-theme.toml`.
    Default,
    Astrovista,
    Jarvis,
}

impl GalleryThemeChoice {
    /// First positional argument, case-insensitive. Missing arg → [`Self::Default`].
    pub fn from_args() -> Self {
        match std::env::args().nth(1).as_deref() {
            None => Self::Default,
            Some(arg) => Self::parse(arg).unwrap_or_else(|usage| {
                eprintln!("{usage}");
                std::process::exit(1);
            }),
        }
    }

    fn parse(arg: &str) -> Result<Self, String> {
        match arg.to_ascii_lowercase().as_str() {
            "default" => Ok(Self::Default),
            "astrovista" => Ok(Self::Astrovista),
            "jarvis" => Ok(Self::Jarvis),
            other => Err(format!(
                "unknown gallery theme {other:?}\n{}",
                Self::usage_line()
            )),
        }
    }

    fn usage_line() -> String {
        let program = std::env::args().next().unwrap_or_else(|| "gpui-luma-gallery".into());
        format!("usage: {program} [default|astrovista|jarvis]")
    }

    pub fn theme_pack(self) -> GalleryThemePack {
        match self {
            Self::Default => GalleryThemePack::new(),
            Self::Astrovista => GalleryThemePack::from_toml_str(themes::ASTROVISTA)
                .expect("gallery tweakcn-astrovista theme should parse"),
            Self::Jarvis => GalleryThemePack::from_toml_str(themes::JARVIS)
                .expect("gallery tweakcn-jarvis theme should parse"),
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui_luma::theme::{LumaTheme, ThemeMode};

    use super::themes::ASTROVISTA;

    #[test]
    fn jarvis_theme_sans_family_matches_embedded_font() {
        use super::themes::JARVIS;

        use crate::fonts::RAJDHANI_FAMILY;

        let theme = LumaTheme::from_toml_str(JARVIS).expect("jarvis theme should parse");
        assert_eq!(theme.mode(ThemeMode::Light).typography.font.sans.family, RAJDHANI_FAMILY);
    }

    #[test]
    fn astrovista_theme_parses_with_distinct_action_roles() {
        let theme = LumaTheme::from_toml_str(ASTROVISTA).expect("astrovista theme should parse");
        let light = theme.mode(ThemeMode::Light);

        assert_eq!(theme.name, "Astrovista");
        assert_eq!(light.palette.action.subtle.background, light.palette.surface.panel.background);
        assert_ne!(light.palette.action.subtle.background, light.palette.action.standard.background);
        assert_ne!(light.palette.action.prominent.background, light.palette.action.standard.background);
        assert_eq!(light.palette.app.background, light.palette.action.ghost.background);
    }
}
