use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;

use gpui_luma::theme::RadixTheme;

/// `apps/gallery/tweakcn/` — shared Radix product themes with the gallery app.
pub fn tweakcn_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../gallery/tweakcn")
}

fn theme_css_path(stem: &str) -> PathBuf {
    tweakcn_dir().join(format!("{stem}.css"))
}

pub fn available_theme_names() -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioThemeChoice {
    Default,
    Named(String),
}

impl StudioThemeChoice {
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

        Err(format!("unknown theme studio theme {arg:?} ({hint})\n{}", Self::usage_line()))
    }

    fn usage_line() -> String {
        let program = std::env::args().next().unwrap_or_else(|| "gpui-luma-theme-studio".into());
        let mut options = vec!["default".to_string()];
        options.extend(available_theme_names());
        format!("usage: {program} [{}]", options.join("|"))
    }

    /// Jarvis bundles Rajdhani; load the embedded font when that theme is selected.
    pub fn loads_rajdhani_font(&self) -> bool {
        matches!(self, Self::Named(stem) if stem == "jarvis")
    }

    pub fn id(&self) -> String {
        match self {
            Self::Default => "default".to_string(),
            Self::Named(stem) => stem.clone(),
        }
    }

    pub fn from_id(theme_id: &str) -> Self {
        if theme_id == "default" {
            Self::Default
        } else {
            Self::Named(theme_id.to_string())
        }
    }

    pub fn radix_theme(self) -> Arc<RadixTheme> {
        match self {
            Self::Default => Arc::new(RadixTheme::native()),
            Self::Named(stem) => {
                let path = theme_css_path(&stem);
                Arc::new(
                    RadixTheme::from_css_path(&path)
                        .unwrap_or_else(|err| panic!("parse radix theme {}: {err}", path.display())),
                )
            }
        }
    }
}
