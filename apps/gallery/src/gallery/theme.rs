use std::ffi::OsStr;
use std::path::PathBuf;

use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;

pub(in crate::gallery) use gpui_luma::theme::{LumaChrome as GalleryChrome};

/// `apps/gallery/tweakcn/` — shadcn product themes (CSS source of truth).
pub(in crate::gallery) fn tweakcn_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tweakcn")
}

fn theme_css_path(stem: &str) -> PathBuf {
    tweakcn_dir().join(format!("{stem}.css"))
}

/// Lists tweakcn CSS theme stems (e.g. `retro-arcade`) sorted for usage text.
pub(in crate::gallery) fn available_theme_names() -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(tweakcn_dir()) else {
        return Vec::new();
    };

    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension() == Some(OsStr::new("css")))
        .filter_map(|path| path.file_stem().and_then(|s| s.to_str()).map(str::to_string))
        .collect();
    names.sort();
    names
}

/// Startup theme: `default` or a CSS stem under [`tweakcn_dir`] (e.g. `retro-arcade`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GalleryThemeChoice {
    /// Native SDK palette when no tweakcn CSS is selected.
    Default,
    /// `tweakcn/<stem>.css` loaded at runtime.
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

        let path = theme_css_path(&stem);
        if path.is_file() {
            return Ok(Self::Named(stem));
        }

        let available = available_theme_names();
        let hint = if available.is_empty() {
            format!("no .css themes found in {}", tweakcn_dir().display())
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

    /// `cargo run -p gpui-luma-gallery -- <stem>|default|…`
    pub(in crate::gallery) fn cli_usage_line() -> String {
        let program = "gpui-luma-gallery";
        let mut options = vec!["default".to_string()];
        options.extend(available_theme_names());
        format!("cargo run -p {program} -- [{}]", options.join("|"))
    }

    pub fn shadcn_look(self) -> Arc<ShadcnLook> {
        match self {
            Self::Default => Arc::new(ShadcnLook::native()),
            Self::Named(stem) => {
                let path = theme_css_path(&stem);
                Arc::new(
                    ShadcnLook::from_css_path(&path)
                        .unwrap_or_else(|err| panic!("parse shadcn theme {}: {err}", path.display())),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui_luma_look_shadcn::ShadcnLook;

    use super::{available_theme_names, theme_css_path, tweakcn_dir};

    #[test]
    fn tweakcn_dir_exists_and_lists_imported_samples() {
        assert!(tweakcn_dir().is_dir());
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
    fn retro_arcade_css_loads_into_shadcn_look() {
        let path = theme_css_path("retro-arcade");
        let theme = ShadcnLook::from_css_path(&path).expect("retro-arcade css should parse");
        let palette = &theme.mode_tokens().palette;
        assert_ne!(palette.primary.background, palette.secondary.background);
    }

    #[test]
    fn jarvis_css_sans_family_matches_embedded_font() {
        use crate::fonts::RAJDHANI_FAMILY;

        let path = theme_css_path("jarvis");
        let theme = ShadcnLook::from_css_path(&path).expect("jarvis css should parse");
        assert_eq!(theme.mode_tokens().typography.font.sans.family, RAJDHANI_FAMILY);
    }

    #[test]
    fn retro_arcade_css_applies_radius_from_catalog() {
        let path = theme_css_path("retro-arcade");
        let theme = ShadcnLook::from_css_path(&path).expect("retro-arcade css should parse");
        // `--radius: 0.25rem` → 4px base; shadcn rounded-md uses radius - 2px.
        assert_eq!(theme.mode_tokens().metrics.radius.md, 4.0);
        assert_eq!(theme.mode_tokens().metrics.control.md.radius, 2.0);
    }

    #[test]
    fn native_theme_has_empty_css_catalog() {
        let theme = ShadcnLook::native();
        assert!(!theme.has_css_catalog());
    }

    #[test]
    fn css_theme_has_css_catalog() {
        let path = theme_css_path("retro-arcade");
        let theme = ShadcnLook::from_css_path(&path).expect("retro-arcade css should parse");
        assert!(theme.has_css_catalog());
    }
}
