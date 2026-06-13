use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let themes_dir = manifest_dir.join("assets/tweakcn");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));

    println!("cargo:rerun-if-changed={}", themes_dir.display());

    let mut themes = read_theme_ids(&themes_dir);
    themes.sort_unstable();

    let generated = render_built_in_catalog(&themes);
    let generated_path = out_dir.join("built_in_themes.rs");
    fs::write(&generated_path, generated).expect("write generated built-in themes catalog");
}

fn read_theme_ids(themes_dir: &Path) -> Vec<String> {
    let entries = fs::read_dir(themes_dir)
        .unwrap_or_else(|err| panic!("read built-in themes dir {}: {err}", themes_dir.display()));

    let mut ids = Vec::new();
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| panic!("read built-in theme entry: {err}"));
        let path = entry.path();
        println!("cargo:rerun-if-changed={}", path.display());

        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("css") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_else(|| panic!("theme filename must be valid UTF-8: {}", path.display()));
        ids.push(stem.to_string());
    }

    ids
}

fn render_built_in_catalog(ids: &[String]) -> String {
    let mut output = String::from("static BUILT_IN_THEMES: &[BuiltInTheme] = &[\n");

    for id in ids {
        let requires_rajdhani_font = *id == *"jarvis";
        output.push_str("    BuiltInTheme {\n");
        output.push_str(&format!("        id: {id:?},\n"));
        output.push_str(&format!(
            "        css: include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets/tweakcn/{id}.css\")),\n"
        ));
        output.push_str(&format!("        requires_rajdhani_font: {requires_rajdhani_font},\n"));
        output.push_str("    },\n");
    }

    output.push_str("];\n");
    output
}
