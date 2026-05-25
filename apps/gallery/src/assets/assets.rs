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
            "assets/labels/outline-label.svg" => Some(include_bytes!("labels/outline-label.svg").as_slice()),
            "assets/labels/subtle-label.svg" => Some(include_bytes!("labels/subtle-label.svg").as_slice()),
            "assets/labels/ghost-label.svg" => Some(include_bytes!("labels/ghost-label.svg").as_slice()),
            "assets/labels/selected-label.svg" => Some(include_bytes!("labels/selected-label.svg").as_slice()),
            "assets/labels/unselected-label.svg" => Some(include_bytes!("labels/unselected-label.svg").as_slice()),
            "assets/labels/default-label.svg" => Some(include_bytes!("labels/default-label.svg").as_slice()),
            "assets/labels/hover-label.svg" => Some(include_bytes!("labels/hover-label.svg").as_slice()),
            "assets/labels/focused-label.svg" => Some(include_bytes!("labels/focused-label.svg").as_slice()),
            "assets/labels/pressed-label.svg" => Some(include_bytes!("labels/pressed-label.svg").as_slice()),
            "assets/labels/disabled-label.svg" => Some(include_bytes!("labels/disabled-label.svg").as_slice()),
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
                SharedString::from("assets/labels/outline-label.svg"),
                SharedString::from("assets/labels/subtle-label.svg"),
                SharedString::from("assets/labels/ghost-label.svg"),
                SharedString::from("assets/labels/selected-label.svg"),
                SharedString::from("assets/labels/unselected-label.svg"),
                SharedString::from("assets/labels/default-label.svg"),
                SharedString::from("assets/labels/hover-label.svg"),
                SharedString::from("assets/labels/focused-label.svg"),
                SharedString::from("assets/labels/pressed-label.svg"),
                SharedString::from("assets/labels/disabled-label.svg"),
            ],
            _ => Vec::new(),
        };

        Ok(entries)
    }
}
