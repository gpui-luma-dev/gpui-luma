//! Shared demo assets for Luma apps (Studio, shells, color-viz).
//!
//! Embeds tweakcn CSS packs and fonts referenced by those themes. Not part of the
//! SDK or look runtime — production apps should supply their own CSS and fonts.

mod built_in;
mod fonts;

pub use built_in::{BuiltInTheme, built_in_theme, built_in_themes};
pub use fonts::{GEIST_MONO_FAMILY, JETBRAINS_MONO_FAMILY, RAJDHANI_FAMILY, register, register_all};
use luma_look_shadcn::ShadcnLook;

/// Default sample theme CSS (`native.css`).
pub const NATIVE_CSS: &str = include_str!("../assets/native.css");

/// Look parsed from [`NATIVE_CSS`].
pub fn native_look() -> ShadcnLook {
    ShadcnLook::from_css_str(NATIVE_CSS).expect("embedded shadcn native CSS should parse")
}

/// Look parsed from a bundled tweakcn theme id.
pub fn built_in_look(theme_id: &str) -> anyhow::Result<ShadcnLook> {
    let Some(theme) = built_in_theme(theme_id) else {
        anyhow::bail!("unknown built-in shadcn theme `{theme_id}`");
    };
    ShadcnLook::from_css_str(theme.css)
}
