use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result};
use google_fonts::{Family, Font};

struct FontSpec {
    name: &'static str,
    family: Family,
}

const FONT_SPECS: &[FontSpec] = &[
    FontSpec { name: "Antic", family: Family::Antic },
    FontSpec { name: "Architects Daughter", family: Family::ArchitectsDaughter },
    FontSpec { name: "Crimson Pro", family: Family::CrimsonPro },
    FontSpec { name: "DM Sans", family: Family::DMSans },
    FontSpec { name: "Fira Code", family: Family::FiraCode },
    FontSpec { name: "IBM Plex Mono", family: Family::IBMPlexMono },
    FontSpec { name: "Inter", family: Family::Inter },
    FontSpec { name: "JetBrains Mono", family: Family::JetBrainsMono },
    FontSpec { name: "Libre Baskerville", family: Family::LibreBaskerville },
    FontSpec { name: "Merriweather", family: Family::Merriweather },
    FontSpec { name: "Montserrat", family: Family::Montserrat },
    FontSpec { name: "Noto Serif Thai", family: Family::NotoSerifThai },
    FontSpec { name: "Open Sans", family: Family::OpenSans },
    FontSpec { name: "Outfit", family: Family::Outfit },
    FontSpec { name: "Orbitron", family: Family::Orbitron },
    FontSpec { name: "Playfair Display", family: Family::PlayfairDisplay },
    FontSpec { name: "Poppins", family: Family::Poppins },
    FontSpec { name: "Raleway", family: Family::Raleway },
    FontSpec { name: "Sorts Mill Goudy", family: Family::SortsMillGoudy },
    FontSpec { name: "Source Serif 4", family: Family::SourceSerif4 },
    FontSpec { name: "Space Mono", family: Family::SpaceMono },
    FontSpec { name: "Share Tech Mono", family: Family::ShareTechMono },
    FontSpec { name: "VT323", family: Family::VT323 },
];

fn main() -> Result<()> {
    let requested = std::env::args().skip(1).collect::<Vec<_>>();
    let specs = if requested.is_empty() || requested.iter().any(|arg| arg == "--all") {
        FONT_SPECS.iter().collect::<Vec<_>>()
    } else {
        requested
            .iter()
            .map(|requested| {
                FONT_SPECS
                    .iter()
                    .find(|spec| spec.name.eq_ignore_ascii_case(requested))
                    .with_context(|| format!("unsupported Google Font {requested:?}"))
            })
            .collect::<Result<Vec<_>>>()?
    };

    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/google");
    for spec in &specs {
        download_font(spec, &asset_root)?;
    }

    println!("Downloaded {} Google font families into {}", specs.len(), asset_root.display());
    Ok(())
}

fn download_font(spec: &FontSpec, asset_root: &Path) -> Result<()> {
    let font = select_font(spec.family).with_context(|| format!("no usable font files for {}", spec.name))?;
    let bytes = font.get_with_cache().with_context(|| format!("download {}", spec.name))?;
    let family_dir = asset_root.join(slug(spec.name));
    fs::create_dir_all(&family_dir).with_context(|| format!("create {}", family_dir.display()))?;

    let filename = if font.is_variable() {
        format!("{}-Variable.ttf", spec.name.replace(' ', ""))
    } else {
        format!("{}-{}.ttf", spec.name.replace(' ', ""), font.name().replace(spec.name, ""))
    };
    let path = family_dir.join(filename);
    fs::write(&path, bytes).with_context(|| format!("write {}", path.display()))?;
    println!("  {} -> {}", spec.name, path.display());
    Ok(())
}

fn select_font(family: Family) -> Option<Font> {
    family
        .fonts()
        .into_iter()
        .find(|font| font.is_variable())
        .or_else(|| family.fonts().into_iter().find(|font| font.name().to_ascii_lowercase().contains("regular")))
}

fn slug(name: &str) -> String {
    name.to_ascii_lowercase().replace(' ', "-")
}
