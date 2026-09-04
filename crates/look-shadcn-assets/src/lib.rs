//! Optional Shadcn CSS theme packs.
//!
//! These files are demo/Studio conveniences, not part of the look runtime.
//! Production apps should pass chosen CSS into [`luma_look_shadcn::ShadcnLook::from_css_str`].

mod built_in;

pub use built_in::{BuiltInTheme, built_in_theme, built_in_themes};
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
