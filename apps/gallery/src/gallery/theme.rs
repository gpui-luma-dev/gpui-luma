use std::sync::Arc;

use gpui_luma_look_shadcn::{ShadcnLook, built_in_theme, built_in_themes};

pub(in crate::gallery) use gpui_luma::theme::{LumaChrome as GalleryChrome};

/// Lists embedded built-in shadcn theme IDs (e.g. `retro-arcade`) sorted for usage text.
pub(in crate::gallery) fn available_theme_names() -> Vec<String> {
    built_in_themes().iter().map(|theme| theme.id.to_string()).collect()
}

/// Startup theme: `default` or a built-in CSS theme ID (e.g. `retro-arcade`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GalleryThemeChoice {
    /// Native SDK palette when no tweakcn CSS is selected.
    Default,
    /// Built-in shadcn theme loaded from `gpui-luma-look-shadcn`.
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

        if built_in_theme(&stem).is_some() {
            return Ok(Self::Named(stem));
        }

        let available = available_theme_names();
        let hint = if available.is_empty() {
            "no embedded built-in themes found".to_string()
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
        match self {
            Self::Default => false,
            Self::Named(stem) => built_in_theme(stem).is_some_and(|theme| theme.requires_rajdhani_font),
        }
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
            Self::Named(stem) => Arc::new(
                ShadcnLook::from_built_in_theme(&stem)
                    .unwrap_or_else(|err| panic!("parse built-in shadcn theme {stem}: {err}")),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui_luma_look_shadcn::{ShadcnLook, built_in_theme};

    use super::available_theme_names;

    #[test]
    fn embedded_theme_list_contains_imported_samples() {
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
        let theme = ShadcnLook::from_built_in_theme("retro-arcade").expect("retro-arcade css should parse");
        let palette = &theme.mode_tokens().palette;
        assert_ne!(palette.primary.background, palette.secondary.background);
    }

    #[test]
    fn jarvis_css_sans_family_matches_embedded_font() {
        use crate::fonts::RAJDHANI_FAMILY;

        let jarvis = built_in_theme("jarvis").expect("jarvis");
        assert!(jarvis.requires_rajdhani_font);
        let theme = ShadcnLook::from_built_in_theme("jarvis").expect("jarvis css should parse");
        assert_eq!(theme.mode_tokens().typography.font.sans.family, RAJDHANI_FAMILY);
    }

    #[test]
    fn retro_arcade_css_applies_radius_from_catalog() {
        let theme = ShadcnLook::from_built_in_theme("retro-arcade").expect("retro-arcade css should parse");
        // `--radius: 0.25rem` → 4px base; shadcn rounded-md uses radius - 2px.
        assert_eq!(theme.mode_tokens().metrics.radius.md, 4.0);
        assert_eq!(theme.mode_tokens().metrics.control.md.radius, 2.0);
    }

    #[test]
    fn native_theme_has_css_catalog() {
        let theme = ShadcnLook::native();
        assert!(theme.has_css_catalog());
    }

    #[test]
    fn css_theme_has_css_catalog() {
        let theme = ShadcnLook::from_built_in_theme("retro-arcade").expect("retro-arcade css should parse");
        assert!(theme.has_css_catalog());
    }
}
