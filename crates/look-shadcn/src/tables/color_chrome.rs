//! Shared color-control chrome, using the values synchronized into luma-color.
use crate::{ColorSource, ResolvedColor, ShadcnLook};

#[derive(Clone, Debug)]
pub struct ColorChromeTable {
    pub border: ResolvedColor,
    pub background: ResolvedColor,
}

pub fn resolve_color_chrome(look: &ShadcnLook) -> ColorChromeTable {
    let mode = look.mode_tokens();
    let background_token = if mode.catalog.get("card").is_some() {
        "card"
    } else {
        "background"
    };
    ColorChromeTable {
        border: ResolvedColor {
            value: mode.palette.border_default,
            source: ColorSource::CssVar { token: "border".into() },
        },
        background: ResolvedColor {
            value: mode.palette.panel_background,
            source: ColorSource::CssVar { token: background_token.into() },
        },
    }
}
