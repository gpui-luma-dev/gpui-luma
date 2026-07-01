use std::sync::Arc;

use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::{BuiltInTheme, ShadcnLook, built_in_theme, built_in_themes};

pub fn available_theme_names() -> Vec<String> {
    built_in_themes().iter().map(|theme| theme.id.to_string()).collect()
}

pub fn available_themes() -> &'static [BuiltInTheme] {
    built_in_themes()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioLaunchOptions {
    pub theme_choice: StudioThemeChoice,
    pub initial_mode: ThemeMode,
}

impl StudioLaunchOptions {
    pub fn from_args() -> Self {
        let mut theme_choice = StudioThemeChoice::Default;
        let mut initial_mode = ThemeMode::Dark;

        for arg in std::env::args().skip(1) {
            if let Some(mode_arg) = arg.strip_prefix("--mode=") {
                initial_mode = parse_mode(mode_arg).unwrap_or_else(|usage| {
                    eprintln!("{usage}");
                    std::process::exit(1);
                });
                continue;
            }

            match arg.as_str() {
                "--light" => {
                    initial_mode = ThemeMode::Light;
                    continue;
                }
                "--dark" => {
                    initial_mode = ThemeMode::Dark;
                    continue;
                }
                _ => {}
            }

            theme_choice = StudioThemeChoice::parse(&arg).unwrap_or_else(|usage| {
                eprintln!("{usage}");
                std::process::exit(1);
            });
        }

        Self { theme_choice, initial_mode }
    }

    pub fn loads_rajdhani_font(&self) -> bool {
        self.theme_choice.loads_rajdhani_font()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioThemeChoice {
    Default,
    Named(String),
}

impl StudioThemeChoice {
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

        Err(format!("unknown theme studio theme {arg:?} ({hint})\n{}", usage_line()))
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

fn parse_mode(arg: &str) -> Result<ThemeMode, String> {
    match arg.trim().to_ascii_lowercase().as_str() {
        "light" => Ok(ThemeMode::Light),
        "dark" => Ok(ThemeMode::Dark),
        _ => Err(format!("unknown theme studio mode {arg:?}\n{}", usage_line())),
    }
}

fn usage_line() -> String {
    let program = std::env::args().next().unwrap_or_else(|| "gpui-luma-theme-studio".into());
    let mut options = vec!["default".to_string()];
    options.extend(available_theme_names());
    format!("usage: {program} [theme] [--mode=light|dark|--light|--dark]\nthemes: {}", options.join("|"))
}
