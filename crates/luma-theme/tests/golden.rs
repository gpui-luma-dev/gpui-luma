use std::fs;
use std::path::Path;

use gpui_luma::theme::LumaTheme;
use luma_theme::{
    base::load_base_theme,
    catalog::parse_css_catalog,
    emit::emit_theme_toml,
    import::{default_base_path, default_lexicon_path},
    lexicon::load_lexicon,
    resolve::resolve_theme,
    samples::{astrovista_css, astrovista_golden_toml, jarvis_css, jarvis_golden_toml},
};

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn parse_hsl_channels(value: &str) -> (f32, f32, f32) {
    let value = value.trim();
    let inner = value
        .strip_prefix("hsla(")
        .or_else(|| value.strip_prefix("hsl("))
        .and_then(|v| v.strip_suffix(')'))
        .unwrap_or_else(|| panic!("expected hsl value, got {value}"));
    let channels = inner.split('/').next().expect("channels").split_whitespace();
    let mut parts = channels.map(|part| {
        part.strip_suffix('%')
            .unwrap_or(part)
            .parse::<f32>()
            .unwrap_or_else(|_| panic!("invalid channel in {value}"))
    });
    let h = parts.next().expect("hue");
    let s = parts.next().expect("saturation");
    let l = parts.next().expect("lightness");
    (h, s, l)
}

fn hsl_equivalent(left: &str, right: &str) -> bool {
    let left = parse_hsl_channels(left);
    let right = parse_hsl_channels(right);
    const EPS: f32 = 0.6;
    (left.0 - right.0).abs() <= EPS && (left.1 - right.1).abs() <= EPS && (left.2 - right.2).abs() <= EPS
}

fn assert_palette_eq(imported: &str, golden: &str, mode: &str, path: &str) {
    let left = get_palette_leaf(imported, mode, path);
    let right = get_palette_leaf(golden, mode, path);
    assert!(hsl_equivalent(&left, &right), "{mode}.{path}: left={left} right={right}");
}

fn get_palette_leaf(toml: &str, mode: &str, path: &str) -> String {
    let root: toml::Value = toml::from_str(toml).expect("parse theme toml");
    let mut current = root.get(mode).expect("mode").as_table().expect("mode table");
    for segment in path.split('.') {
        match current.get(segment).expect(segment) {
            toml::Value::Table(table) => current = table,
            toml::Value::String(value) => return value.clone(),
            other => panic!("unexpected value at {segment}: {other:?}"),
        }
    }
    panic!("path {path} did not end at a string leaf");
}

fn import_css(css_path: &Path, name: &str) -> String {
    let css = read(css_path);
    let catalog = parse_css_catalog(&css).expect("parse css catalog");
    let lexicon = load_lexicon(&default_lexicon_path()).expect("load lexicon");
    let base = load_base_theme(&default_base_path()).expect("load base theme");
    let resolved = resolve_theme(&catalog, &lexicon, &base.light, &base.dark).expect("resolve theme");
    emit_theme_toml(name, base.version, &base.light, &base.dark, &resolved).expect("emit theme toml")
}

#[test]
fn astrovista_css_catalog_has_light_and_dark_tokens() {
    let css = read(&astrovista_css());
    let catalog = parse_css_catalog(&css).expect("astrovista css should parse");
    assert!(catalog.light.contains_key("primary"));
    assert!(catalog.dark.contains_key("primary"));
    assert_eq!(catalog.light.get("background").unwrap(), "hsl(204.0000 12.1951% 91.9608%)");
}

#[test]
fn jarvis_css_catalog_has_light_and_dark_tokens() {
    let css = read(&jarvis_css());
    let catalog = parse_css_catalog(&css).expect("jarvis css should parse");
    assert!(catalog.light.contains_key("font-sans"));
    assert!(catalog.dark.contains_key("font-sans"));
}

#[test]
fn astrovista_import_parses_and_matches_catalog_backed_palette() {
    let imported = import_css(&astrovista_css(), "Astrovista");
    LumaTheme::from_toml_str(&imported).expect("imported astrovista should validate");

    let golden = read(&astrovista_golden_toml());

    for path in [
        "palette.app.background",
        "palette.app.foreground",
        "palette.app.muted_foreground",
        "palette.action.prominent.background",
        "palette.action.prominent.foreground",
        "palette.action.standard.background",
        "palette.action.subtle.background",
        "palette.border.default",
    ] {
        assert_palette_eq(&imported, &golden, "light", path);
    }
}

#[test]
fn jarvis_import_parses_and_matches_catalog_backed_palette() {
    let imported = import_css(&jarvis_css(), "Jarvis");
    LumaTheme::from_toml_str(&imported).expect("imported jarvis should validate");

    let golden = read(&jarvis_golden_toml());

    for path in [
        "palette.app.background",
        "palette.app.foreground",
        "palette.action.prominent.background",
        "palette.action.prominent.foreground",
        "palette.border.default",
    ] {
        assert_palette_eq(&imported, &golden, "light", path);
    }
}
