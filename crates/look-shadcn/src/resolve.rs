use gpui::{Hsla, hsla};

use super::catalog::CssTokenMap;

pub(crate) fn resolve_color(catalog: &CssTokenMap, token: &str) -> anyhow::Result<Hsla> {
    catalog.color(token)
}

/// Paint-safe color lookup. Missing or invalid tokens use the incomplete-palette foreground.
pub(crate) fn resolve_color_or_fallback(catalog: &CssTokenMap, token: &str) -> Hsla {
    resolve_color(catalog, token).unwrap_or_else(|_| hsla(0.0, 0.0, 0.0, 1.0))
}
