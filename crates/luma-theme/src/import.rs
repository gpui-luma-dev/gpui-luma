use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use gpui_luma::theme::LumaTheme;

use crate::base::load_base_theme;
use crate::catalog::parse_css_catalog;
use crate::emit::{emit_theme_toml, format_binding_report};
use crate::lexicon::load_lexicon;
use crate::resolve::resolve_theme;

#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub css_path: PathBuf,
    pub lexicon_path: PathBuf,
    pub base_path: PathBuf,
    pub out_path: Option<PathBuf>,
    pub report_path: Option<PathBuf>,
    pub name: Option<String>,
    pub validate: bool,
}

#[derive(Debug, Clone)]
pub struct ImportReport {
    pub toml: String,
    pub binding_report: String,
}

pub fn import_theme(options: ImportOptions) -> Result<ImportReport> {
    let css = std::fs::read_to_string(&options.css_path)
        .with_context(|| format!("read CSS catalog {}", options.css_path.display()))?;
    let catalog = parse_css_catalog(&css)?;
    let lexicon = load_lexicon(&options.lexicon_path)?;
    let base = load_base_theme(&options.base_path)?;

    let resolved = resolve_theme(&catalog, &lexicon, &base.light, &base.dark)?;
    let name = options.name.clone().unwrap_or_else(|| base.name.clone());
    let toml = emit_theme_toml(&name, base.version, &base.light, &base.dark, &resolved)?;
    let binding_report = format_binding_report(&resolved);

    if options.validate {
        LumaTheme::from_toml_str(&toml).context("generated theme failed LumaTheme validation")?;
    }

    if let Some(out_path) = &options.out_path {
        std::fs::write(out_path, &toml).with_context(|| format!("write theme TOML {}", out_path.display()))?;
    }
    if let Some(report_path) = &options.report_path {
        std::fs::write(report_path, &binding_report)
            .with_context(|| format!("write binding report {}", report_path.display()))?;
    }

    Ok(ImportReport { toml, binding_report })
}

pub fn default_lexicon_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../sdk/src/theme/lexicon.toml")
}

pub fn default_base_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../sdk/src/theme/default-theme.toml")
}

#[derive(Debug, Clone)]
pub struct ImportDirOptions {
    pub source_dir: PathBuf,
    pub dest_dir: PathBuf,
    pub lexicon_path: PathBuf,
    pub base_path: PathBuf,
    pub validate: bool,
}

/// Map `themes/astrovista.css` → `out/astrovista.toml`.
pub fn toml_output_path(dest_dir: &Path, css_path: &Path) -> PathBuf {
    let stem = css_path.file_stem().and_then(|s| s.to_str()).unwrap_or("theme");
    dest_dir.join(format!("{stem}.toml"))
}

/// Human-readable theme name from a CSS filename stem (`retro-arcade` → `Retro Arcade`).
pub fn theme_name_from_stem(stem: &str) -> String {
    stem.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut out = first.to_uppercase().to_string();
                    out.push_str(chars.as_str());
                    out
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Import every `*.css` file in `source_dir` into `dest_dir` as `*.toml`.
pub fn import_dir(options: ImportDirOptions) -> Result<Vec<PathBuf>> {
    if !options.source_dir.is_dir() {
        bail!("source directory does not exist: {}", options.source_dir.display());
    }
    std::fs::create_dir_all(&options.dest_dir)
        .with_context(|| format!("create destination directory {}", options.dest_dir.display()))?;

    let lexicon = load_lexicon(&options.lexicon_path)?;
    let base = load_base_theme(&options.base_path)?;

    let mut css_files: Vec<PathBuf> = std::fs::read_dir(&options.source_dir)
        .with_context(|| format!("read source directory {}", options.source_dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "css"))
        .collect();
    css_files.sort();

    if css_files.is_empty() {
        bail!("no .css files found in {}", options.source_dir.display());
    }

    let mut written = Vec::new();
    for css_path in css_files {
        let stem = css_path.file_stem().and_then(|s| s.to_str()).unwrap_or("theme");
        let out_path = toml_output_path(&options.dest_dir, &css_path);
        let css =
            std::fs::read_to_string(&css_path).with_context(|| format!("read CSS catalog {}", css_path.display()))?;
        let catalog = parse_css_catalog(&css)?;
        let resolved = resolve_theme(&catalog, &lexicon, &base.light, &base.dark)?;
        let name = theme_name_from_stem(stem);
        let toml = emit_theme_toml(&name, base.version, &base.light, &base.dark, &resolved)?;

        if options.validate {
            LumaTheme::from_toml_str(&toml)
                .with_context(|| format!("generated theme failed validation for {}", css_path.display()))?;
        }

        std::fs::write(&out_path, &toml).with_context(|| format!("write theme TOML {}", out_path.display()))?;
        eprintln!("{} → {}", css_path.display(), out_path.display());
        written.push(out_path);
    }

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_css_stem_to_toml_in_dest_dir() {
        let out = toml_output_path(Path::new("/out"), Path::new("/in/astrovista.css"));
        assert_eq!(out, Path::new("/out/astrovista.toml"));
    }

    #[test]
    fn theme_name_from_stem_title_cases_parts() {
        assert_eq!(theme_name_from_stem("retro-arcade"), "Retro Arcade");
        assert_eq!(theme_name_from_stem("jarvis"), "Jarvis");
    }
}
