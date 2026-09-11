//! `@radix-ui/react-icons` SVG catalog for Radix Studio.
//!
//! SVG sources live in `../react-icons/*.svg` (Radix Icons, MIT).

mod catalog;
mod groups;

pub use catalog::Icon;
pub use groups::ICON_GROUPS;

use gpui::{AnyElement, Hsla, px, prelude::*, svg};

/// Resolves an `assets/react-icons/...` path to embedded SVG bytes.
pub fn asset_bytes(path: &str) -> Option<&'static [u8]> {
    Icon::from_asset_path(path).map(Icon::svg_bytes)
}

/// Looks up an icon by its kebab-case basename (e.g. `"chevron-down"`).
pub fn icon_named(name: &str) -> Option<Icon> {
    Icon::from_asset_path(&format!("assets/react-icons/{name}.svg"))
}

/// Renders a react-icons glyph at the given size (native icons are 15×15).
pub fn react_icon(icon: Icon, color: Hsla, size: f32) -> AnyElement {
    svg().size(px(size)).path(icon.asset_path()).text_color(color).into_any_element()
}
