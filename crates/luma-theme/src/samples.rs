//! Canonical tweakcn CSS inputs checked in under the gallery app.

use std::path::{Path, PathBuf};

/// `apps/gallery/tweakcn/astrovista.css`
pub fn astrovista_css() -> PathBuf {
    gallery_tweakcn_dir().join("astrovista.css")
}

/// `apps/gallery/tweakcn/jarvis.css`
pub fn jarvis_css() -> PathBuf {
    gallery_tweakcn_dir().join("jarvis.css")
}

/// `apps/gallery/tweakcn/enhance-material.css`
pub fn enhance_material_css() -> PathBuf {
    gallery_tweakcn_dir().join("enhance-material.css")
}

/// `apps/gallery/src/assets/themes/tweakcn-astrovista.toml`
pub fn astrovista_golden_toml() -> PathBuf {
    gallery_assets_themes_dir().join("tweakcn-astrovista.toml")
}

/// `apps/gallery/src/assets/themes/tweakcn-jarvis.toml`
pub fn jarvis_golden_toml() -> PathBuf {
    gallery_assets_themes_dir().join("tweakcn-jarvis.toml")
}

pub fn gallery_tweakcn_dir() -> PathBuf {
    workspace_root().join("apps/gallery/tweakcn")
}

fn gallery_assets_themes_dir() -> PathBuf {
    workspace_root().join("apps/gallery/src/assets/themes")
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
