//! Rajdhani from [Fontshare](https://www.fontshare.com/fonts/rajdhani) (OFL, see `Rajdhani_Complete/License/OFL.txt`).
//!
//! GPUI needs raw TTF/OTF bytes (`add_fonts`), not the WEB `.woff` bundle.

use std::borrow::Cow;

use anyhow::Context as _;
use gpui::App;

/// Family name to use in `.font_family(...)` / `theme.toml` `typography.font.sans`.
pub const RAJDHANI_FAMILY: &str = "Rajdhani";

const RAJDHANI_VARIABLE: &[u8] = include_bytes!("Rajdhani_Complete/Fonts/TTF/Rajdhani-Variable.ttf");
const RAJDHANI_REGULAR: &[u8] = include_bytes!("Rajdhani_Complete/Fonts/WEB/fonts/Rajdhani-Regular.ttf");
const RAJDHANI_MEDIUM: &[u8] = include_bytes!("Rajdhani_Complete/Fonts/WEB/fonts/Rajdhani-Medium.ttf");
const RAJDHANI_SEMIBOLD: &[u8] = include_bytes!("Rajdhani_Complete/Fonts/WEB/fonts/Rajdhani-SemiBold.ttf");

pub fn load_rajdhani(cx: &mut App) -> anyhow::Result<()> {
    cx.text_system()
        .add_fonts(vec![
            Cow::Borrowed(RAJDHANI_VARIABLE),
            Cow::Borrowed(RAJDHANI_REGULAR),
            Cow::Borrowed(RAJDHANI_MEDIUM),
            Cow::Borrowed(RAJDHANI_SEMIBOLD),
        ])
        .context("failed to register Rajdhani fonts")?;

    tracing::info!(family = RAJDHANI_FAMILY, "gallery sans fonts registered");

    Ok(())
}
