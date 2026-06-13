//! Rajdhani for built-in themes that reference it (e.g. Jarvis in `gpui-luma-look-shadcn`).
//! License: `fonts/Rajdhani/OFL.txt` (SIL OFL 1.1 — keep with the font). See `fonts/Rajdhani/README.md`.

use std::borrow::Cow;

use anyhow::Context as _;
use gpui::App;

/// Must match the typographic family in `Rajdhani-Variable.ttf` (name ID 16).
pub const RAJDHANI_FAMILY: &str = "Rajdhani Variable";

const RAJDHANI_VARIABLE: &[u8] = include_bytes!("fonts/Rajdhani/Rajdhani-Variable.ttf");

/// Registers Rajdhani (variable weight 300–700).
pub fn load_rajdhani(cx: &mut App) -> anyhow::Result<()> {
    cx.text_system()
        .add_fonts(vec![Cow::Borrowed(RAJDHANI_VARIABLE)])
        .context("failed to register Rajdhani variable font")?;

    tracing::info!(family = RAJDHANI_FAMILY, "Rajdhani font registered");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use font_kit::handle::Handle;
    use font_kit::sources::mem::MemSource;

    #[test]
    fn embedded_rajdhani_registers_under_core_text_family_name() {
        let bytes = include_bytes!("fonts/Rajdhani/Rajdhani-Variable.ttf");
        let mut source = MemSource::empty();
        let font = source
            .add_font(Handle::from_memory(Arc::new(bytes.to_vec()), 0))
            .expect("register embedded Rajdhani");

        let family = font.family_name();
        let postscript = font.postscript_name().unwrap_or_default();
        let has_m = font.glyph_for_char('m').is_some();

        assert!(has_m, "Rajdhani must include an 'm' glyph or GPUI will skip loading it");
        assert!(source.select_family_by_name(&family).is_ok(), "GPUI must resolve family {family:?}");
        assert!(!postscript.is_empty());
    }
}
