use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use luma_theme::{ImportDirOptions, ImportOptions, import_dir, import_theme};

#[derive(Parser)]
#[command(name = "luma-theme", about = "Import shadcn/tweakcn CSS into Luma theme.toml")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Import one CSS file, or every `*.css` in a directory.
    Import {
        /// Single CSS file (omit when using --source-dir).
        css: Option<PathBuf>,

        /// Import all `*.css` files from this directory.
        #[arg(long)]
        source_dir: Option<PathBuf>,

        /// Output directory; each `name.css` becomes `name.toml`.
        #[arg(long)]
        dest_dir: Option<PathBuf>,

        #[arg(long, default_value = "crates/sdk/src/theme/lexicon.toml")]
        lexicon: PathBuf,

        #[arg(long, default_value = "crates/sdk/src/theme/default-theme.toml")]
        base: PathBuf,

        /// Output path for a single-file import (stdout when omitted).
        #[arg(long)]
        out: Option<PathBuf>,

        #[arg(long)]
        report: Option<PathBuf>,

        /// Theme `name` for a single-file import (derived from filename in batch mode).
        #[arg(long)]
        name: Option<String>,

        #[arg(long, default_value_t = true)]
        validate: bool,
    },

    /// Parse CSS into a flat light/dark token catalog (debug).
    Catalog { css: PathBuf },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Import { css, source_dir, dest_dir, lexicon, base, out, report, name, validate } => {
            match (css, source_dir) {
                (Some(css_path), None) => {
                    let result = import_theme(ImportOptions {
                        css_path,
                        lexicon_path: lexicon,
                        base_path: base,
                        out_path: out.clone(),
                        report_path: report,
                        name,
                        validate,
                    })?;
                    if out.is_none() {
                        print!("{}", result.toml);
                    }
                }
                (None, Some(source_dir)) => {
                    let dest_dir =
                        dest_dir.ok_or_else(|| anyhow::anyhow!("--dest-dir is required when using --source-dir"))?;
                    if out.is_some() || report.is_some() {
                        bail!("--out and --report apply only to single-file import; use --dest-dir with --source-dir");
                    }
                    let written = import_dir(ImportDirOptions {
                        source_dir,
                        dest_dir,
                        lexicon_path: lexicon,
                        base_path: base,
                        validate,
                    })?;
                    eprintln!("wrote {} theme(s)", written.len());
                }
                (Some(_), Some(_)) => bail!("pass either a CSS file or --source-dir, not both"),
                (None, None) => bail!("pass a CSS file or --source-dir"),
            }
        }
        Command::Catalog { css } => {
            let source = std::fs::read_to_string(&css)?;
            let catalog = luma_theme::catalog::parse_css_catalog(&source)?;
            println!("light tokens: {}", catalog.light.len());
            for (name, value) in &catalog.light {
                println!("  {name}: {value}");
            }
            println!("dark tokens: {}", catalog.dark.len());
            for (name, value) in &catalog.dark {
                println!("  {name}: {value}");
            }
        }
    }
    Ok(())
}
