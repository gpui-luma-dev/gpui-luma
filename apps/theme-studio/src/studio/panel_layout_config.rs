//! Persists Theme Studio window size in `panel-layout.toml`.

use std::fs;
use std::path::PathBuf;

use gpui::{Pixels, Size, px, size};
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "panel-layout.toml";

pub const DEFAULT_WINDOW_WIDTH_PX: f32 = 1600.0;
pub const DEFAULT_WINDOW_HEIGHT_PX: f32 = 1000.0;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
struct WindowSize {
    width: f32,
    height: f32,
}

impl WindowSize {
    fn to_gpui(self) -> Size<Pixels> {
        size(px(self.width), px(self.height))
    }

    fn from_gpui(size: Size<Pixels>) -> Self {
        Self { width: size.width.as_f32(), height: size.height.as_f32() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StudioLayoutFile {
    #[serde(default)]
    window: WindowSize,
}

impl Default for StudioLayoutFile {
    fn default() -> Self {
        Self { window: WindowSize { width: DEFAULT_WINDOW_WIDTH_PX, height: DEFAULT_WINDOW_HEIGHT_PX } }
    }
}

pub fn config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(CONFIG_FILE)
}

pub fn load_window_size() -> Size<Pixels> {
    load_layout_file().window.to_gpui()
}

pub fn save_studio_layout(window_size: Size<Pixels>) -> anyhow::Result<()> {
    let path = config_path();
    let file = StudioLayoutFile { window: WindowSize::from_gpui(window_size) };
    write_layout_file(&path, &file)?;
    tracing::debug!("saved studio layout to {}", path.display());
    Ok(())
}

fn load_layout_file() -> StudioLayoutFile {
    let path = config_path();
    if !path.exists() {
        let file = StudioLayoutFile::default();
        if let Err(err) = write_layout_file(&path, &file) {
            tracing::warn!("failed to create default layout at {}: {err}", path.display());
        }
        return file;
    }

    match fs::read_to_string(&path) {
        Ok(contents) => parse_layout_file(&contents).unwrap_or_else(|err| {
            tracing::warn!("invalid layout {}: {err}; using defaults", path.display());
            default_layout_file()
        }),
        Err(err) => {
            tracing::warn!("failed to read layout {}: {err}; using defaults", path.display());
            default_layout_file()
        }
    }
}

fn default_layout_file() -> StudioLayoutFile {
    StudioLayoutFile::default()
}

fn parse_layout_file(contents: &str) -> anyhow::Result<StudioLayoutFile> {
    let mut file: StudioLayoutFile = toml::from_str(contents)?;
    if file.window == WindowSize::default() {
        file.window = default_layout_file().window;
    }
    Ok(file)
}

fn write_layout_file(path: &PathBuf, file: &StudioLayoutFile) -> anyhow::Result<()> {
    let body = toml::to_string_pretty(file).map_err(|err| anyhow::anyhow!("serialize layout: {err}"))?;
    let contents = format!("# Luma Theme Studio layout — window size (px).\n\n{body}");
    fs::write(path, contents).map_err(|err| anyhow::anyhow!("write {}: {err}", path.display()))
}
