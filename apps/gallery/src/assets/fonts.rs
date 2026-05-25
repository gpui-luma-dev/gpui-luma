//! Rajdhani for the Jarvis gallery theme ([Fontshare](https://www.fontshare.com/fonts/rajdhani)).
//! License: `fonts/Rajdhani/OFL.txt` (SIL OFL 1.1 — keep with the font). See `fonts/Rajdhani/README.md`.
//!
//! Astrovista (default) uses system/Outfit — call [`load_rajdhani`] before switching to `themes::JARVIS`.

use std::borrow::Cow;

use anyhow::Context as _;
use gpui::App;

/// Must match `themes/tweakcn-jarvis.toml` `typography.font.sans.family` and the name table in `Rajdhani-Variable.ttf`.
pub const RAJDHANI_FAMILY: &str = "Rajdhani Variable";

const RAJDHANI_VARIABLE: &[u8] = include_bytes!("fonts/Rajdhani/Rajdhani-Variable.ttf");

/// Registers Rajdhani (variable weight 300–700). No-op safe to call more than once only if GPUI allows; call once before Jarvis theme.
pub fn load_rajdhani(cx: &mut App) -> anyhow::Result<()> {
    cx.text_system()
        .add_fonts(vec![Cow::Borrowed(RAJDHANI_VARIABLE)])
        .context("failed to register Rajdhani variable font")?;

    tracing::info!(family = RAJDHANI_FAMILY, "Rajdhani font registered for Jarvis theme");

    Ok(())
}
