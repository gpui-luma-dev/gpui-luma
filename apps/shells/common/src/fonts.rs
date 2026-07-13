use std::borrow::Cow;

use anyhow::Context as _;
use gpui::App;

pub const RAJDHANI_FAMILY: &str = "Rajdhani Variable";

const RAJDHANI_VARIABLE: &[u8] = include_bytes!("assets/fonts/Rajdhani/Rajdhani-Variable.ttf");

pub fn load_rajdhani(cx: &mut App) -> anyhow::Result<()> {
    cx.text_system()
        .add_fonts(vec![Cow::Borrowed(RAJDHANI_VARIABLE)])
        .context("failed to register Rajdhani variable font")?;

    tracing::info!(family = RAJDHANI_FAMILY, "Rajdhani font registered for Jarvis theme");

    Ok(())
}
