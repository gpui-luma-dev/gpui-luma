//! Persists demo card positions and window size in `panel-layout.toml`.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use gpui::{Point, Pixels, Size, point, px, size};
use serde::{Deserialize, Serialize};

use super::inspectable::InspectableId;
use super::panel_layout::{default_panel_position, default_panel_positions, panel_width};

const CONFIG_FILE: &str = "panel-layout.toml";

pub const DEFAULT_WINDOW_WIDTH_PX: f32 = 1600.0;
pub const DEFAULT_WINDOW_HEIGHT_PX: f32 = 1000.0;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
struct WindowSize {
    width: f32,
    height: f32,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: DEFAULT_WINDOW_WIDTH_PX,
            height: DEFAULT_WINDOW_HEIGHT_PX,
        }
    }
}

impl WindowSize {
    fn to_gpui(self) -> Size<Pixels> {
        size(px(self.width), px(self.height))
    }

    fn from_gpui(size: Size<Pixels>) -> Self {
        Self {
            width: size.width.as_f32(),
            height: size.height.as_f32(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PanelCorner {
    right: f32,
    top: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StudioLayoutFile {
    #[serde(default)]
    window: WindowSize,
    /// Panel corners live at the root of the TOML file (e.g. `[upgrade_subscription]`).
    #[serde(default, flatten)]
    panels: HashMap<String, PanelCorner>,
}

pub fn config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(CONFIG_FILE)
}

pub fn load_window_size() -> Size<Pixels> {
    load_layout_file().window.to_gpui()
}

pub fn load_panel_positions() -> HashMap<InspectableId, Point<Pixels>> {
    positions_from_file(&load_layout_file())
}

pub fn save_studio_layout(window_size: Size<Pixels>, positions: &HashMap<InspectableId, Point<Pixels>>) -> anyhow::Result<()> {
    let path = config_path();
    let mut panels = HashMap::new();
    for &id in InspectableId::all() {
        let top_left = positions.get(&id).copied().unwrap_or_else(|| default_panel_position(id));
        let corner = corner_from_top_left(top_left, id);
        panels.insert(id.config_key().to_string(), corner);
    }

    let file = StudioLayoutFile {
        window: WindowSize::from_gpui(window_size),
        panels,
    };
    write_layout_file(&path, &file)?;
    tracing::debug!("saved studio layout to {}", path.display());
    Ok(())
}

fn load_layout_file() -> StudioLayoutFile {
    let path = config_path();
    if !path.exists() {
        let file = default_layout_file();
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
    let mut panels = HashMap::new();
    for (&id, &top_left) in default_panel_positions().iter() {
        panels.insert(id.config_key().to_string(), corner_from_top_left(top_left, id));
    }
    StudioLayoutFile {
        window: WindowSize::default(),
        panels,
    }
}

fn parse_layout_file(contents: &str) -> anyhow::Result<StudioLayoutFile> {
    let mut file: StudioLayoutFile = toml::from_str(contents)?;
    if file.panels.is_empty() {
        file.panels = default_layout_file().panels;
    }
    Ok(file)
}

fn positions_from_file(file: &StudioLayoutFile) -> HashMap<InspectableId, Point<Pixels>> {
    let mut positions = HashMap::new();
    for &id in InspectableId::all() {
        let top_left = file
            .panels
            .get(id.config_key())
            .map(|corner| top_left_from_corner(corner.right, corner.top, id))
            .unwrap_or_else(|| default_panel_position(id));
        positions.insert(id, top_left);
    }
    positions
}

fn write_layout_file(path: &PathBuf, file: &StudioLayoutFile) -> anyhow::Result<()> {
    let body = toml::to_string_pretty(file).map_err(|err| anyhow::anyhow!("serialize layout: {err}"))?;
    let contents = format!(
        "# Luma Theme Studio layout — window size (px) and panel upper-right corners (right, top).\n\n{body}"
    );
    fs::write(path, contents).map_err(|err| anyhow::anyhow!("write {}: {err}", path.display()))
}

fn top_left_from_corner(right: f32, top: f32, id: InspectableId) -> Point<Pixels> {
    point(px(right - panel_width(id)), px(top))
}

fn corner_from_top_left(pos: Point<Pixels>, id: InspectableId) -> PanelCorner {
    PanelCorner {
        right: pos.x.as_f32() + panel_width(id),
        top: pos.y.as_f32(),
    }
}
