//! Embedded fonts shared by the Luma applications.
//!
//! Font family names exposed here must match the names registered by GPUI's
//! text system. CSS aliases are normalized by the look crate separately.

use std::borrow::Cow;

use anyhow::Context as _;
use gpui::App;

/// Typographic family name registered by the bundled Rajdhani variable font.
pub const RAJDHANI_FAMILY: &str = "Rajdhani Variable";
/// Typographic family name registered by the bundled JetBrains Mono variable font.
pub const JETBRAINS_MONO_FAMILY: &str = "JetBrains Mono";

const RAJDHANI_VARIABLE: &[u8] = include_bytes!("../assets/Rajdhani/Rajdhani-Variable.ttf");
const JETBRAINS_MONO_VARIABLE: &[u8] = include_bytes!("../assets/google/jetbrains-mono/JetBrainsMono-Variable.ttf");
const ANTIC: &[u8] = include_bytes!("../assets/google/antic/Antic-Regular.ttf");
const ARCHITECTS_DAUGHTER: &[u8] =
    include_bytes!("../assets/google/architects-daughter/ArchitectsDaughter-ArchitectsDaughterRegular.ttf");
const CRIMSON_PRO: &[u8] = include_bytes!("../assets/google/crimson-pro/CrimsonPro-Variable.ttf");
const DM_SANS: &[u8] = include_bytes!("../assets/google/dm-sans/DMSans-Variable.ttf");
const FIRA_CODE: &[u8] = include_bytes!("../assets/google/fira-code/FiraCode-Variable.ttf");
const IBM_PLEX_MONO: &[u8] = include_bytes!("../assets/google/ibm-plex-mono/IBMPlexMono-IBMPlexMonoRegular.ttf");
const INTER: &[u8] = include_bytes!("../assets/google/inter/Inter-Variable.ttf");
const LIBRE_BASKERVILLE: &[u8] =
    include_bytes!("../assets/google/libre-baskerville/LibreBaskerville-LibreBaskervilleRegular.ttf");
const MERRIWEATHER: &[u8] = include_bytes!("../assets/google/merriweather/Merriweather-Regular.ttf");
const MONTSERRAT: &[u8] = include_bytes!("../assets/google/montserrat/Montserrat-Variable.ttf");
const NOTO_SERIF_THAI: &[u8] = include_bytes!("../assets/google/noto-serif-thai/NotoSerifThai-Variable.ttf");
const OPEN_SANS: &[u8] = include_bytes!("../assets/google/open-sans/OpenSans-Variable.ttf");
const OUTFIT: &[u8] = include_bytes!("../assets/google/outfit/Outfit-Variable.ttf");
const ORBITRON: &[u8] = include_bytes!("../assets/google/orbitron/Orbitron-Variable.ttf");
const PLAYFAIR_DISPLAY: &[u8] = include_bytes!("../assets/google/playfair-display/PlayfairDisplay-Variable.ttf");
const POPPINS: &[u8] = include_bytes!("../assets/google/poppins/Poppins-Regular.ttf");
const RALEWAY: &[u8] = include_bytes!("../assets/google/raleway/Raleway-Variable.ttf");
const SORTS_MILL_GOUDY: &[u8] =
    include_bytes!("../assets/google/sorts-mill-goudy/SortsMillGoudy-SortsMillGoudyRegular.ttf");
const SOURCE_SERIF_4: &[u8] = include_bytes!("../assets/google/source-serif-4/SourceSerif4-Variable.ttf");
const SPACE_MONO: &[u8] = include_bytes!("../assets/google/space-mono/SpaceMono-SpaceMonoRegular.ttf");
const SHARE_TECH_MONO: &[u8] =
    include_bytes!("../assets/google/share-tech-mono/ShareTechMono-ShareTechMonoRegular.ttf");
const VT323: &[u8] = include_bytes!("../assets/google/vt323/VT323-Regular.ttf");

/// Register the fonts bundled by this crate with GPUI.
pub fn register(cx: &mut App) -> anyhow::Result<()> {
    cx.text_system()
        .add_fonts(vec![
            Cow::Borrowed(RAJDHANI_VARIABLE),
            Cow::Borrowed(ANTIC),
            Cow::Borrowed(ARCHITECTS_DAUGHTER),
            Cow::Borrowed(CRIMSON_PRO),
            Cow::Borrowed(DM_SANS),
            Cow::Borrowed(FIRA_CODE),
            Cow::Borrowed(IBM_PLEX_MONO),
            Cow::Borrowed(INTER),
            Cow::Borrowed(JETBRAINS_MONO_VARIABLE),
            Cow::Borrowed(LIBRE_BASKERVILLE),
            Cow::Borrowed(MERRIWEATHER),
            Cow::Borrowed(MONTSERRAT),
            Cow::Borrowed(NOTO_SERIF_THAI),
            Cow::Borrowed(OPEN_SANS),
            Cow::Borrowed(OUTFIT),
            Cow::Borrowed(ORBITRON),
            Cow::Borrowed(PLAYFAIR_DISPLAY),
            Cow::Borrowed(POPPINS),
            Cow::Borrowed(RALEWAY),
            Cow::Borrowed(SORTS_MILL_GOUDY),
            Cow::Borrowed(SOURCE_SERIF_4),
            Cow::Borrowed(SPACE_MONO),
            Cow::Borrowed(SHARE_TECH_MONO),
            Cow::Borrowed(VT323),
        ])
        .context("failed to register bundled Luma fonts")?;

    tracing::info!("Luma fonts registered");
    Ok(())
}

/// Register all bundled fonts with GPUI.
pub fn register_all(cx: &mut App) -> anyhow::Result<()> {
    register(cx)
}
