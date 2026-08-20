use anyhow::Result;
use gpui::{AssetSource, SharedString};
use lucide_svg_static::asset_bytes;
use std::{borrow::Cow, fs, path::PathBuf};

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = asset_bytes(path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }

        let Some(asset_path) = resolve_asset_path(path) else {
            return Ok(None);
        };

        match fs::read(asset_path) {
            Ok(bytes) => Ok(Some(Cow::Owned(bytes))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let path = path.trim_start_matches('/');
        if path.is_empty() {
            return Ok(vec![SharedString::from("assets")]);
        }

        let Some(directory) = resolve_asset_path(path) else {
            return Ok(Vec::new());
        };

        if !directory.is_dir() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            entries.push(SharedString::from(format!("{path}/{name}")));
        }
        entries.sort();
        Ok(entries)
    }
}

fn resolve_asset_path(path: &str) -> Option<PathBuf> {
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        return Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/assets"));
    }
    if !path.starts_with("assets/") {
        return None;
    }
    Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join(path))
}
