use gpui::{AssetSource, SharedString};
use anyhow::{Result};
use std::borrow::Cow;

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, _path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        // Always return None or an Error. 
        // This satisfies the trait but provides no data.
        Ok(None)
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        // Return an empty list of assets
        Ok(Vec::new())
    }
}