use std::sync::Arc;

use gpui_luma_look_shadcn::{BuiltInTheme, ShadcnLook, built_in_theme, built_in_themes};

pub fn available_theme_names() -> Vec<String> {
    built_in_themes().iter().map(|theme| theme.id.to_string()).collect()
}

pub fn available_themes() -> &'static [BuiltInTheme] {
    built_in_themes()
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

        if built_in_theme(&stem).is_some() {
            return Ok(Self::Named(stem));
        }

        let available = available_theme_names();
        let hint = if available.is_empty() {
            "no embedded built-in themes found".to_string()
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
        match self {
            Self::Default => false,
            Self::Named(stem) => built_in_theme(stem).is_some_and(|theme| theme.requires_rajdhani_font),
        }
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
