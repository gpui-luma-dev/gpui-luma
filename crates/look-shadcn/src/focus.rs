use super::resolve::resolve_color;
use super::catalog::CssTokenMap;

pub fn focus_ring_color(catalog: &CssTokenMap) -> anyhow::Result<gpui::Hsla> {
    resolve_color(catalog, "ring")
}
