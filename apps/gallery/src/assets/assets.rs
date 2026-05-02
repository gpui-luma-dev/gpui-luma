use anyhow::Result;
use gpui::{AssetSource, SharedString};
use std::borrow::Cow;

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let path = path.trim_start_matches('/');

        let bytes = match path {
            "assets/labels/prominent-label.svg" => Some(include_bytes!("labels/prominent-label.svg").as_slice()),
            "assets/labels/standard-label.svg" => Some(include_bytes!("labels/standard-label.svg").as_slice()),
            "assets/labels/ghost-label.svg" => Some(include_bytes!("labels/ghost-label.svg").as_slice()),
            "assets/labels/selected-label.svg" => Some(include_bytes!("labels/selected-label.svg").as_slice()),
            "assets/labels/unselected-label.svg" => Some(include_bytes!("labels/unselected-label.svg").as_slice()),
            _ => None,
        };

        Ok(bytes.map(Cow::Borrowed))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let path = path.trim_start_matches('/');

        let entries: Vec<SharedString> = match path {
            "" => vec![SharedString::from("assets")],
            "assets" => vec![SharedString::from("assets/labels")],
            "assets/labels" => vec![
                SharedString::from("assets/labels/prominent-label.svg"),
                SharedString::from("assets/labels/standard-label.svg"),
                SharedString::from("assets/labels/ghost-label.svg"),
                SharedString::from("assets/labels/selected-label.svg"),
                SharedString::from("assets/labels/unselected-label.svg"),
            ],
            _ => Vec::new(),
        };

        Ok(entries)
    }
}
