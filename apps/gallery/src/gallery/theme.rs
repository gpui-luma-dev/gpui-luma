use std::ffi::OsStr;
use std::path::PathBuf;

pub(in crate::gallery) use gpui_luma::theme::{LumaChrome as GalleryChrome, LumaThemePack as GalleryThemePack};

/// `apps/gallery/src/assets/themes/`
pub(in crate::gallery) fn themes_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/assets/themes")
}

fn theme_toml_path(stem: &str) -> PathBuf {
    themes_dir().join(format!("{stem}.toml"))
}

/// Lists theme file stems (e.g. `retro-arcade`) sorted for usage text.
pub(in crate::gallery) fn available_theme_names() -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(themes_dir()) else {
        return Vec::new();
    };

    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension() == Some(OsStr::new("toml")))
        .filter_map(|path| path.file_stem().and_then(|s| s.to_str()).map(str::to_string))
        .collect();
    names.sort();
    names
}

/// Startup theme: `default` or a file stem under [`themes_dir`] (e.g. `retro-arcade`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GalleryThemeChoice {
    /// SDK [`GalleryThemePack::new`] / `default-theme.toml`.
    Default,
    /// `themes/<stem>.toml` loaded at runtime.
    Named(String),
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
        let stem = arg.trim().to_ascii_lowercase();
        if stem == "default" {
            return Ok(Self::Default);
        }

        let path = theme_toml_path(&stem);
        if path.is_file() {
            return Ok(Self::Named(stem));
        }

        let available = available_theme_names();
        let hint = if available.is_empty() {
            format!("no .toml themes found in {}", themes_dir().display())
        } else {
            format!("available: {}", available.join(", "))
        };

        Err(format!("unknown gallery theme {arg:?} ({hint})\n{}", Self::usage_line()))
    }

    fn usage_line() -> String {
        let program = std::env::args().next().unwrap_or_else(|| "gpui-luma-gallery".into());
        let mut options = vec!["default".to_string()];
        options.extend(available_theme_names());
        format!("usage: {program} [{}]", options.join("|"))
    }

    /// Jarvis bundles Rajdhani; load the font when that theme is selected.
    pub fn loads_rajdhani_font(&self) -> bool {
        matches!(self, Self::Named(stem) if stem == "jarvis")
    }

    pub fn theme_pack(self) -> GalleryThemePack {
        match self {
            Self::Default => GalleryThemePack::new(),
            Self::Named(stem) => {
                let path = theme_toml_path(&stem);
                let source = std::fs::read_to_string(&path)
                    .unwrap_or_else(|err| panic!("read gallery theme {}: {err}", path.display()));
                GalleryThemePack::from_toml_str(&source)
                    .unwrap_or_else(|err| panic!("parse gallery theme {}: {err}", path.display()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui_luma::theme::{LumaTheme, ThemeMode};

    use super::{available_theme_names, theme_toml_path, themes_dir};

    fn read_theme(stem: &str) -> String {
        std::fs::read_to_string(theme_toml_path(stem)).expect("read theme toml")
    }

    #[test]
    fn themes_dir_exists_and_lists_imported_samples() {
        assert!(themes_dir().is_dir());
        let names = available_theme_names();
        assert!(names.iter().any(|name| name == "astrovista"));
        assert!(names.iter().any(|name| name == "jarvis"));
        assert!(names.iter().any(|name| name == "retro-arcade"));
    }

    #[test]
    fn parse_accepts_theme_stem_case_insensitively() {
        assert_eq!(
            super::GalleryThemeChoice::parse("Retro-Arcade").expect("parse"),
            super::GalleryThemeChoice::Named("retro-arcade".into())
        );
    }

    #[test]
    fn jarvis_theme_sans_family_matches_embedded_font() {
        use crate::fonts::RAJDHANI_FAMILY;

        let theme = LumaTheme::from_toml_str(&read_theme("jarvis")).expect("jarvis theme should parse");
        assert_eq!(theme.mode(ThemeMode::Light).typography.font.sans.family, RAJDHANI_FAMILY);
    }

    #[test]
    fn astrovista_theme_parses_with_distinct_action_roles() {
        let theme = LumaTheme::from_toml_str(&read_theme("astrovista")).expect("astrovista theme should parse");
        let light = theme.mode(ThemeMode::Light);

        assert_eq!(theme.name, "Astrovista");
        assert_eq!(light.palette.action.subtle.background, light.palette.surface.panel.background);
        assert_ne!(light.palette.action.subtle.background, light.palette.action.standard.background);
        assert_ne!(light.palette.action.prominent.background, light.palette.action.standard.background);
        assert_eq!(light.palette.app.background, light.palette.action.ghost.background);
    }
}
