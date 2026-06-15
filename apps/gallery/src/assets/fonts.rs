//! Rajdhani for the Jarvis gallery theme ([Fontshare](https://www.fontshare.com/fonts/rajdhani)).
//! License: `fonts/Rajdhani/OFL.txt` (SIL OFL 1.1 — keep with the font). See `fonts/Rajdhani/README.md`.
//!
//! Astrovista (default) uses system/Outfit — call [`load_rajdhani`] before switching to `themes::JARVIS`.

use std::borrow::Cow;

use anyhow::Context as _;
use gpui::App;

/// Must match the typographic family in `Rajdhani-Variable.ttf` (name ID 16).
/// tweakcn exports `--font-sans: Rajdhani`; the catalog maps that alias at load time.
pub const RAJDHANI_FAMILY: &str = "Rajdhani Variable";

const RAJDHANI_VARIABLE: &[u8] = include_bytes!("fonts/Rajdhani/Rajdhani-Variable.ttf");

/// Monospace for inspector and token-style values. Theme `--font-mono` families (e.g. Space Mono)
/// are not registered with GPUI unless explicitly embedded like Rajdhani.
pub fn gallery_mono_font() -> gpui::SharedString {
    #[cfg(target_os = "macos")]
    {
        return "Menlo".into();
    }
    #[cfg(target_os = "windows")]
    {
        return "Consolas".into();
    }
    #[cfg(target_os = "linux")]
    {
        return "DejaVu Sans Mono".into();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "monospace".into()
    }
}

/// Registers Rajdhani (variable weight 300–700). No-op safe to call more than once only if GPUI allows; call once before Jarvis theme.
pub fn load_rajdhani(cx: &mut App) -> anyhow::Result<()> {
    cx.text_system()
        .add_fonts(vec![Cow::Borrowed(RAJDHANI_VARIABLE)])
        .context("failed to register Rajdhani variable font")?;

    tracing::info!(family = RAJDHANI_FAMILY, "Rajdhani font registered for Jarvis theme");

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
        eprintln!("family={family:?} postscript={postscript:?} has_m={has_m}");

        let families = source.all_families().expect("list families");
        eprintln!("mem source families: {families:?}");

        assert!(has_m, "Rajdhani must include an 'm' glyph or GPUI will skip loading it");
        assert!(source.select_family_by_name(&family).is_ok(), "GPUI must resolve family {family:?}");
    }
}
